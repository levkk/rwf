use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{Controller, Error};
use crate::http::{Request, Response};

use async_trait::async_trait;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::sync::oneshot::channel;
use tracing::{info, warn};

use tokio::fs::{metadata, File};

use rwf_ruby::{RackRequest, RackResponseOwned, Ruby};
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

    fn runtime(threads: usize) -> ThreadPool {
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .panic_handler(|_| {
                warn!("Rack thread panicked. This is a bug in the Rack application.");
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
        let query = request.query().to_string();
        let req_uri = format!("{}{}", req_path, query);

        let mut env = HashMap::from([
            ("REQUEST_URI".into(), req_uri),
            ("PATH_INFO".into(), req_path.clone()),
            ("REQUEST_PATH".into(), req_path),
            ("SERVER_PROTOCOL".into(), "HTTP/1.1".into()),
            ("REQUEST_METHOD".into(), method),
            ("QUERY_STRING".into(), query.replace("?", "")),
        ]);

        for (key, value) in request.headers().iter() {
            env.insert(
                format!("HTTP_{}", crate::snake_case(key).to_ascii_uppercase()),
                value.to_string(),
            );
        }

        self.pool.spawn(move || {
            // We only have one thread in Rust, so there is no race.
            // Besides, if you try this from multiple threads, you'll segfault.
            if !loaded.load(Ordering::Relaxed) {
                info!("Loading the Rack app, hold your horses...");
                Ruby::load_app(&path).unwrap();
                loaded.store(true, Ordering::Relaxed);
                info!("Rack app loaded, let's go!");
            }

            let response = RackRequest::send(env).unwrap();
            let owned = RackResponseOwned::from(response);

            let _ = tx.send(owned);
        });

        let response = rx.await.unwrap();

        if response.is_file() {
            let path = PathBuf::from(String::from_utf8_lossy(response.body()).to_string());

            let meta = if let Ok(meta) = metadata(&path).await {
                meta
            } else {
                return Ok(Response::not_found());
            };

            // Don't think the file will disappear here, but you really can't know.
            let file = if let Ok(file) = File::open(&path).await {
                file
            } else {
                return Ok(Response::not_found());
            };

            Ok(Response::new().body((path, file, meta)))
        } else {
            let mut res = Response::new().body(response.body());
            for (key, value) in response.headers() {
                res = res.header(key, value);
            }

            Ok(res.code(response.code()))
        }
    }
}
