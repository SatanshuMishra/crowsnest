use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::body::{MAX_RESPONSE_BYTES, read_limited};
use crate::model::ExpectedStats;

const EXPECTED_VALUES_URL: &str = "https://api.wows-numbers.com/personal/rating/expected/json/";
const CACHE_FILE_NAME: &str = "expected-values.json";
const CACHE_MAX_AGE_SECONDS: u64 = 24 * 60 * 60;

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

#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    fetched_at: u64,
    body: String,
}

#[derive(Debug, Deserialize)]
struct PublishedEntry {
    average_damage_dealt: f64,
    average_frags: f64,
    win_rate: f64,
}

#[derive(Debug, Deserialize)]
struct PublishedFile {
    data: HashMap<String, serde_json::Value>,
}

pub fn parse_expected(body: &str) -> Result<HashMap<u64, Expected>, ExpectedError> {
    let published: PublishedFile =
        serde_json::from_str(body).map_err(|error| ExpectedError::Decode(error.to_string()))?;
    Ok(published
        .data
        .into_iter()
        .filter_map(|(key, value)| {
            let ship_id = key.parse::<u64>().ok()?;
            let entry = serde_json::from_value::<PublishedEntry>(value).ok()?;
            Some((
                ship_id,
                Expected {
                    damage: entry.average_damage_dealt,
                    frags: entry.average_frags,
                    win_rate: entry.win_rate,
                },
            ))
        })
        .collect())
}

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn is_fresh(fetched_at: u64, now: u64) -> bool {
    now.saturating_sub(fetched_at) < CACHE_MAX_AGE_SECONDS
}

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(CACHE_FILE_NAME)
}

fn read_cache(cache_dir: &Path) -> Option<(u64, ExpectedValues)> {
    let text = std::fs::read_to_string(cache_path(cache_dir)).ok()?;
    let file: CacheFile = serde_json::from_str(&text).ok()?;
    let ships = parse_expected(&file.body).ok()?;
    Some((file.fetched_at, ExpectedValues { ships }))
}

fn write_cache(cache_dir: &Path, body: &str, fetched_at: u64) -> std::io::Result<()> {
    std::fs::create_dir_all(cache_dir)?;
    let file = CacheFile {
        fetched_at,
        body: body.to_string(),
    };
    let text = serde_json::to_string(&file).map_err(std::io::Error::other)?;
    let target = cache_path(cache_dir);
    let temporary = target.with_extension("json.tmp");
    std::fs::write(&temporary, text)?;
    std::fs::rename(&temporary, &target)
}

fn fall_back_to_cache(
    cached: Option<ExpectedValues>,
    error: ExpectedError,
) -> Result<ExpectedValues, ExpectedError> {
    match cached {
        Some(values) => {
            log::warn!("using cached expected values after: {error}");
            Ok(values)
        }
        None => Err(error),
    }
}

async fn download(http: &reqwest::Client) -> Result<String, ExpectedError> {
    let response = http
        .get(EXPECTED_VALUES_URL)
        .send()
        .await
        .and_then(|response| response.error_for_status())
        .map_err(|error| ExpectedError::Network(error.to_string()))?;
    let body = read_limited(response, MAX_RESPONSE_BYTES)
        .await
        .map_err(|error| ExpectedError::Network(error.to_string()))?;
    String::from_utf8(body).map_err(|error| ExpectedError::Decode(error.to_string()))
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
        let cached = read_cache(cache_dir);
        let now = now_seconds();
        let stale = match cached {
            Some((fetched_at, values)) if is_fresh(fetched_at, now) => return Ok(values),
            Some((_, values)) => Some(values),
            None => None,
        };
        let fetched = download(http).await.and_then(|body| {
            let ships = parse_expected(&body)?;
            Ok((body, ExpectedValues { ships }))
        });
        match fetched {
            Ok((body, values)) => {
                if let Err(error) = write_cache(cache_dir, &body, now) {
                    log::warn!("could not write expected values cache: {error}");
                }
                Ok(values)
            }
            Err(error) => fall_back_to_cache(stale, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/expected.json");

    #[test]
    fn parses_entries_and_skips_empty_arrays() {
        let ships = parse_expected(FIXTURE).unwrap();
        assert_eq!(ships.len(), 3);
        assert!(!ships.contains_key(&3000));
        assert_eq!(
            ships[&2000],
            Expected {
                damage: 50000.0,
                frags: 0.5,
                win_rate: 40.0
            }
        );
    }

    #[test]
    fn skips_entries_missing_a_field() {
        let body = r#"{"time":1,"data":{"1":{"average_frags":1.0,"win_rate":50.0},"2":{"average_damage_dealt":1.0,"average_frags":1.0,"win_rate":50.0}}}"#;
        let ships = parse_expected(body).unwrap();
        assert_eq!(ships.len(), 1);
        assert!(ships.contains_key(&2));
    }

    #[test]
    fn rejects_malformed_body() {
        assert!(matches!(
            parse_expected("not json"),
            Err(ExpectedError::Decode(_))
        ));
    }

    #[test]
    fn freshness_uses_the_stored_fetch_time() {
        assert!(is_fresh(1_000, 1_000 + CACHE_MAX_AGE_SECONDS - 1));
        assert!(!is_fresh(1_000, 1_000 + CACHE_MAX_AGE_SECONDS));
    }

    #[tokio::test]
    async fn fresh_cache_is_returned_without_downloading() {
        let dir = tempfile::tempdir().unwrap();
        write_cache(dir.path(), FIXTURE, now_seconds()).unwrap();
        let values = ExpectedValues::load(dir.path(), &reqwest::Client::new())
            .await
            .unwrap();
        assert_eq!(values.ships.len(), 3);
    }

    #[test]
    fn written_cache_round_trips_with_fetch_time() {
        let dir = tempfile::tempdir().unwrap();
        write_cache(dir.path(), FIXTURE, 42).unwrap();
        let (fetched_at, values) = read_cache(dir.path()).unwrap();
        assert_eq!(fetched_at, 42);
        assert_eq!(values.ships.len(), 3);
        assert!(!dir.path().join("expected-values.json.tmp").exists());
    }

    #[test]
    fn stale_cache_is_used_when_download_fails() {
        let dir = tempfile::tempdir().unwrap();
        write_cache(dir.path(), FIXTURE, 1).unwrap();
        let (_, cached) = read_cache(dir.path()).unwrap();
        let values =
            fall_back_to_cache(Some(cached), ExpectedError::Network("offline".into())).unwrap();
        assert_eq!(values.ships.len(), 3);
    }

    #[test]
    fn error_is_returned_when_nothing_is_cached() {
        let result = fall_back_to_cache(None, ExpectedError::Network("offline".into()));
        assert!(matches!(result, Err(ExpectedError::Network(_))));
    }
}
