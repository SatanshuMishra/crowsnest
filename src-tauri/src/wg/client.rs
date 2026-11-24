use std::collections::HashMap;

use crate::model::{Region, Ship, Totals};

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum WgError {
    #[error("the Wargaming application ID was rejected")]
    InvalidApplicationId,
    #[error("Wargaming is rate limiting requests; try again shortly")]
    RateLimited,
    #[error("Wargaming returned an error: {code} {message}")]
    Api { code: u16, message: String },
    #[error("could not reach Wargaming: {0}")]
    Network(String),
    #[error("unexpected response from Wargaming: {0}")]
    Decode(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRecord {
    pub account_id: u64,
    pub hidden: bool,
    pub created_at: Option<i64>,
    pub pvp: Option<Totals>,
    pub pvp_solo: Option<Totals>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShipRecord {
    pub ship_id: u64,
    pub pvp: Totals,
    pub pvp_solo: Option<Totals>,
}

#[derive(Clone)]
pub struct WgClient {
    _private: (),
}

impl WgClient {
    pub fn new(http: reqwest::Client, region: Region, application_id: String) -> WgClient {
        let _ = (http, region, application_id);
        todo!("leaf: wg-api")
    }

    pub fn region(&self) -> Region {
        todo!("leaf: wg-api")
    }

    pub async fn verify_application_id(&self) -> Result<(), WgError> {
        todo!("leaf: wg-api")
    }

    pub async fn account_ids(&self, names: &[String]) -> Result<HashMap<String, u64>, WgError> {
        let _ = names;
        todo!("leaf: wg-api")
    }

    pub async fn accounts(&self, ids: &[u64]) -> Result<HashMap<u64, AccountRecord>, WgError> {
        let _ = ids;
        todo!("leaf: wg-api")
    }

    pub async fn ship_stats(&self, account_id: u64) -> Result<Vec<ShipRecord>, WgError> {
        let _ = account_id;
        todo!("leaf: wg-api")
    }

    pub async fn clan_tags(&self, ids: &[u64]) -> Result<HashMap<u64, String>, WgError> {
        let _ = ids;
        todo!("leaf: wg-api")
    }

    pub async fn ships(&self) -> Result<Vec<Ship>, WgError> {
        todo!("leaf: wg-api")
    }
}
