use std::collections::HashMap;
use std::path::Path;

use crate::model::ExpectedStats;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Expected {
    pub damage: f64,
    pub frags: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExpectedValues {
    pub ships: HashMap<u64, Expected>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExpectedError {
    #[error("could not download expected values: {0}")]
    Network(String),
    #[error("expected values were not in the format wows-numbers publishes: {0}")]
    Decode(String),
}

impl ExpectedValues {
    pub fn for_ship(&self, ship_id: u64) -> Option<ExpectedStats> {
        self.ships.get(&ship_id).map(|expected| ExpectedStats {
            damage: expected.damage,
            frags: expected.frags,
            win_rate: expected.win_rate,
        })
    }

    pub async fn load(
        cache_dir: &Path,
        http: &reqwest::Client,
    ) -> Result<ExpectedValues, ExpectedError> {
        let _ = (cache_dir, http);
        todo!("leaf: ratings")
    }
}
