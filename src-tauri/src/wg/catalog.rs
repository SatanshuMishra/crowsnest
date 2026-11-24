use std::collections::HashMap;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{Region, Ship};
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
        load_or_fetch(
            cache_path(cache_dir, client.region()),
            game_version,
            required,
            || client.ships(),
        )
        .await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedCatalog {
    game_version: String,
    ships: Vec<Ship>,
}

impl CachedCatalog {
    fn is_fresh(&self, game_version: &str, required: &[u64]) -> bool {
        self.game_version == game_version
            && required
                .iter()
                .all(|ship_id| self.ships.iter().any(|ship| ship.id == *ship_id))
    }
}

async fn load_or_fetch<F, Fut>(
    path: PathBuf,
    game_version: &str,
    required: &[u64],
    fetch: F,
) -> Result<ShipCatalog, WgError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<Vec<Ship>, WgError>>,
{
    match read_cache_off_thread(path.clone()).await {
        Some(cached) if cached.is_fresh(game_version, required) => {
            Ok(ShipCatalog::from_ships(cached.ships))
        }
        stale => match fetch().await {
            Ok(ships) => {
                let refreshed = CachedCatalog {
                    game_version: game_version.to_owned(),
                    ships,
                };
                if let Err(error) = write_cache_off_thread(path, refreshed.clone()).await {
                    log::warn!("could not save the ship catalog cache: {error}");
                }
                let catalog = ShipCatalog::from_ships(refreshed.ships);
                log::info!(
                    "ship catalog refreshed for game version {game_version} with {} ships",
                    catalog.len()
                );
                Ok(catalog)
            }
            Err(error) => match stale {
                Some(cached) => {
                    log::warn!("ship catalog refresh failed, using the cached copy: {error}");
                    Ok(ShipCatalog::from_ships(cached.ships))
                }
                None => Err(error),
            },
        },
    }
}

fn cache_path(cache_dir: &Path, region: Region) -> PathBuf {
    cache_dir.join(format!("ships-{}.json", region_slug(region)))
}

fn region_slug(region: Region) -> &'static str {
    match region {
        Region::Eu => "eu",
        Region::Na => "na",
        Region::Asia => "asia",
    }
}

fn staging_path(path: &Path) -> PathBuf {
    path.with_extension("json.tmp")
}

async fn read_cache_off_thread(path: PathBuf) -> Option<CachedCatalog> {
    tokio::task::spawn_blocking(move || read_cache(&path))
        .await
        .ok()
        .flatten()
}

async fn write_cache_off_thread(path: PathBuf, catalog: CachedCatalog) -> io::Result<()> {
    tokio::task::spawn_blocking(move || write_cache(&path, &catalog))
        .await
        .map_err(io::Error::other)?
}

fn read_cache(path: &Path) -> Option<CachedCatalog> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes)
        .inspect_err(|error| log::warn!("ignoring an unreadable ship catalog cache: {error}"))
        .ok()
}

