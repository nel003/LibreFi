use crate::network::server::Server;
use crate::utils::db::{USERS_TABLE, User, get_db};
use redb::{ReadableDatabase, ReadableTable};
use serde::{Deserialize, Serialize};

use super::auth::parse_admin_payload;

fn is_valid_mac(mac: &str) -> bool {
    let parts: Vec<&str> = mac.split(':').collect();
    parts.len() == 6 && parts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

#[derive(Deserialize, Default)]
struct GetUsersPayload {
    page: Option<usize>,
}

#[derive(Serialize, Clone)]
struct UserWithMac {
    mac: String,
    name: String,
    ip: String,
    paused: bool,
    pause_attempts: u8,
    pause_day: u32,
    paused_on: u32,
    expires_on: u32,
}

#[derive(Serialize)]
struct UsersListResponse {
    ok: bool,
    users: Vec<UserWithMac>,
    total_pages: usize,
    current_page: usize,
}

#[derive(Deserialize)]
struct UpdateUserPayload {
    mac: String,
    name: String,
    ip: String,
    paused: bool,
    pause_attempts: u8,
    pause_day: u32,
    paused_on: u32,
    expires_on: u32,
    new_expiry: Option<u32>,
}

pub fn handle_users(server: &mut Server) {
    server.get("http://localhost:8000/api/admin/users", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let get_payload: GetUsersPayload = serde_json::from_str(&json).unwrap_or_default();
        let page = get_payload.page.unwrap_or(1).max(1);
        let limit = 10;

        let db = get_db();
        let mut all_users = Vec::new();

        if let Ok(read_txn) = db.begin_read() {
            if let Ok(table) = read_txn.open_table(USERS_TABLE) {
                if let Ok(iter) = table.iter() {
                    for item in iter {
                        if let Ok((key, value)) = item {
                            if let Ok(user) = serde_json::from_str::<User>(value.value()) {
                                all_users.push(UserWithMac {
                                    mac: key.value().to_string(),
                                    name: user.name,
                                    ip: user.ip,
                                    paused: user.paused,
                                    pause_attempts: user.pause_attempts,
                                    pause_day: user.pause_day,
                                    paused_on: user.paused_on,
                                    expires_on: user.expires_on,
                                });
                            }
                        }
                    }
                }
            }
        }

        let total_users = all_users.len();
        let total_pages = if total_users == 0 {
            1
        } else {
            (total_users as f64 / limit as f64).ceil() as usize
        };

        let start = (page - 1) * limit;
        let end = (start + limit).min(total_users);
        let users = if start < total_users {
            all_users[start..end].to_vec()
        } else {
            Vec::new()
        };

        let resp = UsersListResponse {
            ok: true,
            users,
            total_pages,
            current_page: page,
        };
        res.status = 200;
        res.body = serde_json::to_vec(&resp).unwrap();
        res.content_type = String::from("application/json");
    });

    server.post("http://localhost:8000/api/admin/users", |req, res| {
        if req.body.len() > 65536 {
            res.status = 413;
            res.body = b"{\"error\":\"Payload too large\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        }

        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: UpdateUserPayload = match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Invalid user payload: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let mut user = User {
            name: payload.name,
            ip: payload.ip,
            paused: payload.paused,
            pause_attempts: payload.pause_attempts,
            pause_day: payload.pause_day,
            paused_on: payload.paused_on,
            expires_on: payload.expires_on,
        };

        if let Some(new_expiry) = payload.new_expiry {
            if user.paused {
                let p_on = if user.paused_on > 0 {
                    user.paused_on
                } else {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as u32
                };
                user.paused_on = p_on;
                user.expires_on = p_on + new_expiry;
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as u32;
                user.expires_on = now + new_expiry;
            }
        }

        if !is_valid_mac(&payload.mac) {
            res.status = 400;
            res.body = b"{\"error\":\"Invalid MAC address format\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        }

        let json_value = serde_json::to_string(&user).unwrap();
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(USERS_TABLE).unwrap();
            table
                .insert(payload.mac.as_str(), json_value.as_str())
                .unwrap();
        }

        write_txn.commit().unwrap();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        if user.paused || user.expires_on <= now {
            let cmd = format!(
                "nft delete element inet librefi allowed_macs {{ {} }}",
                payload.mac
            );
            let _ = crate::utils::setup_captive_portal::run_sh_cmd(&cmd, true);
        } else {
            let diff = user.expires_on.saturating_sub(now);
            if diff > 0 {
                let cmd = format!(
                    "nft add element inet librefi allowed_macs {{ {} timeout {}s }}",
                    payload.mac, diff
                );
                let _ = crate::utils::setup_captive_portal::run_sh_cmd(&cmd, true);
            }
        }

        res.status = 200;
        res.body = b"{\"ok\":true}".to_vec();
        res.content_type = String::from("application/json");
    });
}
