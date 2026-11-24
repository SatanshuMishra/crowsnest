use crate::model::ArenaInfo;

pub const FILE_NAME: &str = "tempArenaInfo.json";

#[derive(Debug, thiserror::Error)]
pub enum ArenaError {
    #[error("tempArenaInfo.json is not valid JSON yet: {0}")]
    Json(#[from] serde_json::Error),
    #[error("tempArenaInfo.json has no vehicles")]
    Empty,
}

pub fn parse(bytes: &[u8]) -> Result<ArenaInfo, ArenaError> {
    let _ = bytes;
    todo!("leaf: arena-watcher")
}
