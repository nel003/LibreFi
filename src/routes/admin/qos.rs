use crate::network::server::Server;
use crate::utils::db::{CONFIG_TABLE, get_db};
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};

use super::auth::parse_admin_payload;

#[derive(Serialize, Deserialize)]
pub struct QosPayload {
    pub download: f64,
    pub upload: f64,
}

pub fn handle_qos(server: &mut Server) {
    server.post("/admin/qos", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: QosPayload = match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Invalid qos payload: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        // Save QoS to database
        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(CONFIG_TABLE).unwrap();
            table.insert("qos", json.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        // Convert Mbps to kbps (1 Mbps = 1000 kbps for SQM)
        let download_kbps = (payload.download * 1000.0).round() as u32;
        let upload_kbps = (payload.upload * 1000.0).round() as u32;

        let script = format!(
            "if [ ! -f /etc/init.d/sqm ]; then\n\
                 if command -v apk >/dev/null 2>&1; then\n\
                     apk update && apk add sqm-scripts\n\
                 elif command -v opkg >/dev/null 2>&1; then\n\
                     opkg update && opkg install sqm-scripts\n\
                 else\n\
                     echo \"No supported package manager found\" >&2\n\
                     exit 1\n\
                 fi\n\
             fi\n\
             uci set sqm.openfi=queue\n\
             uci set sqm.openfi.enabled='1'\n\
             uci set sqm.openfi.interface='br-lan'\n\
             uci set sqm.openfi.download='{}'\n\
             uci set sqm.openfi.upload='{}'\n\
             uci set sqm.openfi.qdisc='cake'\n\
             uci set sqm.openfi.script='piece_of_cake.qos'\n\
             uci commit sqm\n\
             mkdir -p /var/lock\n\
             /etc/init.d/sqm enable\n\
             /etc/init.d/sqm restart",
            download_kbps, upload_kbps
        );

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&script)
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    res.status = 500;
                    res.body = format!(
                        "{{\"error\":\"Failed to apply QoS: {}\"}}",
                        err.replace("\"", "\\\"")
                    )
                    .into_bytes();
                    res.content_type = String::from("application/json");
                    return;
                }
            }
            Err(e) => {
                res.status = 500;
                res.body = format!("{{\"error\":\"Failed to execute sh: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        }

        println!(
            "-> QoS updated: Download = {} Mbps, Upload = {} Mbps",
            payload.download, payload.upload
        );

        res.status = 200;
        res.body = b"{\"ok\":true}".to_vec();
        res.content_type = String::from("application/json");
    });

    server.get("/admin/qos", |req, res| {
        if req.ip.starts_with("10.0.0.") {
            res.status = 403;
            res.body = b"{\"error\":\"Admin access denied from LAN\"}".to_vec();
            res.content_type = String::from("application/json");
            return;
        }

        let db = get_db();
        if let Ok(read_txn) = db.begin_read() {
            if let Ok(table) = read_txn.open_table(CONFIG_TABLE) {
                if let Ok(Some(value)) = table.get("qos") {
                    res.status = 200;
                    res.body = value.value().as_bytes().to_vec();
                    res.content_type = String::from("application/json");
                    return;
                }
            }
        }

        res.status = 200;
        res.body = b"{\"download\":0.0,\"upload\":0.0}".to_vec();
        res.content_type = String::from("application/json");
    });
}
