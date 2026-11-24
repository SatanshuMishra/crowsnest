use std::path::{Path, PathBuf};

use crate::model::Region;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaysLocation {
    pub dir: PathBuf,
    pub enabled: bool,
}

pub fn latest_build(root: &Path) -> Option<u32> {
    let _ = root;
    todo!("leaf: game-detection")
}

pub fn client_version(root: &Path) -> Option<String> {
    let _ = root;
    todo!("leaf: game-detection")
}

pub fn replays_location(root: &Path, build: u32, client_version: Option<&str>) -> ReplaysLocation {
    let _ = (root, build, client_version);
    todo!("leaf: game-detection")
}

pub fn active_region(root: &Path) -> Option<Region> {
    let _ = root;
    todo!("leaf: game-detection")
}
