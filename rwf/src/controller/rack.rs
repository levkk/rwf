use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{Controller, Error};
use crate::http::{Request, Response};

use async_trait::async_trait;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::sync::oneshot::channel;
use tokio::time::{timeout, Duration};
use tracing::warn;

use tokio::fs::{metadata, File};

use rwf_ruby::{RackRequest, RackResponse, RackResponseOwned, Ruby};
use std::sync::Arc;

pub struct RackController {
    pool: ThreadPool,
    path: PathBuf,
    loaded: Arc<AtomicBool>,
}

impl RackController {
    pub fn new(path: &str) -> Self {
        Self {
            // There can only be _one_ Rust thread.
            // The only way to multi-thread safely
            // would be to do it in Ruby using GVL-protected threads.
            // Even if we have a Mutex in Rust, loading the app in one thread and running it in
            // another will segfault.
            pool: Self::runtime(1),
            path: PathBuf::from(path).join("config/environment.rb"),
            loaded: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn load(&self) {
        Ruby::load_app(&self.path).unwrap();
    }

    fn runtime(threads: usize) -> ThreadPool {
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .panic_handler(|_| {
                warn!("WSGI thread panicked. This is a bug in the WSGI application.");
            })
            .build()
            .unwrap()
    }
}

#[async_trait]
impl Controller for RackController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let (tx, rx) = channel();
        let path = PathBuf::from(&self.path);
        let loaded = self.loaded.clone();

        let req_path = request.path().path().to_string();
        let method = request.method().to_string();

        self.pool.spawn(move || {
            if !loaded.load(Ordering::Relaxed) {
                Ruby::load_app(&path).unwrap();
                loaded.store(true, Ordering::Relaxed);
            }

            let env = HashMap::from([
                ("REQUEST_URI".into(), req_path.clone()),
                ("PATH_INFO".into(), req_path.clone()),
                ("REQUEST_PATH".into(), req_path.clone()),
                ("SERVER_PROTOCOL".into(), "HTTP/1.1".into()),
                ("HTTP_HOST".into(), ("127.0.0.1:8000".into())),
                ("REQUEST_METHOD".into(), method),
            ]);

            let response = RackRequest::send(env);
            let owned = RackResponseOwned::from(response);

            let _ = tx.send(owned);
        });

        let response = rx.await.unwrap();

        if response.is_file() {
            let path = PathBuf::from(String::from_utf8_lossy(response.body()).to_string());
            let meta = metadata(&path).await.unwrap();
            let file = File::open(&path).await.unwrap();

            Ok(Response::new().body((path, file, meta)))
        } else {
            Ok(Response::new()
                .body(response.body())
                .header("Content-Type", "text/html")
                .code(response.code()))
        }
    }
}
