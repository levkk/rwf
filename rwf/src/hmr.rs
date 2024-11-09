#![allow(unused_imports)]
use std::path::PathBuf;
use std::time::Duration;

use notify::{
    event::{AccessKind, AccessMode},
    Event, EventKind, RecursiveMode, Result, Watcher,
};
use tokio::time::sleep;

use crate::http::websocket::Message;
use crate::{comms::Comms, view::TurboStream};

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;

#[cfg(debug_assertions)]
pub fn hmr(path: PathBuf) {
    use notify::event::ModifyKind;
    use tracing::info;

    let last_reload = Arc::new(Mutex::new(Instant::now()));

    tokio::task::spawn(async move {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event>| match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Access(AccessKind::Close(AccessMode::Write))
                    | EventKind::Modify(ModifyKind::Data(_)) => {
                        let since_last_reload = last_reload.lock().elapsed();
                        *last_reload.lock() = Instant::now();

                        if since_last_reload > Duration::from_millis(250) {
                            let everyone = Comms::notify();
                            let reload = TurboStream::new("").action("reload-page").render();
                            let _ = everyone.send(Message::Text(reload));
                            info!("Starting hot reload");
                        }
                    }
                    _ => {}
                };
            }
            Err(_) => {}
        })?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        info!("Hot reload enabled");

        sleep(Duration::MAX).await;

        Result::Ok(())
    });
}

#[cfg(not(debug_assertions))]
pub fn hmr(_path: PathBuf) {}
