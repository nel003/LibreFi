use crate::network::server::Server;
use crate::utils::db::{CONFIG_TABLE, get_db};
use redb::ReadableDatabase;
use serde::Serialize;
use std::time::SystemTime;

use super::auth::parse_admin_payload;

#[derive(Serialize)]
struct CoinslotKeyResponse {
    key: String,
}

#[derive(Serialize)]
struct CoinslotKeyGetResponse {
    prefix: String,
    generated_at: String,
}

fn generate_random_key() -> String {
    #[cfg(unix)]
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let mut buf = [0u8; 16];
        if f.read_exact(&mut buf).is_ok() {
            return buf.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        }
    }

    // Fallback if not unix or /dev/urandom fails
    let t1 = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let t2 = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .wrapping_mul(31);
    format!("{:016x}{:016x}", t1 as u64, t2 as u64)
}

pub fn get_coinslot_key_bytes() -> Option<[u8; 32]> {
    let db = get_db();
    if let Ok(read_txn) = db.begin_read() {
        if let Ok(table) = read_txn.open_table(CONFIG_TABLE) {
            if let Ok(Some(value)) = table.get("coinslot_key") {
                let key_str = value.value();
                let src = key_str.as_bytes();
                let mut key = [0u8; 32];
                let len = src.len().min(32);
                key[..len].copy_from_slice(&src[..len]);
                return Some(key);
            }
        }
    }
    None
}

pub fn handle_coinslot_key(server: &mut Server) {
    server.post("http://localhost:8000/api/admin/coinslot-key", |req, res| {
        // Enforce encrypted payload for authentication
        let _json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let new_key = generate_random_key();

        // Save new key to database
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
            let mut table = write_txn.open_table(CONFIG_TABLE).unwrap();
            table.insert("coinslot_key", new_key.as_str()).unwrap();
            table.insert("coinslot_key_generated_at", now.to_string().as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        let resp = CoinslotKeyResponse { key: new_key };

        res.status = 200;
        res.body = serde_json::to_vec(&resp).unwrap();
        res.content_type = String::from("application/json");
    });

    server.get("http://localhost:8000/api/admin/coinslot-key", |req, res| {
        // Enforce encrypted payload even for GET
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
        let mut existing_key = String::new();
        let mut generated_at = String::from("Unknown");

        if let Ok(read_txn) = db.begin_read() {
            if let Ok(table) = read_txn.open_table(CONFIG_TABLE) {
                if let Ok(Some(value)) = table.get("coinslot_key") {
                    existing_key = value.value().to_string();
                }
                if let Ok(Some(value)) = table.get("coinslot_key_generated_at") {
                    generated_at = value.value().to_string();
                }
            }
        }

        let prefix = if existing_key.len() >= 8 {
            format!("{}...{}", &existing_key[0..4], &existing_key[existing_key.len() - 4..])
        } else {
            String::from("None")
        };

        let resp = CoinslotKeyGetResponse { prefix, generated_at };

        res.status = 200;
        res.body = serde_json::to_vec(&resp).unwrap();
        res.content_type = String::from("application/json");
    });
}
