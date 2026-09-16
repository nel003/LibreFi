use crate::network::server::Server;
use crate::utils::db::{CONFIG_TABLE, USERS_TABLE, User, VOUCHERS_TABLE, Voucher, get_db};
use crate::utils::get_mac_from_ip::get_mac_from_ip;
use crate::utils::setup_captive_portal::run_sh_cmd;
use redb::ReadableDatabase;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
struct RedeemBody {
    code: String,
}

fn is_valid_mac(mac: &str) -> bool {
    let parts: Vec<&str> = mac.split(':').collect();
    parts.len() == 6
        && parts
            .iter()
            .all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

fn get_fail_count(mac: &str) -> (u32, u64) {
    let db = get_db();
    if let Ok(read_txn) = db.begin_read() {
        if let Ok(table) = read_txn.open_table(CONFIG_TABLE) {
            let key = format!("redeem_fail:{}", mac);
            if let Ok(Some(v)) = table.get(key.as_str()) {
                let parts: Vec<&str> = v.value().splitn(2, ':').collect();
                if parts.len() == 2 {
                    let count = parts[0].parse::<u32>().unwrap_or(0);
                    let ts = parts[1].parse::<u64>().unwrap_or(0);
                    return (count, ts);
                }
            }
        }
    }
    (0, 0)
}

fn set_fail_count(mac: &str, count: u32) {
    let db = get_db();
    if let Ok(write_txn) = db.begin_write() {
        if let Ok(mut table) = write_txn.open_table(CONFIG_TABLE) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let key = format!("redeem_fail:{}", mac);
            let val = format!("{}:{}", count, now);
            let _ = table.insert(key.as_str(), val.as_str());
        }
        let _ = write_txn.commit();
    }
}

fn clear_fail_count(mac: &str) {
    let db = get_db();
    if let Ok(write_txn) = db.begin_write() {
        if let Ok(mut table) = write_txn.open_table(CONFIG_TABLE) {
            let key = format!("redeem_fail:{}", mac);
            let _ = table.remove(key.as_str());
        }
        let _ = write_txn.commit();
    }
}

pub fn redeem(server: &mut Server) {
    server.post("/api/redeem", |req, res| {
        if req.body.len() > 65536 {
            res.status = 413;
            res.body = b"{\"error\":\"Payload too large\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        }

        let Some(mac) = get_mac_from_ip(&req.ip) else {
            res.status = 403;
            res.body = b"{\"error\":\"Device not recognized\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        };

        if !is_valid_mac(&mac) {
            res.status = 403;
            res.body = b"{\"error\":\"Device not recognized\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        }

        // Rate limiting: 5 failures → 60-second lockout
        let (fail_count, last_fail_ts) = get_fail_count(&mac);
        if fail_count >= 5 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if now.saturating_sub(last_fail_ts) < 60 {
                res.status = 429;
                res.body =
                    b"{\"error\":\"Too many failed attempts. Try again in 60 seconds.\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            } else {
                clear_fail_count(&mac);
            }
        }

        let body_str = match String::from_utf8(req.body.clone()) {
            Ok(s) => s,
            Err(_) => {
                res.status = 400;
                res.body = b"{\"error\":\"Body is not valid UTF-8\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let body: RedeemBody = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                res.status = 400;
                res.body = format!(
                    "{{\"error\":\"Expected {{\\\"code\\\":\\\"...\\\"}}: {}\"}}",
                    e
                )
                .into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let db = get_db();

        let read_txn = db.begin_read().unwrap();
        let vtable = read_txn.open_table(VOUCHERS_TABLE).unwrap();

        let voucher: Voucher = match vtable.get(body.code.as_str()).unwrap() {
            None => {
                set_fail_count(&mac, fail_count + 1);
                res.status = 404;
                res.body = b"{\"error\":\"Voucher not found\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
            Some(record) => match serde_json::from_str(record.value()) {
                Ok(v) => v,
                Err(_) => {
                    res.status = 500;
                    res.body = b"{\"error\":\"Corrupted voucher record\"}".to_vec();
                    res.content_type = String::from("application/json");
                    return;
                }
            },
        };

        if voucher.used {
            set_fail_count(&mac, fail_count + 1);
            res.status = 409;
            res.body = b"{\"error\":\"Voucher already used\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        }

        let utable = read_txn.open_table(USERS_TABLE).unwrap();

        let mut user: User = match utable.get(mac.as_str()).unwrap() {
            None => {
                res.status = 404;
                res.body = b"{\"error\":\"User not found\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
            Some(record) => match serde_json::from_str(record.value()) {
                Ok(u) => u,
                Err(_) => {
                    res.status = 500;
                    res.body = b"{\"error\":\"Corrupted user record\"}".to_vec();
                    res.content_type = String::from("application/json");
                    return;
                }
            },
        };

        drop(utable);
        drop(vtable);
        drop(read_txn);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        if user.expires_on == 0 || user.expires_on < now {
            user.expires_on = now + voucher.time;
        } else {
            user.expires_on += voucher.time;
        }

        let write_txn = db.begin_write().unwrap();
        {
            let mut wtable = write_txn.open_table(VOUCHERS_TABLE).unwrap();
            wtable.remove(body.code.as_str()).unwrap();

            let mut utable = write_txn.open_table(USERS_TABLE).unwrap();
            let uj = serde_json::to_string(&user).unwrap();
            utable.insert(mac.as_str(), uj.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        clear_fail_count(&mac);

        crate::debug_println!(
            "-> Voucher redeemed by MAC={} — expires_on={}",
            mac,
            user.expires_on
        );

        if !user.paused {
            let diff = user.expires_on.saturating_sub(now);
            if diff > 0 {
                let cmd = format!(
                    "nft add element inet librefi allowed_macs {{ {} timeout {}s }}",
                    mac, diff
                );
                let _ = run_sh_cmd(&cmd, true);
            }
        }

        res.status = 200;
        res.body = serde_json::json!({
            "ok": true,
            "expires_on": user.expires_on,
            "now": now,
            "added_seconds": voucher.time,
        })
        .to_string()
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
