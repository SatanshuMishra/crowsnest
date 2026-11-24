use std::collections::{BTreeMap, HashMap, VecDeque};
use std::future::Future;
use std::hash::Hash;
use std::iter;
use std::sync::Arc;
use std::time::Duration;

use futures::future::try_join_all;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::model::{Region, Ship, ShipClass, Totals};

const BATCH_SIZE: usize = 100;
const SHIPS_PAGE_SIZE: usize = 100;
const MAX_STARTS_PER_WINDOW: usize = 8;
const RATE_WINDOW: Duration = Duration::from_secs(1);
const RATE_LIMIT_RETRY_DELAY: Duration = Duration::from_secs(1);
const TOTALS_FIELDS: [&str; 6] = [
    "battles",
    "wins",
    "survived_battles",
    "damage_dealt",
    "frags",
    "xp",
];
const SHIP_FIELDS: &str = "ship_id,name,tier,type,nation,is_premium,is_special";

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
    http: reqwest::Client,
    region: Region,
    application_id: Arc<str>,
    limiter: Arc<RateLimiter>,
}

impl WgClient {
    pub fn new(http: reqwest::Client, region: Region, application_id: String) -> WgClient {
        WgClient {
            http,
            region,
            application_id: Arc::from(application_id),
            limiter: Arc::new(RateLimiter::new(MAX_STARTS_PER_WINDOW, RATE_WINDOW)),
        }
    }

    pub fn region(&self) -> Region {
        self.region
    }

    pub async fn verify_application_id(&self) -> Result<(), WgError> {
        self.call(&game_version_request()).await.map(|_| ())
    }

    pub async fn account_ids(&self, names: &[String]) -> Result<HashMap<String, u64>, WgError> {
        self.gather(account_list_requests(names), parse_account_ids)
            .await
    }

    pub async fn accounts(&self, ids: &[u64]) -> Result<HashMap<u64, AccountRecord>, WgError> {
        self.gather(account_info_requests(ids), parse_accounts)
            .await
    }

    pub async fn ship_stats(&self, account_id: u64) -> Result<Vec<ShipRecord>, WgError> {
        let payload = self.call(&ship_stats_request(account_id)).await?;
        parse_ship_stats(payload, account_id)
    }

    pub async fn clan_tags(&self, ids: &[u64]) -> Result<HashMap<u64, String>, WgError> {
        self.gather(clan_info_requests(ids), parse_clan_tags).await
    }

    pub async fn ships(&self) -> Result<Vec<Ship>, WgError> {
        let first = self.ships_page(1).await?;
        let rest =
            try_join_all((2..=first.page_total).map(|page_no| self.ships_page(page_no))).await?;
        Ok(sorted_by_id(
            iter::once(first).chain(rest).flat_map(|page| page.ships),
        ))
    }

    async fn ships_page(&self, page_no: u32) -> Result<ShipsPage, WgError> {
        parse_ships_page(self.call(&encyclopedia_ships_request(page_no)).await?)
    }

    async fn gather<K: Eq + Hash, V>(
        &self,
        requests: Vec<ApiRequest>,
        parse: fn(Payload) -> Result<HashMap<K, V>, WgError>,
    ) -> Result<HashMap<K, V>, WgError> {
        let parts = try_join_all(
            requests
                .iter()
                .map(|request| async move { parse(self.call(request).await?) }),
        )
        .await?;
        Ok(parts.into_iter().flatten().collect())
    }

    async fn call(&self, request: &ApiRequest) -> Result<Payload, WgError> {
        if self.application_id.trim().is_empty() {
            return Err(WgError::InvalidApplicationId);
        }
        retry_once_when_rate_limited(RATE_LIMIT_RETRY_DELAY, || self.attempt(request)).await
    }

    async fn attempt(&self, request: &ApiRequest) -> Result<Payload, WgError> {
        let url = request_url(self.region, &self.application_id, request)?;
        self.limiter.acquire().await;
        let response = self.http.get(url).send().await.map_err(transport_failure)?;
        let status = response.status();
        if !status.is_success() {
            return Err(WgError::Network(format!(
                "Wargaming answered with HTTP status {}",
                status.as_u16()
            )));
        }
        let body = response.bytes().await.map_err(transport_failure)?;
        decode_envelope(&body)
    }
}

fn transport_failure(error: reqwest::Error) -> WgError {
    let description = if error.is_timeout() {
        "the request timed out".to_owned()
    } else if error.is_connect() {
        "the connection could not be established".to_owned()
    } else {
        error.without_url().to_string()
    };
    WgError::Network(description)
}

