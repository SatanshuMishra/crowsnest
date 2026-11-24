use std::collections::HashMap;
use std::path::Path;

use crate::model::Ship;
use crate::wg::client::{WgClient, WgError};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShipCatalog {
    ships: HashMap<u64, Ship>,
}

impl ShipCatalog {
    pub fn from_ships(ships: Vec<Ship>) -> ShipCatalog {
        ShipCatalog {
            ships: ships.into_iter().map(|ship| (ship.id, ship)).collect(),
        }
    }

    pub fn get(&self, ship_id: u64) -> Option<&Ship> {
        self.ships.get(&ship_id)
    }

    pub fn len(&self) -> usize {
        self.ships.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ships.is_empty()
    }

    pub async fn load(
        cache_dir: &Path,
        client: &WgClient,
        game_version: &str,
        required: &[u64],
    ) -> Result<ShipCatalog, WgError> {
        let _ = (cache_dir, client, game_version, required);
        todo!("leaf: wg-api")
    }
}
