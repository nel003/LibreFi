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
}
