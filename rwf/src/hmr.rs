use std::path::PathBuf;
use std::time::Duration;

use notify::{
    event::{AccessKind, AccessMode},
    Event, EventKind, RecursiveMode, Result, Watcher,
};
use tokio::time::sleep;

use crate::http::websocket::Message;
use crate::{comms::Comms, view::TurboStream};

#[cfg(debug_assertions)]
pub fn hmr(path: PathBuf) {
    tokio::task::spawn(async move {
        let mut watcher = notify::recommended_watcher(|res: Result<Event>| match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                        let everyone = Comms::notify();
                        let reload = TurboStream::new("").action("reload-page").render();
                        let _ = everyone.send(Message::Text(reload));
                    }
                    _ => {}
                };
            }
            Err(_) => {}
        })?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        sleep(Duration::MAX).await;

        Result::Ok(())
    });
}

#[cfg(not(debug_assertions))]
pub fn hmr(_path: PathBuf) {}
