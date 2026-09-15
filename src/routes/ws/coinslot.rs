use base64::Engine;
use std::sync::mpsc;
use std::time::Duration;
use tiny_http::ReadWrite;
use tungstenite::protocol::{Message, WebSocket};

use crate::routes::admin::coinslot_key::get_coinslot_key_bytes;
use crate::utils::crypto::decrypt_payload;
use crate::utils::db::{add_time_to_user, convert_amount_to_time, record_sale};
use crate::utils::setup_captive_portal::allow_mac;

use super::state::{
    ACTIVE_USER, EncryptedPayload, RelayRequest, RelayResult, USER_EVENTS, WsMessage,
    get_coinslot_senders,
};

pub fn run_coinslot(mut ws: WebSocket<Box<dyn ReadWrite + Send>>, subnet: String, _ip: String) {
    let (tx, rx) = mpsc::sync_channel::<RelayRequest>(1);

    get_coinslot_senders()
        .lock()
        .unwrap()
        .insert(subnet.clone(), tx);

    crate::debug_println!(
        "[CS:{}] Registered coinslot, waiting for relay requests.",
        subnet
    );

    loop {
        match rx.try_recv() {
            Ok(req) => {
                if req.command == "ACK" {
                    crate::debug_println!("[CS:{}] Relaying ACK to ESP device…", subnet);

                    if ws
                        .send(Message::Text("{\"type\":\"ACK\"}".to_string().into()))
                        .is_err()
                    {
                        crate::debug_println!(
                            "[CS:{}] Failed to send ACK to ESP — connection dead.",
                            subnet
                        );
                        if let Some(reply_tx) = req.reply_tx {
                            let _ = reply_tx.send(RelayResult::NotAvailable(
                                "Coinslot connection lost".to_string(),
                            ));
                        }
                        break;
                    }

                    let result = wait_for_coinslot_ack(&mut ws, Duration::from_secs(4));
                    if let Some(reply_tx) = req.reply_tx {
                        let _ = reply_tx.send(result);
                    }
                } else if req.command == "close" {
                    let _ = ws.send(Message::Text(
                        "{\"type\":\"cmd\",\"value\":\"close\"}".to_string().into(),
                    ));
                }
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                crate::debug_println!("[CS:{}] Relay channel dropped — cleaning up.", subnet);
                break;
            }
        }

        match ws.read() {
            Ok(msg) if msg.is_text() => {
                let txt = msg.to_text().unwrap_or("");
                match try_decrypt_esp_msg(txt) {
                    Some(plain) => {
                        crate::debug_println!("[CS:{}] Message from ESP: {}", subnet, plain);
                        handle_esp_message(&plain, &subnet);
                    }
                    None => {
                        crate::debug_println!("[CS:{}] Rejected unencrypted/invalid message.", subnet);
                    }
                }
            }
            Ok(msg) if msg.is_close() => {
                crate::debug_println!("[CS:{}] ESP closed the connection.", subnet);
                break;
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                crate::debug_println!("[CS:{}] Read error: {} — disconnecting.", subnet, e);
                break;
            }
        }
    }

    // Clean up the sender so user threads don't try to use a stale entry.
    get_coinslot_senders().lock().unwrap().remove(&subnet);

    crate::debug_println!("[CS:{}] Coinslot thread exiting.", subnet);
}

/// Block (with timeout) reading from `ws` until we get a response that tells
/// us whether the ESP coinslot is actually available.
pub fn wait_for_coinslot_ack(
    ws: &mut WebSocket<Box<dyn ReadWrite + Send>>,
    timeout: Duration,
) -> RelayResult {
    let deadline = std::time::Instant::now() + timeout;

    while std::time::Instant::now() < deadline {
        match ws.read() {
            Ok(msg) if msg.is_text() => {
                let txt = msg.to_text().unwrap_or("");
                crate::debug_println!("[CS relay] ESP raw reply received");

                let plain = match try_decrypt_esp_msg(txt) {
                    Some(p) => p,
                    None => {
                        crate::debug_println!("[CS relay] Rejected unencrypted reply — still waiting…");
                        continue;
                    }
                };

                crate::debug_println!("[CS relay] ESP decrypted reply: {}", plain);
                if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&plain) {
                    let t = doc["type"].as_str().unwrap_or("");
                    if t == "ACK_SUCCESS"
                        || t == "ping"
                        || t == "notify"
                        || t == "status"
                        || t == "init"
                    {
                        return RelayResult::Available;
                    }
                    if doc["error"].is_string() {
                        return RelayResult::NotAvailable(
                            doc["error"].as_str().unwrap_or("Unknown error").to_string(),
                        );
                    }
                    return RelayResult::Available;
                }
            }
            Ok(msg) if msg.is_close() => {
                return RelayResult::NotAvailable("ESP closed connection".to_string());
            }
            Ok(_) => {}
            Err(tungstenite::Error::Io(ref e))
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return RelayResult::NotAvailable(format!("Read error: {}", e));
            }
        }
    }
    RelayResult::Timeout
}

