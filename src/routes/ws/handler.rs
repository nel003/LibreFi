use base64::Engine;
use tiny_http::ReadWrite;
use tungstenite::protocol::{Message, WebSocket};

use crate::routes::admin::coinslot_key::get_coinslot_key_bytes;
use crate::routes::admin::key_bytes;
use crate::utils::crypto::decrypt_payload;
use crate::utils::get_mac_from_ip::get_mac_from_ip;

use super::coinslot::run_coinslot;
use super::state::{EncryptedPayload, WsMessage, get_subnet};
use super::user::{handle_user_ack, run_user};

pub fn ws_handler(ws: WebSocket<Box<dyn ReadWrite + Send>>, ip: String) {
    let mac = get_mac_from_ip(&ip).unwrap_or_else(|| String::from("Unknown MAC"));
    crate::debug_println!("New WebSocket connection from IP: {}, MAC: {}", ip, mac);

    let subnet = get_subnet(&ip);

    let mut ws = ws;
    let first_msg = match ws.read() {
        Ok(m) => m,
        Err(e) => {
            crate::debug_println!("[WS:{}] Failed to read first message: {}", ip, e);
            return;
        }
    };

    if !first_msg.is_text() {
        crate::debug_println!("[WS:{}] First message is not text — dropping.", ip);
        return;
    }

    let txt = first_msg.to_text().unwrap_or("").to_string();

    if let Ok(enc) = serde_json::from_str::<EncryptedPayload>(&txt) {
        match base64::engine::general_purpose::STANDARD.decode(&enc.payload) {
            Ok(bytes) => {
                let mut decrypted = false;

                // 1. Try Coinslot Key
                if let Some(coinslot_key) = get_coinslot_key_bytes() {
                    if let Ok(json_str) = decrypt_payload(&bytes, &coinslot_key) {
                        crate::debug_println!("[WS:{}] Decrypted with coinslot_key: {}", ip, json_str);
                        if let Ok(msg) = serde_json::from_str::<WsMessage>(&json_str) {
                            if msg.msg_type == "COINSLOT" {
                                crate::debug_println!("[WS:{}] Authenticated as COINSLOT on subnet {}", ip, subnet);
                                let _ = ws.send(Message::Text(
                                    "{\"type\":\"status\",\"value\":\"ok\"}".to_string().into(),
                                ));
                                run_coinslot(ws, subnet, ip);
                                return;
                            }
                        }
                        decrypted = true;
                    }
                }

                // 2. Try User/Admin Key fallback
                if !decrypted {
                    match decrypt_payload(&bytes, &key_bytes()) {
                        Ok(json_str) => {
                            crate::debug_println!("[WS:{}] Decrypted with admin key: {}", ip, json_str);
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&json_str) {
                                if msg.msg_type == "COINSLOT" {
                                    crate::debug_println!("[WS:{}] WARNING: Coinslot attempted to authenticate using the master admin key. Rejected.", ip);
                                    return;
                                }
                            }
                        }
                        Err(e) => crate::debug_println!("[WS:{}] Decryption failed: {:?} — treating as user.", ip, e),
                    }
                }
            }
            Err(e) => crate::debug_println!(
                "[WS:{}] Base64 decode failed: {} — treating as user.",
                ip, e
            ),
        }
    }

    let mut user_ws = ws;

    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&txt) {
        if ws_msg.msg_type == "ACK" {
            handle_user_ack(&mut user_ws, &ip, &subnet);
        }
    }

    run_user(user_ws, subnet, ip);
}
