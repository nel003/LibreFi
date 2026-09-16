use super::auth::parse_admin_payload;
use crate::network::server::Server;
use crate::utils::db::{SALES_TABLE, USERS_TABLE, User, get_db};
use redb::{ReadableDatabase, ReadableTable};
use std::process::Command;
use std::time::SystemTime;

pub fn handle_dashboard(server: &mut Server) {
    server.get("/api/admin/dashboard", |req, res| {
        let _json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let output = Command::new("sh")
            .arg("-c")
            .arg(r#"
                cpu=$(awk '/^cpu /{t=0; for(i=2;i<=NF;i++)t+=$i; print t,$5; exit}' /proc/stat | { read t1 i1; sleep 1; awk -v t1="$t1" -v i1="$i1" '/^cpu /{t=0; for(i=2;i<=NF;i++)t+=$i; printf "%.1f", 100*(1-(($5-i1)/(t-t1))); exit}' /proc/stat; })
                awk -v cpu="${cpu:-0.0}" '/MemTotal/ {t=$2} /MemAvailable/ {a=$2} END {printf "{\n  \"cpu_used_pct\": %s,\n  \"ram_used_pct\": %.1f,\n  \"ram_free_pct\": %.1f,\n", cpu, ((t-a)/t)*100, (a/t)*100}' /proc/meminfo && df /overlay | awk 'NR==2 {gsub("%","",$5); printf "  \"storage_used_pct\": %d,\n  \"storage_free_pct\": %d\n}\n", $5, 100-$5}'
            "#)
            .output()
            .expect("Failed to execute shell command");

        let mut sys_stats: serde_json::Value = if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            serde_json::from_str(&stdout).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let db = get_db();
        let mut total_coins = 0;
        let mut total_users = 0;
        let mut active_users = 0;

        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        if let Ok(read_txn) = db.begin_read() {
            if let Ok(sales_table) = read_txn.open_table(SALES_TABLE) {
                for result in sales_table.iter().unwrap() {
                    if let Ok((_k, v)) = result {
                        total_coins += v.value();
                    }
                }
            }

            if let Ok(users_table) = read_txn.open_table(USERS_TABLE) {
                for result in users_table.iter().unwrap() {
                    if let Ok((_k, v)) = result {
                        total_users += 1;
                        if let Ok(user) = serde_json::from_str::<User>(v.value()) {
                            if !user.paused && user.expires_on > now {
                                active_users += 1;
                            }
                        }
                    }
                }
            }
        }

        if let Some(obj) = sys_stats.as_object_mut() {
            obj.insert("total_coins".to_string(), serde_json::json!(total_coins));
            obj.insert("total_users".to_string(), serde_json::json!(total_users));
            obj.insert("total_active_users".to_string(), serde_json::json!(active_users));
        }

        res.status = 200;
        res.body = serde_json::to_vec(&sys_stats).unwrap();
        res.content_type = String::from("application/json");
    });
}
