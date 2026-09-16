use crate::network::server::Server;
use crate::utils::db::{RATES_TABLE, Rate, get_db};
use redb::{ReadableDatabase, ReadableTable};
use serde::Deserialize;

use super::auth::parse_admin_payload;

#[derive(Deserialize)]
struct CreateRatePayload {
    price: f64,
    time: u32,
}

#[derive(Deserialize)]
struct UpdateRatePayload {
    id: u32,
    price: f64,
    time: u32,
}

#[derive(Deserialize)]
struct DeleteRatePayload {
    id: u32,
}

pub fn handle_rates(server: &mut Server) {
    server.post("/api/admin/rates", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: CreateRatePayload = match serde_json::from_str(&json) {
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
        let mut next_id = 1;

        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(RATES_TABLE).unwrap();

            if let Ok(iter) = table.iter() {
                for item in iter {
                    if let Ok((k, _)) = item {
                        if k.value() >= next_id {
                            next_id = k.value() + 1;
                        }
                    }
                }
            }

            table.insert(next_id, json_value.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        crate::debug_println!("-> Rate [id={}] created: {}", next_id, json_value);

        res.status = 200;
        res.body =
            format!("{{\"ok\":true,\"id\":{},\"rate\":{}}}", next_id, json_value).into_bytes();
        res.content_type = String::from("application/json");
    });

    server.put("/api/admin/rates", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: UpdateRatePayload = match serde_json::from_str(&json) {
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

        {
            let check_txn = db.begin_read().unwrap();
            let table = check_txn.open_table(RATES_TABLE).unwrap();
            if table.get(payload.id).unwrap().is_none() {
                res.status = 404;
                res.body =
                    format!("{{\"error\":\"Rate id {} not found\"}}", payload.id).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        }

        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(RATES_TABLE).unwrap();
            table.insert(payload.id, json_value.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        crate::debug_println!(
            "-> Rate [id={}] upserted via PUT: {}",
            payload.id,
            json_value
        );

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"id\":{},\"rate\":{}}}",
            payload.id, json_value
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });

    server.delete("/api/admin/rates", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: DeleteRatePayload = match serde_json::from_str(&json) {
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
            let mut table = write_txn.open_table(RATES_TABLE).unwrap();
            if table.remove(payload.id).unwrap().is_some() {
                deleted = true;
            }
        }
        write_txn.commit().unwrap();

        crate::debug_println!("-> Rate [id={}] deleted: {}", payload.id, deleted);

        res.status = 200;
        res.body = format!(
            "{{\"ok\":true,\"id\":{},\"deleted\":{}}}",
            payload.id, deleted
        )
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
