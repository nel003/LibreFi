use base64::Engine;
use crate::network::server::Server;
use crate::utils::crypto::decrypt_payload;
use crate::utils::db::{get_db, Rate, Voucher, RATES_TABLE, VOUCHERS_TABLE};
use serde::Deserialize;

pub const ADMIN_SECRET_KEY: &str = "arns_super_pogi_4ever";

#[derive(Deserialize)]
struct AdminBody {
    payload: String,
}

#[derive(Deserialize)]
struct InsertRatePayload {
    id: u32,
    price: f64,
    time: u32,
}

#[derive(Deserialize)]
struct InsertVoucherPayload {
    code: String,
    price: f64,
    time: u32,
}

fn key_bytes() -> [u8; 32] {
    let mut key = [0u8; 32];
    let src = ADMIN_SECRET_KEY.as_bytes();
    let len = src.len().min(32);
    key[..len].copy_from_slice(&src[..len]);
    key
}

pub fn admin(server: &mut Server) {
    server.post("/admin/rates", |req, res| {
        let body_str = match String::from_utf8(req.body.clone()) {
            Ok(s) => s,
            Err(_) => {
                res.status = 400;
                res.body = b"{\"error\":\"Body is not valid UTF-8\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let admin_body: AdminBody = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Expected {{\\\"payload\\\":\\\"...\\\"}}: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let encrypted = match base64::engine::general_purpose::STANDARD.decode(&admin_body.payload) {
            Ok(b) => b,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Base64 decode failed: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let json = match decrypt_payload(&encrypted, &key_bytes()) {
            Ok(j) => j,
            Err(e) => {
                res.status = 401;
                res.body = format!("{{\"error\":\"Decryption failed: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };
        
        let payload: InsertRatePayload = match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Invalid rate payload: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let rate = Rate {
            price: payload.price,
            time: payload.time,
        };

        let json_value = serde_json::to_string(&rate).unwrap();
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(RATES_TABLE).unwrap();
            table.insert(payload.id, json_value.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        println!("-> Rate [id={}] upserted: {}", payload.id, json_value);

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"id\":{},\"rate\":{}}}",
            payload.id, json_value
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });

    server.post("/admin/vouchers", |req, res| {
        let body_str = match String::from_utf8(req.body.clone()) {
            Ok(s) => s,
            Err(_) => {
                res.status = 400;
                res.body = b"{\"error\":\"Body is not valid UTF-8\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let admin_body: AdminBody = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Expected {{\\\"payload\\\":\\\"...\\\"}}: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let encrypted = match base64::engine::general_purpose::STANDARD.decode(&admin_body.payload) {
            Ok(b) => b,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Base64 decode failed: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let json = match decrypt_payload(&encrypted, &key_bytes()) {
            Ok(j) => j,
            Err(e) => {
                res.status = 401;
                res.body = format!("{{\"error\":\"Decryption failed: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: InsertVoucherPayload = match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Invalid voucher payload: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let voucher = Voucher {
            code: payload.code.clone(),
            price: payload.price,
            time: payload.time,
            used: false,
        };

        let json_value = serde_json::to_string(&voucher).unwrap();
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(VOUCHERS_TABLE).unwrap();
            table.insert(payload.code.as_str(), json_value.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        println!("-> Voucher [code={}] inserted: {}", payload.code, json_value);

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"code\":\"{}\",\"voucher\":{}}}",
            payload.code, json_value
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
