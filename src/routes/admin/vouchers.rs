use crate::network::server::Server;
use crate::utils::db::{get_db, Voucher, VOUCHERS_TABLE};
use serde::Deserialize;

use super::auth::parse_admin_payload;

#[derive(Deserialize)]
struct InsertVoucherPayload {
    code: String,
    price: f64,
    time: u32,
}

pub fn handle_vouchers(server: &mut Server) {
    server.post("/admin/vouchers", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
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