fn write_cache(path: &Path, catalog: &CachedCatalog) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let staging = staging_path(path);
    let json = serde_json::to_vec(catalog).map_err(io::Error::other)?;
    std::fs::write(&staging, json)?;
    std::fs::rename(&staging, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&staging);
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde_json::json;

    use super::*;
    use crate::model::ShipClass;

    const CURRENT_VERSION: &str = "15.10.0.0.13533378";
    const PREVIOUS_VERSION: &str = "15.9.0.0.13400000";

    fn ship(id: u64, name: &str) -> Ship {
        Ship {
            id,
            name: name.to_owned(),
            tier: 10,
            class: ShipClass::Cruiser,
            nation: "usa".to_owned(),
            is_premium: false,
            is_special: false,
        }
    }

    fn write_cache_file(path: &Path, game_version: &str, ships: &[(u64, &str)]) {
        let entries: Vec<serde_json::Value> = ships
            .iter()
            .map(|(id, name)| {
                json!({
                    "id": id,
                    "name": name,
                    "tier": 10,
                    "class": "cruiser",
                    "nation": "usa",
                    "isPremium": false,
                    "isSpecial": false
                })
            })
            .collect();
        let document = json!({ "gameVersion": game_version, "ships": entries });
        std::fs::write(path, document.to_string()).expect("cache file written");
    }

    fn cache_file_in(dir: &tempfile::TempDir) -> PathBuf {
        cache_path(dir.path(), Region::Eu)
    }

    fn names(catalog: &ShipCatalog, ids: &[u64]) -> Vec<Option<String>> {
        ids.iter()
            .map(|id| catalog.get(*id).map(|ship| ship.name.clone()))
            .collect()
    }

    #[test]
    fn cache_file_is_named_per_region() {
        let dir = Path::new("cache");
        assert_eq!(cache_path(dir, Region::Eu), dir.join("ships-eu.json"));
        assert_eq!(cache_path(dir, Region::Na), dir.join("ships-na.json"));
        assert_eq!(cache_path(dir, Region::Asia), dir.join("ships-asia.json"));
    }

    #[tokio::test]
    async fn fresh_cache_is_used_without_fetching() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = cache_file_in(&dir);
        write_cache_file(
            &path,
            CURRENT_VERSION,
            &[(1, "Cached One"), (2, "Cached Two")],
        );
        let fetched = Cell::new(false);
        let catalog = load_or_fetch(path, CURRENT_VERSION, &[1, 2], || {
            fetched.set(true);
            async { Ok(vec![ship(1, "Fetched One")]) }
        })
        .await
        .expect("catalog");
        assert!(!fetched.get());
        assert_eq!(catalog.len(), 2);
        assert_eq!(
            names(&catalog, &[1, 2]),
            vec![Some("Cached One".to_owned()), Some("Cached Two".to_owned())]
        );
    }

    #[tokio::test]
    async fn new_game_version_refetches_and_rewrites_the_cache() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = cache_file_in(&dir);
        write_cache_file(&path, PREVIOUS_VERSION, &[(1, "Cached One")]);
        let catalog = load_or_fetch(path.clone(), CURRENT_VERSION, &[1], || async {
            Ok(vec![ship(1, "Fetched One"), ship(3, "Fetched Three")])
        })
        .await
        .expect("catalog");
        assert_eq!(
            names(&catalog, &[1, 3]),
            vec![
                Some("Fetched One".to_owned()),
                Some("Fetched Three".to_owned())
            ]
        );
        let stored = read_cache(&path).expect("rewritten cache");
        assert_eq!(stored.game_version, CURRENT_VERSION);
        assert_eq!(stored.ships.len(), 2);
        assert!(!staging_path(&path).exists());
    }

    #[tokio::test]
    async fn missing_required_ship_refetches() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = cache_file_in(&dir);
        write_cache_file(&path, CURRENT_VERSION, &[(1, "Cached One")]);
        let catalog = load_or_fetch(path, CURRENT_VERSION, &[1, 9], || async {
            Ok(vec![ship(1, "Fetched One"), ship(9, "Fetched Nine")])
        })
        .await
        .expect("catalog");
        assert_eq!(names(&catalog, &[9]), vec![Some("Fetched Nine".to_owned())]);
    }

    #[tokio::test]
    async fn failed_fetch_falls_back_to_the_older_cache() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = cache_file_in(&dir);
        write_cache_file(&path, PREVIOUS_VERSION, &[(1, "Cached One")]);
        let catalog = load_or_fetch(path.clone(), CURRENT_VERSION, &[1, 9], || async {
            Err(WgError::Network("offline".to_owned()))
        })
        .await
        .expect("older catalog");
        assert_eq!(names(&catalog, &[1]), vec![Some("Cached One".to_owned())]);
        let untouched = read_cache(&path).expect("older cache kept");
        assert_eq!(untouched.game_version, PREVIOUS_VERSION);
    }

    #[tokio::test]
    async fn failed_fetch_without_a_cache_reports_the_error() {
        let dir = tempfile::tempdir().expect("temp dir");
        let outcome = load_or_fetch(cache_file_in(&dir), CURRENT_VERSION, &[1], || async {
            Err(WgError::RateLimited)
        })
        .await;
        assert_eq!(outcome, Err(WgError::RateLimited));
    }

    #[tokio::test]
    async fn first_fetch_creates_the_cache_directory_and_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = cache_path(&dir.path().join("nested"), Region::Na);
        let catalog = load_or_fetch(path.clone(), CURRENT_VERSION, &[], || async {
            Ok(vec![ship(5, "Fetched Five")])
        })
        .await
        .expect("catalog");
        assert_eq!(catalog.len(), 1);
        assert_eq!(
            read_cache(&path),
            Some(CachedCatalog {
                game_version: CURRENT_VERSION.to_owned(),
                ships: vec![ship(5, "Fetched Five")],
            })
        );
    }

    #[tokio::test]
    async fn unreadable_cache_is_treated_as_absent() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = cache_file_in(&dir);
        std::fs::write(&path, "{ not json").expect("corrupt cache written");
        let catalog = load_or_fetch(path, CURRENT_VERSION, &[], || async {
            Ok(vec![ship(1, "Fetched One")])
        })
        .await
        .expect("catalog");
        assert_eq!(names(&catalog, &[1]), vec![Some("Fetched One".to_owned())]);
    }

    #[tokio::test]
    async fn public_load_reads_the_region_cache_when_the_client_cannot_fetch() {
        let dir = tempfile::tempdir().expect("temp dir");
        let client = WgClient::new(reqwest::Client::new(), Region::Eu, String::new());
        let without_cache = ShipCatalog::load(dir.path(), &client, CURRENT_VERSION, &[1]).await;
        assert_eq!(without_cache, Err(WgError::InvalidApplicationId));
        write_cache_file(&cache_file_in(&dir), PREVIOUS_VERSION, &[(1, "Cached One")]);
        let with_cache = ShipCatalog::load(dir.path(), &client, CURRENT_VERSION, &[1])
            .await
            .expect("older catalog");
        assert_eq!(
            names(&with_cache, &[1]),
            vec![Some("Cached One".to_owned())]
        );
    }
}
