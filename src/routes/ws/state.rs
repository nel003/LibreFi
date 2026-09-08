use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, OnceLock, mpsc};

pub enum RelayResult {
    Available,
    NotAvailable(String),
    Timeout,
}

pub struct RelayRequest {
    pub reply_tx: Option<mpsc::SyncSender<RelayResult>>,
    pub command: String,
}

pub type CoinslotSender = mpsc::SyncSender<RelayRequest>;

static COINSLOT_SENDERS: OnceLock<Mutex<HashMap<String, CoinslotSender>>> = OnceLock::new();

pub fn get_coinslot_senders() -> &'static Mutex<HashMap<String, CoinslotSender>> {
    COINSLOT_SENDERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub static USER_EVENTS: LazyLock<Mutex<HashMap<String, Vec<WsMessage>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub static ACTIVE_USER: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct EncryptedPayload {
    pub payload: String,
}

pub fn get_subnet(ip: &str) -> String {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() == 4 {
        format!("{}.{}.{}", parts[0], parts[1], parts[2])
    } else {
        ip.to_string()
    }
}
