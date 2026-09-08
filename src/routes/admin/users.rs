use crate::network::server::Server;
use crate::utils::db::{get_db, User, USERS_TABLE};
use redb::{ReadableDatabase, ReadableTable};
use serde::{Deserialize, Serialize};

use super::auth::parse_admin_payload;

#[derive(Serialize)]
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
}

pub fn handle_users(server: &mut Server) {
    server.get("/admin/users", |req, res| {
        // Require encrypted payload for authentication, even on GET requests!
        let _json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let db = get_db();
        let mut users = Vec::new();

        if let Ok(read_txn) = db.begin_read() {
            if let Ok(table) = read_txn.open_table(USERS_TABLE) {
                if let Ok(iter) = table.iter() {
                    for item in iter {
                        if let Ok((key, value)) = item {
                            if let Ok(user) = serde_json::from_str::<User>(value.value()) {
                                users.push(UserWithMac {
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

        let resp = UsersListResponse { ok: true, users };
        res.status = 200;
        res.body = serde_json::to_vec(&resp).unwrap();
        res.content_type = String::from("application/json");
    });

    server.post("/admin/users", |req, res| {
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

        let user = User {
            name: payload.name,
            ip: payload.ip,
            paused: payload.paused,
            pause_attempts: payload.pause_attempts,
            pause_day: payload.pause_day,
            paused_on: payload.paused_on,
            expires_on: payload.expires_on,
        };

        let json_value = serde_json::to_string(&user).unwrap();
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(USERS_TABLE).unwrap();
            table.insert(payload.mac.as_str(), json_value.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        res.status = 200;
        res.body = b"{\"ok\":true}".to_vec();
        res.content_type = String::from("application/json");
    });
}
