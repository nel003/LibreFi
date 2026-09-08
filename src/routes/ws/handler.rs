use base64::Engine;
use tiny_http::ReadWrite;
use tungstenite::protocol::{Message, WebSocket};

use crate::routes::admin::key_bytes;
use crate::utils::crypto::decrypt_payload;
use crate::utils::get_mac_from_ip::get_mac_from_ip;

use super::coinslot::run_coinslot;
use super::state::{EncryptedPayload, WsMessage, get_subnet};
use super::user::{handle_user_ack, run_user};

pub fn ws_handler(ws: WebSocket<Box<dyn ReadWrite + Send>>, ip: String) {
    let mac = get_mac_from_ip(&ip).unwrap_or_else(|| String::from("Unknown MAC"));
    println!("New WebSocket connection from IP: {}, MAC: {}", ip, mac);

    let subnet = get_subnet(&ip);

    let mut ws = ws;
    let first_msg = match ws.read() {
        Ok(m) => m,
        Err(e) => {
            println!("[WS:{}] Failed to read first message: {}", ip, e);
            return;
        }
    };

    if !first_msg.is_text() {
        println!("[WS:{}] First message is not text — dropping.", ip);
        return;
    }

    let txt = first_msg.to_text().unwrap_or("").to_string();

    if let Ok(enc) = serde_json::from_str::<EncryptedPayload>(&txt) {
        match base64::engine::general_purpose::STANDARD.decode(&enc.payload) {
            Ok(bytes) => match decrypt_payload(&bytes, &key_bytes()) {
                Ok(json_str) => {
                    println!("[WS:{}] Decrypted: {}", ip, json_str);
                    if let Ok(msg) = serde_json::from_str::<WsMessage>(&json_str) {
                        if msg.msg_type == "COINSLOT" {
                            println!("[WS:{}] Authenticated as COINSLOT on subnet {}", ip, subnet);
                            let _ = ws.send(Message::Text(
                                "{\"type\":\"status\",\"value\":\"ok\"}".to_string().into(),
                            ));
                            run_coinslot(ws, subnet, ip);
                            return;
                        }
                    }
                    println!(
                        "[WS:{}] Decrypted payload was not COINSLOT — treating as user.",
                        ip
                    );
                }
                Err(e) => println!("[WS:{}] Decryption failed: {:?} — treating as user.", ip, e),
            },
            Err(e) => println!(
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
