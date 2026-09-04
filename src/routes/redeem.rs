use crate::network::server::Server;
use crate::utils::db::{get_db, User, Voucher, USERS_TABLE, VOUCHERS_TABLE};
use crate::utils::get_mac_from_ip::get_mac_from_ip;
use crate::utils::setup_captive_portal::run_sh_cmd;
use redb::ReadableDatabase;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH}; 

#[derive(Deserialize)]
struct RedeemBody {
    code: String,
}

pub fn redeem(server: &mut Server) {
    server.post("/redeem", |req, res| {
        let Some(mac) = get_mac_from_ip(&req.ip) else {
            res.status = 403;
            res.body = b"{\"error\":\"Device not recognized\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        };

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
                res.body = format!("{{\"error\":\"Expected {{\\\"code\\\":\\\"...\\\"}}: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        let db = get_db();

        let read_txn = db.begin_read().unwrap();
        let vtable = read_txn.open_table(VOUCHERS_TABLE).unwrap();

        let voucher: Voucher = match vtable.get(body.code.as_str()).unwrap() {
            None => {
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

        let used_voucher = Voucher { used: true, ..voucher };

        let write_txn = db.begin_write().unwrap();
        {
            let mut wtable = write_txn.open_table(VOUCHERS_TABLE).unwrap();
            let vj = serde_json::to_string(&used_voucher).unwrap();
            wtable.insert(body.code.as_str(), vj.as_str()).unwrap();

            let mut utable = write_txn.open_table(USERS_TABLE).unwrap();
            let uj = serde_json::to_string(&user).unwrap();
            utable.insert(mac.as_str(), uj.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        println!(
            "-> Voucher [{}] redeemed by MAC={} — expires_on={}",
            body.code, mac, user.expires_on
        );

        if !user.paused {
            let diff = user.expires_on.saturating_sub(now);
            if diff > 0 {
                let cmd = if crate::utils::cmds::has_command("nft") {
                    format!("nft add element inet fw4 allowed_macs {{ {} timeout {}s }}", mac, diff)
                } else {
                    format!("ipset add allowed_macs {} timeout {} -exist", mac, diff)
                };
                let _ = run_sh_cmd(&cmd, true);
            }
        }

        res.status = 200;
        res.body = serde_json::json!({
            "ok": true,
            "expires_on": user.expires_on,
            "now": now,
            "added_seconds": used_voucher.time,
        })
        .to_string()
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
