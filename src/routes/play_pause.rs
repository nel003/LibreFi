use crate::network::server::Server;
use crate::utils::db::{USERS_TABLE, User, get_db};
use crate::utils::get_mac_from_ip::get_mac_from_ip;
use redb::ReadableDatabase;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn play_pause(server: &mut Server) {
    server.post("http://localhost:8000/api/play_pause", |req, res| {
        let Some(mac) = get_mac_from_ip(&req.ip) else {
            res.status = 403;
            res.body = b"{\"error\":\"Device not recognized\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        };

        let db = get_db();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;

        // Read current user
        let read_txn = db.begin_read().unwrap();
        let table = read_txn.open_table(USERS_TABLE).unwrap();

        let mut user: User = match table.get(mac.as_str()).unwrap() {
            None => {
                res.status = 404;
                res.body = b"{\"error\":\"User not found\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
            Some(r) => match serde_json::from_str(r.value()) {
                Ok(u) => u,
                Err(_) => {
                    res.status = 500;
                    res.body = b"{\"error\":\"Corrupted user record\"}".to_vec();
                    res.content_type = String::from("application/json");
                    return;
                }
            },
        };

        drop(table);
        drop(read_txn);

        let action: &str;
        const MAX_PAUSES_PER_DAY: u8 = 3;

        if user.paused {
            let remaining = user.expires_on.saturating_sub(user.paused_on);
            user.expires_on = now + remaining;
            user.paused = false;
            user.paused_on = 0;
            action = "play";
        } else {
            let today = now / 86400;

            if user.pause_day != today {
                user.pause_attempts = 0;
                user.pause_day = today;
            }

            if user.pause_attempts >= MAX_PAUSES_PER_DAY {
                res.status = 429;
                res.body = format!(
                    "{{\"error\":\"Daily pause limit reached ({}/{})\",\"pause_attempts\":{},\"resets_in\":{}}}",
                    user.pause_attempts,
                    MAX_PAUSES_PER_DAY,
                    user.pause_attempts,
                    86400 - (now % 86400),  // seconds until midnight UTC
                )
                .into_bytes();
                res.content_type = String::from("application/json");
                return;
            }

            user.paused_on = now;
            user.paused = true;
            user.pause_attempts += 1;
            action = "pause";
        }

        // Write updated user
        let write_txn = db.begin_write().unwrap();
        {
            let mut wtable = write_txn.open_table(USERS_TABLE).unwrap();
            let json = serde_json::to_string(&user).unwrap();
            wtable.insert(mac.as_str(), json.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        crate::debug_println!(
            "-> [{}] MAC={} paused={} expires_on={} pause_attempts={}",
            action, mac, user.paused, user.expires_on, user.pause_attempts
        );

        if user.paused {
            let cmd = format!("nft delete element inet librefi allowed_macs {{ {} }}", mac);
            let _ = crate::utils::setup_captive_portal::run_sh_cmd(&cmd, true);
        } else {
            let diff = user.expires_on.saturating_sub(now);
            if diff > 0 {
                let cmd = format!("nft add element inet librefi allowed_macs {{ {} timeout {}s }}", mac, diff);
                let _ = crate::utils::setup_captive_portal::run_sh_cmd(&cmd, true);
            }
        }

        res.status = 200;
        res.body = serde_json::json!({
            "action": action,
            "paused": user.paused,
            "paused_on": user.paused_on,
            "expires_on": user.expires_on,
            "pause_attempts": user.pause_attempts,
            "now": now,
        })
        .to_string()
        .into_bytes();
        res.content_type = String::from("application/json");
    });
}