pub fn try_decrypt_esp_msg(txt: &str) -> Option<String> {
    let enc = match serde_json::from_str::<EncryptedPayload>(txt) {
        Ok(e) => e,
        Err(e) => {
            crate::debug_println!("[CS-DEBUG] JSON parse failed: {:?}", e);
            return None;
        }
    };

    let bytes = match base64::engine::general_purpose::STANDARD.decode(&enc.payload) {
        Ok(b) => b,
        Err(e) => {
            crate::debug_println!("[CS-DEBUG] Base64 decode failed: {:?}", e);
            return None;
        }
    };

    if let Some(coinslot_key) = get_coinslot_key_bytes() {
        match decrypt_payload(&bytes, &coinslot_key) {
            Ok(plain) => {
                crate::debug_println!("[CS-DEBUG] Decryption successful!");
                return Some(plain);
            }
            Err(e) => {
                crate::debug_println!("[CS-DEBUG] Decryption failed: {:?}", e);
            }
        }
    } else {
        crate::debug_println!("[CS-DEBUG] No coinslot key found in database!");
    }
    None
}

pub fn handle_esp_message(txt: &str, subnet: &str) {
    if let Ok(mut doc) = serde_json::from_str::<WsMessage>(txt) {
        match doc.msg_type.as_str() {
            "ping" => {} // Heartbeat — silently ignored, only used to unblock ws.read()
            "init" => crate::debug_println!("[CS:{}] ESP init: {:?}", subnet, doc.value),
            "res" => crate::debug_println!("[CS:{}] ESP slot response: {:?}", subnet, doc.value),
            "timer" => {
                let active_map = ACTIVE_USER.lock().unwrap();
                if let Some(mac) = active_map.get(subnet) {
                    let mut events = USER_EVENTS.lock().unwrap();
                    events.entry(mac.clone()).or_insert_with(Vec::new).push(doc);
                }
            }
            "notify" => {
                crate::debug_println!("[CS:{}] ESP coin pulse detected", subnet);
                let active_map = ACTIVE_USER.lock().unwrap();
                if let Some(mac) = active_map.get(subnet) {
                    let mut events = USER_EVENTS.lock().unwrap();
                    events.entry(mac.clone()).or_insert_with(Vec::new).push(doc);
                }
            }
            "coin" => {
                let amount: u32 = match &doc.value {
                    Some(v) if v.is_u64() => v.as_u64().unwrap_or(0) as u32,
                    Some(v) if v.is_string() => v.as_str().unwrap().parse().unwrap_or(0),
                    _ => 0,
                };
                crate::debug_println!("[CS:{}] ESP coin inserted, amount: {}", subnet, amount);

                let active_map = ACTIVE_USER.lock().unwrap();
                if let Some(mac) = active_map.get(subnet) {
                    let seconds = convert_amount_to_time(amount);
                    if seconds > 0 {
                        record_sale(amount);
                        if let Ok(user) = add_time_to_user(mac, seconds) {
                            if !user.paused {
                                allow_mac(mac);
                            }
                            doc.time = Some(serde_json::json!(seconds));
                            crate::debug_println!(
                                "[CS:{}] Credited {} seconds to MAC {}",
                                subnet, seconds, mac
                            );
                        } else {
                            crate::debug_println!(
                                "[CS:{}] Failed to credit time: user {} not found",
                                subnet, mac
                            );
                        }
                    } else {
                        crate::debug_println!(
                            "[CS:{}] Warning: 0 seconds calculated for amount {}. Are rates set?",
                            subnet, amount
                        );
                    }

                    // Always push the coin event so the frontend knows the coinslot finished
                    let mut events = USER_EVENTS.lock().unwrap();
                    events.entry(mac.clone()).or_insert_with(Vec::new).push(doc);
                }
            }
            other => crate::debug_println!("[CS:{}] Unknown ESP message type: {}", subnet, other),
        }
    }
}
