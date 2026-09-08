use std::sync::mpsc;
use std::time::Duration;
use tiny_http::ReadWrite;
use tungstenite::protocol::{Message, WebSocket};

use super::state::{
    ACTIVE_USER, RelayRequest, RelayResult, USER_EVENTS, WsMessage, get_coinslot_senders,
};

pub fn run_user(mut ws: WebSocket<Box<dyn ReadWrite + Send>>, subnet: String, ip: String) {
    println!("[USER:{}] Browser client connected.", ip);

    loop {
        let msg = match ws.read() {
            Ok(m) => m,
            Err(_) => {
                println!("[USER:{}] Disconnected.", ip);
                break;
            }
        };

        if msg.is_close() {
            println!("[USER:{}] Browser closed the connection.", ip);
            break;
        }

        if !msg.is_text() {
            continue;
        }

        let txt = msg.to_text().unwrap_or("");
        println!("[USER:{}] Received: {}", ip, txt);

        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(txt) {
            if ws_msg.msg_type == "ping" {
                let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(&ip).unwrap_or_default();
                let mut events_map = USER_EVENTS.lock().unwrap();
                if let Some(events) = events_map.get_mut(&mac) {
                    for ev in events.drain(..) {
                        if let Ok(json) = serde_json::to_string(&ev) {
                            let _ = ws.send(Message::Text(json.into()));
                        }
                    }
                }
            } else if ws_msg.msg_type == "DONE" {
                let sender = get_coinslot_senders().lock().unwrap().get(&subnet).cloned();
                if let Some(coinslot_tx) = sender {
                    let relay_req = RelayRequest { reply_tx: None, command: "close".to_string() };
                    let _ = coinslot_tx.send(relay_req);
                }
                
                let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(&ip).unwrap_or_default();
                let mut active_map = ACTIVE_USER.lock().unwrap();
                if active_map.get(&subnet) == Some(&mac) {
                    active_map.remove(&subnet);
                }
            } else if ws_msg.msg_type == "ACK" {
                let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(&ip).unwrap_or_default();
                {
                    let active_map = ACTIVE_USER.lock().unwrap();
                    if let Some(active_mac) = active_map.get(&subnet) {
                        if active_mac != &mac {
                            println!("[USER:{}] Coinslot is already in use by MAC {}", ip, active_mac);
                            let _ = ws.send(Message::Text(
                                "{\"error\":\"Coinslot is currently in use by another user\"}".to_string().into(),
                            ));
                            continue;
                        }
                    }
                }
                
                let sender = get_coinslot_senders().lock().unwrap().get(&subnet).cloned();

                match sender {
                    None => {
                        println!("[USER:{}] No coinslot registered on subnet {}", ip, subnet);
                        let _ = ws.send(Message::Text(
                            "{\"error\":\"Coinslot is not available\"}"
                                .to_string()
                                .into(),
                        ));
                    }
                    Some(coinslot_tx) => {
                        println!(
                            "[USER:{}] Coinslot found on subnet {}, relaying ACK…",
                            ip, subnet
                        );

                        let (reply_tx, reply_rx) = mpsc::sync_channel::<RelayResult>(1);
                        let relay_req = RelayRequest { reply_tx: Some(reply_tx), command: "ACK".to_string() };

                        if coinslot_tx.send(relay_req).is_err() {
                            println!("[USER:{}] Coinslot relay channel closed.", ip);
                            let _ = ws.send(Message::Text(
                                "{\"error\":\"Coinslot disconnected\"}".to_string().into(),
                            ));
                            continue;
                        }

                        // Wait up to 5 s for the coinslot thread to ping the ESP
                        // and return the result.
                        match reply_rx.recv_timeout(Duration::from_secs(5)) {
                            Ok(RelayResult::Available) => {
                                println!(
                                    "[USER:{}] Coinslot confirmed available → ACK_SUCCESS",
                                    ip
                                );
                                let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(&ip).unwrap_or_default();
                                ACTIVE_USER.lock().unwrap().insert(subnet.clone(), mac);
                                let _ = ws.send(Message::Text(
                                    "{\"type\":\"ACK_SUCCESS\"}".to_string().into(),
                                ));
                            }
                            Ok(RelayResult::NotAvailable(reason)) => {
                                println!("[USER:{}] Coinslot not available: {}", ip, reason);
                                let _ = ws.send(Message::Text(
                                    format!("{{\"error\":\"{}\"}}", reason).into(),
                                ));
                            }
                            Ok(RelayResult::Timeout) | Err(_) => {
                                println!("[USER:{}] Coinslot relay timed out.", ip);
                                let _ = ws.send(Message::Text(
                                    "{\"error\":\"Coinslot did not respond in time\"}"
                                        .to_string()
                                        .into(),
                                ));
                            }
                        }
                    }
                }
            } else {
                println!("[USER:{}] Unhandled message type: {}", ip, ws_msg.msg_type);
            }
        } else {
            println!("[USER:{}] Could not parse message: {}", ip, txt);
        }
    }

    println!("[USER:{}] User thread exiting.", ip);
    let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(&ip).unwrap_or_default();
    let mut active_map = ACTIVE_USER.lock().unwrap();
    if active_map.get(&subnet) == Some(&mac) {
        active_map.remove(&subnet);
        // Force close if they disconnected unexpectedly
        let sender = get_coinslot_senders().lock().unwrap().get(&subnet).cloned();
        if let Some(coinslot_tx) = sender {
            let _ = coinslot_tx.send(RelayRequest { reply_tx: None, command: "close".to_string() });
        }
    }
}

