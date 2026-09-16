use crate::network::server::Server;
use crate::utils::db::{VOUCHERS_TABLE, Voucher, get_db};
use serde::Deserialize;

use super::auth::parse_admin_payload;

#[derive(Deserialize)]
struct InsertVoucherPayload {
    code: String,
    price: f64,
    time: u32,
}

#[derive(Deserialize)]
struct DeleteVoucherPayload {
    code: String,
}

use redb::{ReadableDatabase, ReadableTable};
use serde::Serialize;

#[derive(Deserialize, Default)]
struct GetVouchersPayload {
    page: Option<usize>,
}

#[derive(Serialize)]
struct VouchersListResponse {
    ok: bool,
    vouchers: Vec<Voucher>,
    total_pages: usize,
    current_page: usize,
}

fn generate_random_code() -> String {
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut buf = [0u8; 6];

    #[cfg(unix)]
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = f.read_exact(&mut buf);
    }

    #[cfg(not(unix))]
    {
        // Fallback: mix of multiple timestamps
        let t1 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let bytes = t1.to_ne_bytes();
        buf.copy_from_slice(&bytes[..6]);
    }

    buf.iter()
        .map(|b| chars[(*b as usize) % chars.len()] as char)
        .collect()
}

pub fn handle_vouchers(server: &mut Server) {
    server.get("/api/admin/vouchers", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let get_payload: GetVouchersPayload = serde_json::from_str(&json).unwrap_or_default();
        let page = get_payload.page.unwrap_or(1).max(1);
        let limit = 10;

        let db = get_db();
        let mut all_vouchers = Vec::new();

        if let Ok(read_txn) = db.begin_read() {
            if let Ok(table) = read_txn.open_table(VOUCHERS_TABLE) {
                if let Ok(iter) = table.iter() {
                    for item in iter {
                        if let Ok((_key, value)) = item {
                            if let Ok(voucher) = serde_json::from_str::<Voucher>(value.value()) {
                                all_vouchers.push(voucher);
                            }
                        }
                    }
                }
            }
        }

        all_vouchers.sort_by(|a, b| b.created_at.unwrap_or(0).cmp(&a.created_at.unwrap_or(0)));

        let total_vouchers = all_vouchers.len();
        let total_pages = if total_vouchers == 0 {
            1
        } else {
            (total_vouchers as f64 / limit as f64).ceil() as usize
        };

        let start = (page - 1) * limit;
        let end = (start + limit).min(total_vouchers);
        let vouchers = if start < total_vouchers {
            all_vouchers[start..end].to_vec()
        } else {
            Vec::new()
        };

        let resp = VouchersListResponse {
            ok: true,
            vouchers,
            total_pages,
            current_page: page,
        };
        res.status = 200;
        res.body = serde_json::to_vec(&resp).unwrap();
        res.content_type = String::from("application/json");
    });
    server.post("/api/admin/vouchers", |req, res| {
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

        let mut code = payload.code.clone();
        if code == "RANDOM" {
            code = generate_random_code();
        }

        let voucher = Voucher {
            code: code.clone(),
            price: payload.price,
            time: payload.time,
            used: false,
            created_at: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        };

        let json_value = serde_json::to_string(&voucher).unwrap();
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(VOUCHERS_TABLE).unwrap();
            table.insert(code.as_str(), json_value.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        crate::debug_println!("-> Voucher [code={}] inserted: {}", code, json_value);

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"code\":\"{}\",\"voucher\":{}}}",
            code, json_value
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });

    server.delete("/api/admin/vouchers", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: DeleteVoucherPayload = match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Invalid delete payload: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        let mut deleted = false;
        {
            let mut table = write_txn.open_table(VOUCHERS_TABLE).unwrap();
            if table.remove(payload.code.as_str()).unwrap().is_some() {
                deleted = true;
            }
        }
        write_txn.commit().unwrap();

        crate::debug_println!("-> Voucher [code={}] deleted: {}", payload.code, deleted);

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"code\":\"{}\",\"deleted\":{}}}",
            payload.code, deleted
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
