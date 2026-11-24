use std::path::PathBuf;

use crate::model::ArenaInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArenaEvent {
    Started(ArenaInfo),
    Ended,
}

#[derive(Debug, thiserror::Error)]
pub enum WatchError {
    #[error("the replays folder {0} does not exist")]
    MissingDir(PathBuf),
    #[error("could not watch the replays folder: {0}")]
    Notify(String),
}

pub struct ArenaWatcher {
    _private: (),
}

impl ArenaWatcher {
    pub fn start(
        replays_dir: PathBuf,
        on_event: impl Fn(ArenaEvent) + Send + Sync + 'static,
    ) -> Result<ArenaWatcher, WatchError> {
        let _ = (replays_dir, on_event);
        todo!("leaf: arena-watcher")
    }
}