async fn retry_once_when_rate_limited<T, F, Fut>(delay: Duration, attempt: F) -> Result<T, WgError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, WgError>>,
{
    match attempt().await {
        Err(WgError::RateLimited) => {
            log::debug!("Wargaming rate limited a request; retrying once");
            tokio::time::sleep(delay).await;
            attempt().await
        }
        outcome => outcome,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StartLog {
    capacity: usize,
    window: Duration,
    recent: VecDeque<Instant>,
}

impl StartLog {
    fn new(capacity: usize, window: Duration) -> StartLog {
        StartLog {
            capacity: capacity.max(1),
            window,
            recent: VecDeque::new(),
        }
    }

    fn earliest_start(&self, now: Instant) -> Instant {
        match self.recent.front() {
            Some(oldest) if self.recent.len() >= self.capacity => now.max(*oldest + self.window),
            _ => now,
        }
    }

    fn with_start(&self, start: Instant) -> StartLog {
        let overflow = (self.recent.len() + 1).saturating_sub(self.capacity);
        StartLog {
            capacity: self.capacity,
            window: self.window,
            recent: self
                .recent
                .iter()
                .copied()
                .chain(iter::once(start))
                .skip(overflow)
                .collect(),
        }
    }
}

struct RateLimiter {
    log: Mutex<StartLog>,
}

impl RateLimiter {
    fn new(capacity: usize, window: Duration) -> RateLimiter {
        RateLimiter {
            log: Mutex::new(StartLog::new(capacity, window)),
        }
    }

    async fn acquire(&self) {
        let mut log = self.log.lock().await;
        tokio::time::sleep_until(log.earliest_start(Instant::now())).await;
        *log = log.with_start(Instant::now());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApiRequest {
    section: &'static str,
    method: &'static str,
    params: Vec<(&'static str, String)>,
}

impl ApiRequest {
    fn new(
        section: &'static str,
        method: &'static str,
        params: Vec<(&'static str, String)>,
    ) -> ApiRequest {
        ApiRequest {
            section,
            method,
            params,
        }
    }
}

fn request_url(
    region: Region,
    application_id: &str,
    request: &ApiRequest,
) -> Result<reqwest::Url, WgError> {
    let base = format!(
        "https://{}/wows/{}/{}/",
        region.api_host(),
        request.section,
        request.method
    );
    let params = iter::once(("application_id", application_id)).chain(
        request
            .params
            .iter()
            .map(|(name, value)| (*name, value.as_str())),
    );
    reqwest::Url::parse_with_params(&base, params)
        .map_err(|error| WgError::Network(format!("invalid request address: {error}")))
}

fn game_version_request() -> ApiRequest {
    ApiRequest::new(
        "encyclopedia",
        "info",
        vec![("fields", "game_version".to_owned())],
    )
}

fn account_list_requests(names: &[String]) -> Vec<ApiRequest> {
    names
        .chunks(BATCH_SIZE)
        .map(|chunk| {
            ApiRequest::new(
                "account",
                "list",
                vec![
                    ("search", chunk.join(",")),
                    ("type", "exact".to_owned()),
                    ("limit", BATCH_SIZE.to_string()),
                    ("fields", "nickname,account_id".to_owned()),
                ],
            )
        })
        .collect()
}

fn account_info_requests(ids: &[u64]) -> Vec<ApiRequest> {
    ids.chunks(BATCH_SIZE)
        .map(|chunk| {
            ApiRequest::new(
                "account",
                "info",
                vec![
                    ("account_id", joined(chunk)),
                    ("fields", account_info_fields()),
                    ("extra", "statistics.pvp_solo".to_owned()),
                ],
            )
        })
        .collect()
}

fn ship_stats_request(account_id: u64) -> ApiRequest {
    ApiRequest::new(
        "ships",
        "stats",
        vec![
            ("account_id", account_id.to_string()),
            ("fields", ship_stats_fields()),
            ("extra", "pvp_solo".to_owned()),
        ],
    )
}

fn clan_info_requests(ids: &[u64]) -> Vec<ApiRequest> {
    ids.chunks(BATCH_SIZE)
        .map(|chunk| {
            ApiRequest::new(
                "clans",
                "accountinfo",
                vec![("account_id", joined(chunk)), ("extra", "clan".to_owned())],
            )
        })
        .collect()
}

fn encyclopedia_ships_request(page_no: u32) -> ApiRequest {
    ApiRequest::new(
        "encyclopedia",
        "ships",
        vec![
            ("fields", SHIP_FIELDS.to_owned()),
            ("limit", SHIPS_PAGE_SIZE.to_string()),
            ("page_no", page_no.to_string()),
        ],
    )
}

fn joined(ids: &[u64]) -> String {
    ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}

fn totals_fields(prefix: &str) -> impl Iterator<Item = String> {
    TOTALS_FIELDS
        .iter()
        .map(move |field| format!("{prefix}.{field}"))
}

fn account_info_fields() -> String {
    ["account_id", "hidden_profile", "created_at"]
        .into_iter()
        .map(str::to_owned)
        .chain(totals_fields("statistics.pvp"))
        .chain(totals_fields("statistics.pvp_solo"))
        .collect::<Vec<_>>()
        .join(",")
}

fn ship_stats_fields() -> String {
    iter::once("ship_id".to_owned())
        .chain(totals_fields("pvp"))
        .chain(totals_fields("pvp_solo"))
        .collect::<Vec<_>>()
        .join(",")
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
struct Meta {
    page_total: Option<u32>,
    hidden: Option<Vec<u64>>,
}

impl Meta {
    fn lists_hidden(&self, account_id: u64) -> bool {
        self.hidden
            .as_ref()
            .is_some_and(|hidden| hidden.contains(&account_id))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Payload {
    meta: Meta,
    data: Value,
}

#[derive(Deserialize)]
struct Envelope {
    status: String,
    meta: Option<Meta>,
    #[serde(default)]
    data: Value,
    error: Option<ApiErrorBody>,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    code: Option<u16>,
    message: Option<String>,
}

fn decode_envelope(body: &[u8]) -> Result<Payload, WgError> {
    let envelope: Envelope = serde_json::from_slice(body).map_err(decode_failure)?;
    match envelope.status.as_str() {
        "ok" => Ok(Payload {
            meta: envelope.meta.unwrap_or_default(),
            data: envelope.data,
        }),
        "error" => Err(envelope.error.map_or_else(
            || WgError::Decode("error response without details".to_owned()),
            api_error,
        )),
        other => Err(WgError::Decode(format!("unknown response status {other}"))),
    }
}

fn api_error(error: ApiErrorBody) -> WgError {
    let message = error.message.unwrap_or_default();
    match message.as_str() {
        "INVALID_APPLICATION_ID" | "APPLICATION_ID_NOT_SPECIFIED" => WgError::InvalidApplicationId,
        "REQUEST_LIMIT_EXCEEDED" => WgError::RateLimited,
        _ => WgError::Api {
            code: error.code.unwrap_or_default(),
            message,
        },
    }
}

fn decode_failure(error: serde_json::Error) -> WgError {
    WgError::Decode(error.to_string())
}

fn data_as<T: DeserializeOwned>(data: Value) -> Result<T, WgError> {
    serde_json::from_value(data).map_err(decode_failure)
}

#[derive(Deserialize)]
struct RawTotals {
    battles: Option<u64>,
    wins: Option<u64>,
    survived_battles: Option<u64>,
    damage_dealt: Option<u64>,
    frags: Option<u64>,
    xp: Option<u64>,
}

impl From<RawTotals> for Totals {
    fn from(raw: RawTotals) -> Totals {
        Totals {
            battles: raw.battles.unwrap_or_default(),
            wins: raw.wins.unwrap_or_default(),
            survived: raw.survived_battles.unwrap_or_default(),
            damage: raw.damage_dealt.unwrap_or_default(),
            frags: raw.frags.unwrap_or_default(),
            xp: raw.xp.unwrap_or_default(),
        }
    }
}

#[derive(Deserialize)]
struct RawListedAccount {
    nickname: String,
    account_id: u64,
}

#[derive(Deserialize)]
struct RawAccount {
    hidden_profile: Option<bool>,
    created_at: Option<i64>,
    statistics: Option<RawAccountStatistics>,
}

#[derive(Deserialize)]
struct RawAccountStatistics {
    pvp: Option<RawTotals>,
    pvp_solo: Option<RawTotals>,
}

#[derive(Deserialize)]
struct RawShipStats {
    ship_id: u64,
    pvp: Option<RawTotals>,
    pvp_solo: Option<RawTotals>,
}

#[derive(Deserialize)]
struct RawClanMembership {
    clan: Option<RawClan>,
}

#[derive(Deserialize)]
struct RawClan {
    tag: Option<String>,
}

#[derive(Deserialize)]
struct RawShip {
    name: String,
    tier: u8,
    #[serde(rename = "type")]
    class: Option<String>,
    nation: Option<String>,
    is_premium: Option<bool>,
    is_special: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShipsPage {
    ships: Vec<Ship>,
    page_total: u32,
}

fn parse_account_ids(payload: Payload) -> Result<HashMap<String, u64>, WgError> {
    let listed: Option<Vec<RawListedAccount>> = data_as(payload.data)?;
    Ok(listed
        .unwrap_or_default()
        .into_iter()
        .map(|account| (account.nickname, account.account_id))
        .collect())
}

fn parse_accounts(payload: Payload) -> Result<HashMap<u64, AccountRecord>, WgError> {
    let entries: Option<HashMap<u64, Option<RawAccount>>> = data_as(payload.data)?;
    Ok(entries
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(account_id, raw)| {
            raw.map(|raw| {
                let listed_hidden = payload.meta.lists_hidden(account_id);
                (account_id, account_record(account_id, raw, listed_hidden))
            })
        })
        .collect())
}

fn account_record(account_id: u64, raw: RawAccount, listed_hidden: bool) -> AccountRecord {
    let (pvp, pvp_solo) = raw.statistics.map_or((None, None), |statistics| {
        (
            statistics.pvp.map(Totals::from),
            statistics.pvp_solo.map(Totals::from),
        )
    });
    AccountRecord {
        account_id,
        hidden: raw.hidden_profile.unwrap_or_default() || listed_hidden,
        created_at: raw.created_at,
        pvp,
        pvp_solo,
    }
}

fn parse_ship_stats(payload: Payload, account_id: u64) -> Result<Vec<ShipRecord>, WgError> {
    if payload.meta.lists_hidden(account_id) {
        return Ok(Vec::new());
    }
    let entries: Option<HashMap<u64, Option<Vec<RawShipStats>>>> = data_as(payload.data)?;
    Ok(entries
        .unwrap_or_default()
        .into_iter()
        .find_map(|(id, ships)| (id == account_id).then_some(ships))
        .flatten()
        .unwrap_or_default()
        .into_iter()
        .map(ship_record)
        .collect())
}

fn ship_record(raw: RawShipStats) -> ShipRecord {
    ShipRecord {
        ship_id: raw.ship_id,
        pvp: raw.pvp.map(Totals::from).unwrap_or_default(),
        pvp_solo: raw.pvp_solo.map(Totals::from),
    }
}

fn parse_clan_tags(payload: Payload) -> Result<HashMap<u64, String>, WgError> {
    let entries: Option<HashMap<u64, Option<RawClanMembership>>> = data_as(payload.data)?;
    Ok(entries
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(account_id, membership)| {
            let tag = membership?.clan?.tag?;
            (!tag.is_empty()).then_some((account_id, tag))
        })
        .collect())
}

fn parse_ships_page(payload: Payload) -> Result<ShipsPage, WgError> {
    let entries: Option<HashMap<u64, Option<RawShip>>> = data_as(payload.data)?;
    let ships = entries
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(ship_id, raw)| raw.map(|raw| ship(ship_id, raw)));
    Ok(ShipsPage {
        ships: sorted_by_id(ships),
        page_total: payload.meta.page_total.unwrap_or(1),
    })
}

fn ship(ship_id: u64, raw: RawShip) -> Ship {
    Ship {
        id: ship_id,
        name: raw.name,
        tier: raw.tier,
        class: ShipClass::from_api(raw.class.as_deref().unwrap_or_default()),
        nation: raw.nation.unwrap_or_default(),
        is_premium: raw.is_premium.unwrap_or_default(),
        is_special: raw.is_special.unwrap_or_default(),
    }
}

fn sorted_by_id(ships: impl Iterator<Item = Ship>) -> Vec<Ship> {
    ships
        .map(|ship| (ship.id, ship))
        .collect::<BTreeMap<_, _>>()
        .into_values()
        .collect()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    const APPLICATION_ID: &str = "test-application-id";
    const ACCOUNT_LIST: &str = include_str!("fixtures/account_list.json");
    const ACCOUNT_INFO: &str = include_str!("fixtures/account_info.json");
    const SHIPS_STATS: &str = include_str!("fixtures/ships_stats.json");
    const CLANS_ACCOUNTINFO: &str = include_str!("fixtures/clans_accountinfo.json");
    const ENCYCLOPEDIA_SHIPS: &str = include_str!("fixtures/encyclopedia_ships.json");

    fn payload(body: &str) -> Payload {
        decode_envelope(body.as_bytes()).expect("an ok envelope")
    }

    fn error_body(code: u16, message: &str) -> String {
        format!(
            r#"{{"status":"error","error":{{"field":null,"message":"{message}","code":{code},"value":null}}}}"#
        )
    }

    fn param<'a>(request: &'a ApiRequest, name: &str) -> Option<&'a str> {
        request
            .params
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
    }

    fn query(url: &reqwest::Url) -> HashMap<String, String> {
        url.query_pairs().into_owned().collect()
    }

    fn nicknames(count: usize) -> Vec<String> {
        (0..count)
            .map(|index| format!("Deckhand_{index:03}"))
            .collect()
    }

    fn account_ids(count: u64) -> Vec<u64> {
        (0..count).map(|index| 500_300_000 + index).collect()
    }

    fn client(application_id: &str) -> WgClient {
        WgClient::new(
            reqwest::Client::new(),
            Region::Eu,
            application_id.to_owned(),
        )
    }

    fn totals(battles: u64, wins: u64, survived: u64, damage: u64, frags: u64, xp: u64) -> Totals {
        Totals {
            battles,
            wins,
            survived,
            damage,
            frags,
            xp,
        }
    }

    #[test]
    fn ok_envelope_carries_meta_and_data() {
        let decoded = payload(ENCYCLOPEDIA_SHIPS);
        assert_eq!(decoded.meta.page_total, Some(3));
        assert_eq!(decoded.meta.hidden, None);
        assert!(decoded.data.is_object());
    }

    #[test]
    fn application_id_errors_map_to_invalid_application_id() {
        for (code, message) in [
            (407, "INVALID_APPLICATION_ID"),
            (402, "APPLICATION_ID_NOT_SPECIFIED"),
        ] {
            assert_eq!(
                decode_envelope(error_body(code, message).as_bytes()),
                Err(WgError::InvalidApplicationId)
            );
        }
    }

    #[test]
    fn request_limit_maps_to_rate_limited() {
        assert_eq!(
            decode_envelope(error_body(407, "REQUEST_LIMIT_EXCEEDED").as_bytes()),
            Err(WgError::RateLimited)
        );
    }

    #[test]
    fn other_api_errors_keep_their_code_and_message() {
        assert_eq!(
            decode_envelope(error_body(407, "INVALID_SEARCH").as_bytes()),
            Err(WgError::Api {
                code: 407,
                message: "INVALID_SEARCH".to_owned(),
            })
        );
    }

    #[test]
    fn malformed_envelopes_map_to_decode() {
        for body in [
            "not json",
            r#"{"data":{}}"#,
            r#"{"status":"pending","data":{}}"#,
            r#"{"status":"error"}"#,
        ] {
            assert!(
                matches!(decode_envelope(body.as_bytes()), Err(WgError::Decode(_))),
                "{body}"
            );
        }
    }

    #[test]
    fn unexpected_data_shapes_map_to_decode() {
        let wrong_account = payload(
            r#"{"status":"ok","meta":{"count":1},"data":{"7":{"hidden_profile":"sometimes"}}}"#,
        );
        assert!(matches!(
            parse_accounts(wrong_account),
            Err(WgError::Decode(_))
        ));
        let wrong_list = payload(r#"{"status":"ok","data":{"nickname":1}}"#);
        assert!(matches!(
            parse_account_ids(wrong_list),
            Err(WgError::Decode(_))
        ));
    }

    #[test]
    fn account_ids_keep_the_nickname_exactly_as_returned() {
        let ids = parse_account_ids(payload(ACCOUNT_LIST)).expect("account ids");
        assert_eq!(ids.len(), 3);
        assert_eq!(ids.get("Harbor_Pilot"), Some(&500_100_001));
        assert_eq!(ids.get("TideWatcher_77"), Some(&500_100_003));
        assert!(!ids.contains_key("harbor_pilot"));
    }

    #[test]
    fn empty_account_search_yields_no_ids() {
        let none = payload(r#"{"status":"ok","meta":{"count":0},"data":[]}"#);
        assert_eq!(parse_account_ids(none), Ok(HashMap::new()));
    }

    #[test]
    fn accounts_map_visible_hidden_and_missing_players() {
        let accounts = parse_accounts(payload(ACCOUNT_INFO)).expect("accounts");
        assert_eq!(accounts.len(), 3);
        assert_eq!(
            accounts[&500_100_001],
            AccountRecord {
                account_id: 500_100_001,
                hidden: false,
                created_at: Some(1_420_070_400),
                pvp: Some(totals(4210, 2315, 1580, 194_502_300, 4630, 5_262_500)),
                pvp_solo: Some(totals(3100, 1674, 1120, 140_120_400, 3350, 3_844_000)),
            }
        );
        assert_eq!(
            accounts[&500_100_002],
            AccountRecord {
                account_id: 500_100_002,
                hidden: true,
                created_at: Some(1_451_606_400),
                pvp: None,
                pvp_solo: None,
            }
        );
        assert_eq!(
            accounts[&500_100_003].pvp,
            Some(totals(12, 5, 3, 301_200, 6, 9800))
        );
        assert_eq!(accounts[&500_100_003].pvp_solo, None);
        assert!(!accounts.contains_key(&500_100_004));
    }

    #[test]
    fn ship_stats_map_every_ship_with_optional_solo_totals() {
        let records = parse_ship_stats(payload(SHIPS_STATS), 500_100_001).expect("ship stats");
        assert_eq!(records.len(), 3);
        assert_eq!(
            records[0],
            ShipRecord {
                ship_id: 4_181_604_176,
                pvp: totals(310, 182, 121, 30_380_000, 372, 527_000),
                pvp_solo: Some(totals(240, 137, 90, 23_280_000, 281, 400_800)),
            }
        );
        assert_eq!(records[1].pvp_solo, None);
        assert_eq!(records[2].pvp_solo, None);
        assert_eq!(records[2].pvp, totals(7, 4, 2, 361_000, 5, 8400));
    }

    #[test]
    fn hidden_or_null_ship_stats_yield_no_records() {
        let hidden = payload(
            r#"{"status":"ok","meta":{"count":1,"hidden":[500100002]},"data":{"500100002":null}}"#,
        );
        assert_eq!(parse_ship_stats(hidden, 500_100_002), Ok(Vec::new()));
        let null = payload(
            r#"{"status":"ok","meta":{"count":1,"hidden":null},"data":{"500100009":null}}"#,
        );
        assert_eq!(parse_ship_stats(null, 500_100_009), Ok(Vec::new()));
        let absent = payload(r#"{"status":"ok","meta":{"count":0},"data":null}"#);
        assert_eq!(parse_ship_stats(absent, 500_100_009), Ok(Vec::new()));
    }

    #[test]
    fn clan_tags_cover_only_players_in_a_clan() {
        let tags = parse_clan_tags(payload(CLANS_ACCOUNTINFO)).expect("clan tags");
        assert_eq!(
            tags,
            HashMap::from([
                (500_100_001, "GULL".to_owned()),
                (500_100_003, "-NF-".to_owned()),
            ])
        );
    }

    #[test]
    fn ships_page_maps_types_and_reports_the_page_total() {
        let page = parse_ships_page(payload(ENCYCLOPEDIA_SHIPS)).expect("ships page");
        assert_eq!(page.page_total, 3);
        let ids: Vec<u64> = page.ships.iter().map(|ship| ship.id).collect();
        assert_eq!(
            ids,
            vec![
                3_340_678_960,
                3_751_704_560,
                3_762_206_416,
                4_074_649_552,
                4_181_604_176
            ]
        );
        let classes: Vec<ShipClass> = page.ships.iter().map(|ship| ship.class).collect();
        assert_eq!(
            classes,
            vec![
                ShipClass::Unknown,
                ShipClass::Battleship,
                ShipClass::Submarine,
                ShipClass::AirCarrier,
                ShipClass::Destroyer
            ]
        );
        assert_eq!(
            page.ships[1],
            Ship {
                id: 3_751_704_560,
                name: "Bastion".to_owned(),
                tier: 8,
                class: ShipClass::Battleship,
                nation: "uk".to_owned(),
                is_premium: true,
                is_special: false,
            }
        );
        assert!(page.ships[3].is_special);
    }

    #[test]
    fn two_hundred_thirty_names_make_three_exact_searches() {
        let requests = account_list_requests(&nicknames(230));
        let batch_sizes: Vec<usize> = requests
            .iter()
            .map(|request| param(request, "search").map_or(0, |search| search.split(',').count()))
            .collect();
        assert_eq!(batch_sizes, vec![100, 100, 30]);
        assert!(
            requests
                .iter()
                .all(|request| param(request, "type") == Some("exact"))
        );
        assert!(
            param(&requests[1], "search")
                .is_some_and(|search| search.starts_with("Deckhand_100,Deckhand_101"))
        );
    }

    #[test]
    fn request_url_targets_region_section_and_method() {
        let requests = account_list_requests(&nicknames(230));
        let url = request_url(Region::Na, APPLICATION_ID, &requests[2]).expect("url");
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("api.worldofwarships.com"));
        assert_eq!(url.path(), "/wows/account/list/");
        let pairs = query(&url);
        assert_eq!(pairs["application_id"], APPLICATION_ID);
        assert_eq!(pairs["type"], "exact");
        assert_eq!(pairs["limit"], "100");
        assert_eq!(pairs["search"].split(',').count(), 30);
    }

    #[test]
    fn account_and_clan_lookups_batch_ids_by_one_hundred() {
        let ids = account_ids(230);
        let info = account_info_requests(&ids);
        let clans = clan_info_requests(&ids);
        for requests in [&info, &clans] {
            let batch_sizes: Vec<usize> = requests
                .iter()
                .map(|request| param(request, "account_id").map_or(0, |ids| ids.split(',').count()))
                .collect();
            assert_eq!(batch_sizes, vec![100, 100, 30]);
        }
        assert_eq!(param(&info[0], "extra"), Some("statistics.pvp_solo"));
        let fields: Vec<&str> =
            param(&info[0], "fields").map_or(Vec::new(), |fields| fields.split(',').collect());
        assert!(fields.contains(&"hidden_profile"));
        assert!(fields.contains(&"statistics.pvp.survived_battles"));
        assert!(fields.contains(&"statistics.pvp_solo.damage_dealt"));
        assert_eq!(fields.len(), 15);
        assert_eq!(param(&clans[0], "extra"), Some("clan"));
        let url = request_url(Region::Asia, APPLICATION_ID, &clans[0]).expect("url");
        assert_eq!(url.host_str(), Some("api.worldofwarships.asia"));
        assert_eq!(url.path(), "/wows/clans/accountinfo/");
    }

    #[test]
    fn empty_inputs_need_no_requests() {
        assert!(account_list_requests(&[]).is_empty());
        assert!(account_info_requests(&[]).is_empty());
        assert!(clan_info_requests(&[]).is_empty());
    }

    #[test]
    fn ship_stats_request_asks_for_every_ship_of_one_account() {
        let request = ship_stats_request(500_100_001);
        let url = request_url(Region::Eu, APPLICATION_ID, &request).expect("url");
        assert_eq!(url.host_str(), Some("api.worldofwarships.eu"));
        assert_eq!(url.path(), "/wows/ships/stats/");
        let pairs = query(&url);
        assert_eq!(pairs["account_id"], "500100001");
        assert_eq!(pairs["extra"], "pvp_solo");
        assert!(!pairs.contains_key("ship_id"));
        assert!(pairs["fields"].starts_with("ship_id,pvp.battles"));
        assert!(pairs["fields"].contains("pvp_solo.xp"));
    }

    #[test]
    fn encyclopedia_requests_page_ships_and_read_the_game_version() {
        let page = encyclopedia_ships_request(4);
        assert_eq!(param(&page, "page_no"), Some("4"));
        assert_eq!(param(&page, "limit"), Some("100"));
        assert_eq!(param(&page, "fields"), Some(SHIP_FIELDS));
        let info = request_url(Region::Eu, APPLICATION_ID, &game_version_request()).expect("url");
        assert_eq!(info.path(), "/wows/encyclopedia/info/");
        assert_eq!(query(&info)["fields"], "game_version");
    }

    #[tokio::test]
    async fn rate_limited_request_is_retried_once_after_the_delay() {
        let delay = Duration::from_millis(20);
        let attempts = Cell::new(0);
        let started = Instant::now();
        let outcome = retry_once_when_rate_limited(delay, || {
            attempts.set(attempts.get() + 1);
            let attempt = attempts.get();
            async move {
                if attempt == 1 {
                    Err(WgError::RateLimited)
                } else {
                    Ok(attempt)
                }
            }
        })
        .await;
        assert_eq!(outcome, Ok(2));
        assert_eq!(attempts.get(), 2);
        assert!(started.elapsed() >= delay);
    }

    #[tokio::test]
    async fn persistent_rate_limit_surfaces_after_one_retry() {
        let attempts = Cell::new(0);
        let outcome: Result<(), WgError> =
            retry_once_when_rate_limited(Duration::from_millis(1), || {
                attempts.set(attempts.get() + 1);
                async { Err(WgError::RateLimited) }
            })
            .await;
        assert_eq!(outcome, Err(WgError::RateLimited));
        assert_eq!(attempts.get(), 2);
    }

    #[tokio::test]
    async fn other_failures_are_not_retried() {
        let attempts = Cell::new(0);
        let outcome: Result<(), WgError> =
            retry_once_when_rate_limited(Duration::from_millis(1), || {
                attempts.set(attempts.get() + 1);
                async { Err(WgError::Network("offline".to_owned())) }
            })
            .await;
        assert_eq!(outcome, Err(WgError::Network("offline".to_owned())));
        assert_eq!(attempts.get(), 1);
    }

    fn schedule(log: StartLog, arrival: Instant, count: usize) -> Vec<Instant> {
        (0..count)
            .fold((log, Vec::new()), |(log, starts), _| {
                let start = log.earliest_start(arrival);
                let starts = starts.into_iter().chain(iter::once(start)).collect();
                (log.with_start(start), starts)
            })
            .1
    }

    #[test]
    fn start_log_admits_eight_starts_per_rolling_second() {
        let origin = Instant::now();
        let starts = schedule(
            StartLog::new(MAX_STARTS_PER_WINDOW, RATE_WINDOW),
            origin,
            20,
        );
        assert!(starts[..8].iter().all(|start| *start == origin));
        assert!(
            starts[8..16]
                .iter()
                .all(|start| *start == origin + RATE_WINDOW)
        );
        assert!(
            starts[16..]
                .iter()
                .all(|start| *start == origin + RATE_WINDOW * 2)
        );
    }

    #[test]
    fn start_log_frees_capacity_once_the_window_rolls_past() {
        let origin = Instant::now();
        let busy = (0..MAX_STARTS_PER_WINDOW).fold(
            StartLog::new(MAX_STARTS_PER_WINDOW, RATE_WINDOW),
            |log, _| log.with_start(origin),
        );
        let midway = origin + Duration::from_millis(400);
        assert_eq!(busy.earliest_start(midway), origin + RATE_WINDOW);
        let later = origin + Duration::from_millis(1500);
        assert_eq!(busy.earliest_start(later), later);
        assert_eq!(busy.with_start(later).recent.len(), MAX_STARTS_PER_WINDOW);
    }

    fn sorted_instants(instants: Vec<Instant>) -> Vec<Instant> {
        let mut ordered = instants;
        ordered.sort();
        ordered
    }

    #[tokio::test]
    async fn limiter_holds_concurrent_callers_to_the_window() {
        let window = Duration::from_millis(100);
        let limiter = Arc::new(RateLimiter::new(2, window));
        let origin = Instant::now();
        let starts = futures::future::join_all((0..5).map(|_| {
            let limiter = Arc::clone(&limiter);
            async move {
                limiter.acquire().await;
                Instant::now()
            }
        }))
        .await;
        let ordered = sorted_instants(starts);
        assert!(
            ordered
                .windows(3)
                .all(|three| three[2] - three[0] >= window)
        );
        assert!(
            ordered
                .last()
                .is_some_and(|last| *last - origin >= window * 2)
        );
    }

    #[test]
    fn client_is_clone_send_and_sync() {
        fn assert_shareable<T: Clone + Send + Sync + 'static>() {}
        assert_shareable::<WgClient>();
    }

    #[test]
    fn client_futures_can_move_across_threads() {
        fn assert_send<T: Send>(_: T) {}
        let client = client(APPLICATION_ID);
        assert_send(client.verify_application_id());
        assert_send(client.account_ids(&[]));
        assert_send(client.accounts(&[]));
        assert_send(client.ship_stats(1));
        assert_send(client.clan_tags(&[]));
        assert_send(client.ships());
    }

    #[test]
    fn clones_share_one_rate_limiter() {
        let original = client(APPLICATION_ID);
        let copy = original.clone();
        assert!(Arc::ptr_eq(&original.limiter, &copy.limiter));
        assert_eq!(copy.region(), Region::Eu);
    }

    #[tokio::test]
    async fn blank_application_id_is_rejected_without_a_request() {
        assert_eq!(
            client("   ").verify_application_id().await,
            Err(WgError::InvalidApplicationId)
        );
    }

    #[tokio::test]
    async fn empty_lookups_resolve_without_a_request() {
        let client = client(APPLICATION_ID);
        assert_eq!(client.account_ids(&[]).await, Ok(HashMap::new()));
        assert_eq!(client.accounts(&[]).await, Ok(HashMap::new()));
        assert_eq!(client.clan_tags(&[]).await, Ok(HashMap::new()));
    }
}
