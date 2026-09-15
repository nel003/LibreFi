use crate::network::server::Server;
use crate::utils::db::{get_db, User, USERS_TABLE};
use redb::{ReadableDatabase, ReadableTable};

pub fn debug(server: &mut Server) {
    server.get("/debug/db", |_req, res| {
        let db = get_db();
        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(USERS_TABLE).unwrap();

        let mut entries: Vec<serde_json::Value> = Vec::new();

        for item in table.iter().unwrap() {
            let (key, value) = item.unwrap();
            let mac = key.value().to_string();
            let user: Result<User, _> = serde_json::from_str(value.value());

            let entry = match user {
                Ok(u) => serde_json::json!({
                    "mac": mac,
                    "name": u.name,
                    "ip": u.ip,
                    "paused": u.paused,
                    "pause_attempts": u.pause_attempts,
                    "pause_day": u.pause_day,
                    "paused_on": u.paused_on,
                    "expires_on": u.expires_on,
                }),
                Err(e) => serde_json::json!({
                    "mac": mac,
                    "error": format!("Failed to deserialize: {}", e),
                    "raw": value.value(),
                }),
            };

            entries.push(entry);
        }

        let body = serde_json::json!({
            "table": "users",
            "count": entries.len(),
            "rows": entries,
        });

        res.status = 200;
        res.body = body.to_string().into_bytes();
        res.content_type = String::from("application/json");
    });

    server.get("/debug/reset_pauses", |_req, res| {
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        
        let mut count = 0;
        {
            let mut table = write_txn.open_table(USERS_TABLE).unwrap();
            let mut updates = Vec::new();

            // First, gather all users
            for item in table.iter().unwrap() {
                let (key, value) = item.unwrap();
                let mac = key.value().to_string();
                if let Ok(mut user) = serde_json::from_str::<User>(value.value()) {
                    user.pause_attempts = 0;
                    user.pause_day = 0;
                    updates.push((mac, serde_json::to_string(&user).unwrap()));
                }
            }

            // Then apply updates
            for (mac, user_json) in updates {
                table.insert(mac.as_str(), user_json.as_str()).unwrap();
                count += 1;
            }
        }
        write_txn.commit().unwrap();

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"message\":\"Reset daily pause limits for {} users\"}}",
            count
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });

    server.get("/debug/clear_time", |_req, res| {
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        
        let mut count = 0;
        {
            let mut table = write_txn.open_table(USERS_TABLE).unwrap();
            let mut updates = Vec::new();

            for item in table.iter().unwrap() {
                let (key, value) = item.unwrap();
                let mac = key.value().to_string();
                if let Ok(mut user) = serde_json::from_str::<User>(value.value()) {
                    user.expires_on = 0; // Clear the time
                    updates.push((mac, serde_json::to_string(&user).unwrap()));
                }
            }

            for (mac, user_json) in updates {
                table.insert(mac.as_str(), user_json.as_str()).unwrap();
                count += 1;
            }
        }
        write_txn.commit().unwrap();

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"message\":\"Cleared time for {} users\"}}",
            count
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });

    server.get("/debug/clear_vouchers", |_req, res| {
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        
        let mut count = 0;
        {
            let mut table = write_txn.open_table(crate::utils::db::VOUCHERS_TABLE).unwrap();
            let mut keys = Vec::new();

            for item in table.iter().unwrap() {
                let (key, _) = item.unwrap();
                keys.push(key.value().to_string());
            }

            for key in keys {
                table.remove(key.as_str()).unwrap();
                count += 1;
            }
        }
        write_txn.commit().unwrap();

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"message\":\"Cleared {} vouchers\"}}",
            count
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
