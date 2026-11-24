use std::path::Path;

use crate::model::{GameInstall, InstallSource};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InstallError {
    #[error("the folder does not exist")]
    Missing,
    #[error("the folder is not a World of Warships installation")]
    NotAGameFolder,
    #[error("no game build was found under bin")]
    NoBuild,
}

pub fn detect() -> Vec<GameInstall> {
    todo!("leaf: game-detection")
}

pub fn inspect(root: &Path, source: InstallSource) -> Result<GameInstall, InstallError> {
    let _ = (root, source);
    todo!("leaf: game-detection")
}
