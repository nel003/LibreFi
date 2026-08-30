use crate::network::server::Server;
use crate::utils::db::{get_db, USERS_TABLE};
use crate::utils::get_mac_from_ip::get_mac_from_ip;
use redb::ReadableDatabase;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init(server: &mut Server) {
    server.get("/init", |req, res| {
        let Some(mac) = get_mac_from_ip(&req.ip) else {
            res.status = 403;
            res.body = b"{\"error\":\"Device not recognized\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        };

        let db = get_db();
        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(USERS_TABLE).unwrap();

        match table.get(mac.as_str()).unwrap() {
            Some(record) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let mut data: serde_json::Value =
                    serde_json::from_str(record.value()).unwrap_or(serde_json::Value::Null);

                if let Some(obj) = data.as_object_mut() {
                    obj.insert("now".to_string(), serde_json::json!(now));
                }

                res.status = 200;
                res.body = data.to_string().into_bytes();
                res.content_type = String::from("application/json");
            }
            None => {
                res.status = 404;
                res.body = b"{\"error\":\"User not found\"}".to_vec();
                res.content_type = String::from("application/json");
            }
        }
    });
}