pub fn handle_user_ack(ws: &mut WebSocket<Box<dyn ReadWrite + Send>>, ip: &str, subnet: &str) {
    let sender = get_coinslot_senders().lock().unwrap().get(subnet).cloned();

    match sender {
        None => {
            println!(
                "[USER:{}] No coinslot on subnet {} → not available",
                ip, subnet
            );
            let _ = ws.send(Message::Text(
                "{\"error\":\"Coinslot is not available\"}"
                    .to_string()
                    .into(),
            ));
        }
        Some(coinslot_tx) => {
            println!(
                "[USER:{}] Relaying ACK to coinslot on subnet {}",
                ip, subnet
            );
            
            let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(ip).unwrap_or_default();
            {
                let active_map = ACTIVE_USER.lock().unwrap();
                if let Some(active_mac) = active_map.get(subnet) {
                    if active_mac != &mac {
                        println!("[USER:{}] Coinslot is already in use by MAC {}", ip, active_mac);
                        let _ = ws.send(Message::Text(
                            "{\"error\":\"Coinslot is currently in use by another user\"}".to_string().into(),
                        ));
                        return;
                    }
                }
            }

            let (reply_tx, reply_rx) = mpsc::sync_channel::<RelayResult>(1);
            if coinslot_tx.send(RelayRequest { reply_tx: Some(reply_tx), command: "ACK".to_string() }).is_err() {
                let _ = ws.send(Message::Text(
                    "{\"error\":\"Coinslot disconnected\"}".to_string().into(),
                ));
                return;
            }

            match reply_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(RelayResult::Available) => {
                    println!("[USER:{}] ACK_SUCCESS", ip);
                    let mac = crate::utils::get_mac_from_ip::get_mac_from_ip(ip).unwrap_or_default();
                    ACTIVE_USER.lock().unwrap().insert(subnet.to_string(), mac);
                    let _ = ws.send(Message::Text(
                        "{\"type\":\"ACK_SUCCESS\"}".to_string().into(),
                    ));
                }
                Ok(RelayResult::NotAvailable(reason)) => {
                    println!("[USER:{}] Coinslot not available: {}", ip, reason);
                    let _ = ws.send(Message::Text(
                        format!("{{\"error\":\"{}\"}}", reason).into(),
                    ));
                }
                Ok(RelayResult::Timeout) | Err(_) => {
                    println!("[USER:{}] Coinslot relay timed out.", ip);
                    let _ = ws.send(Message::Text(
                        "{\"error\":\"Coinslot did not respond in time\"}"
                            .to_string()
                            .into(),
                    ));
                }
            }
        }
    }
}
