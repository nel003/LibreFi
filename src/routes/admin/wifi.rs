use crate::network::server::Server;
use crate::utils::db::{CONFIG_TABLE, get_db};
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};

use super::auth::parse_admin_payload;

fn is_valid_ssid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.chars()
            .all(|c| c.is_alphanumeric() || " _-.!@#&".contains(c))
}

fn is_valid_wifi_key(s: &str) -> bool {
    s.is_empty()
        || (s.len() >= 8
            && s.len() <= 63
            && s.chars().all(|c| c.is_ascii() && !c.is_ascii_control()))
}

#[derive(Serialize, Deserialize)]
pub struct WifiPayload {
    pub ssid_2g: String,
    pub key_2g: String,
    pub disabled_2g: bool,
    pub ssid_5g: String,
    pub key_5g: String,
    pub disabled_5g: bool,
}

pub fn handle_wifi(server: &mut Server) {
    server.post("/api/admin/wifi", |req, res| {
        let json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let payload: WifiPayload = match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(e) => {
                res.status = 400;
                res.body = format!("{{\"error\":\"Invalid wifi payload: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        };

        if !payload.disabled_2g {
            if !is_valid_ssid(&payload.ssid_2g) {
                res.status = 400;
                res.body = b"{\"error\":\"Invalid 2.4GHz SSID: only alphanumeric and _-.!@#& allowed, max 32 chars\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
            if !is_valid_wifi_key(&payload.key_2g) {
                res.status = 400;
                res.body = b"{\"error\":\"Invalid 2.4GHz password: must be 8-63 ASCII chars or empty\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
        }

        if !payload.disabled_5g {
            if !is_valid_ssid(&payload.ssid_5g) {
                res.status = 400;
                res.body = b"{\"error\":\"Invalid 5GHz SSID: only alphanumeric and _-.!@#& allowed, max 32 chars\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
            if !is_valid_wifi_key(&payload.key_5g) {
                res.status = 400;
                res.body = b"{\"error\":\"Invalid 5GHz password: must be 8-63 ASCII chars or empty\"}".to_vec();
                res.content_type = String::from("application/json");
                return;
            }
        }

        let db = get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(CONFIG_TABLE).unwrap();
            table.insert("wifi", json.as_str()).unwrap();
        }
        write_txn.commit().unwrap();

        let disabled_2g_str = if payload.disabled_2g { "1" } else { "0" };
        let disabled_5g_str = if payload.disabled_5g { "1" } else { "0" };

        let enc_2g = if payload.key_2g.is_empty() {
            "none"
        } else {
            "psk2"
        };
        let enc_5g = if payload.key_5g.is_empty() {
            "none"
        } else {
            "psk2"
        };

        let key_cmd_2g = if payload.key_2g.is_empty() {
            "".to_string()
        } else {
            format!(
                "uci set wireless.librefi_2g.key='{}'",
                payload.key_2g.replace("'", "'\\''")
            )
        };

        let key_cmd_5g = if payload.key_5g.is_empty() {
            "".to_string()
        } else {
            format!(
                "uci set wireless.librefi_5g.key='{}'",
                payload.key_5g.replace("'", "'\\''")
            )
        };

        let script = format!(
            "while true; do\n\
                 deleted=0\n\
                 for iface in $(uci show wireless | grep '=wifi-iface' | cut -d'=' -f1); do\n\
                     if [ \"$(uci -q get ${{iface}}.mode)\" = \"ap\" ]; then\n\
                         uci delete ${{iface}}\n\
                         deleted=1\n\
                         break\n\
                     fi\n\
                 done\n\
                 if [ \"$deleted\" -eq 0 ]; then break; fi\n\
             done\n\
             \n\
             uci set wireless.librefi_2g=wifi-iface\n\
             uci set wireless.librefi_2g.device='radio0'\n\
             uci set wireless.librefi_2g.mode='ap'\n\
             uci set wireless.librefi_2g.network='lan'\n\
             uci set wireless.librefi_2g.ssid='{ssid_2g}'\n\
             {key_cmd_2g}\n\
             uci set wireless.librefi_2g.encryption='{enc_2g}'\n\
             uci set wireless.librefi_2g.disabled='{disabled_2g}'\n\
             \n\
             uci set wireless.librefi_5g=wifi-iface\n\
             uci set wireless.librefi_5g.device='radio1'\n\
             uci set wireless.librefi_5g.mode='ap'\n\
             uci set wireless.librefi_5g.network='lan'\n\
             uci set wireless.librefi_5g.ssid='{ssid_5g}'\n\
             {key_cmd_5g}\n\
             uci set wireless.librefi_5g.encryption='{enc_5g}'\n\
             uci set wireless.librefi_5g.disabled='{disabled_5g}'\n\
             \n\
             uci commit wireless\n\
             wifi reload",
            ssid_2g = payload.ssid_2g.replace("'", "'\\''"),
            key_cmd_2g = key_cmd_2g,
            enc_2g = enc_2g,
            disabled_2g = disabled_2g_str,
            ssid_5g = payload.ssid_5g.replace("'", "'\\''"),
            key_cmd_5g = key_cmd_5g,
            enc_5g = enc_5g,
            disabled_5g = disabled_5g_str
        );

        match std::process::Command::new("sh")
            .arg("-c")
            .arg(&script)
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    crate::debug_println!("-> Wifi script stderr (may be ignored): {}", err);
                }
            }
            Err(e) => {
                res.status = 500;
                res.body = format!("{{\"error\":\"Failed to execute sh: {}\"}}", e).into_bytes();
                res.content_type = String::from("application/json");
                return;
            }
        }

        crate::debug_println!("-> WiFi configuration updated successfully");

        res.status = 200;
        res.body = b"{\"ok\":true}".to_vec();
        res.content_type = String::from("application/json");
    });

    server.get("/api/admin/wifi", |req, res| {
        let _json = match parse_admin_payload(req) {
            Ok(j) => j,
            Err((status, body)) => {
                res.status = status;
                res.body = body;
                res.content_type = String::from("application/json");
                return;
            }
        };

        let supports_2g = std::process::Command::new("sh")
            .arg("-c")
            .arg("iw phy | grep -q '2412 MHz'")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let supports_5g = std::process::Command::new("sh")
            .arg("-c")
            .arg("iw phy | grep -q '5180 MHz'")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let mut current_payload = WifiPayload {
            ssid_2g: "openfi-2g".to_string(),
            key_2g: "".to_string(),
            disabled_2g: false,
            ssid_5g: "openfi-5g".to_string(),
            key_5g: "".to_string(),
            disabled_5g: false,
        };

        let db = get_db();
        if let Ok(read_txn) = db.begin_read() {
            if let Ok(table) = read_txn.open_table(CONFIG_TABLE) {
                if let Ok(Some(value)) = table.get("wifi") {
                    if let Ok(parsed) = serde_json::from_str::<WifiPayload>(value.value()) {
                        current_payload = parsed;
                    }
                }
            }
        }

        #[derive(Serialize)]
        struct WifiResponse {
            ssid_2g: String,
            key_2g: String,
            disabled_2g: bool,
            ssid_5g: String,
            key_5g: String,
            disabled_5g: bool,
            supports_2g: bool,
            supports_5g: bool,
        }

        let resp = WifiResponse {
            ssid_2g: current_payload.ssid_2g,
            key_2g: current_payload.key_2g,
            disabled_2g: current_payload.disabled_2g,
            ssid_5g: current_payload.ssid_5g,
            key_5g: current_payload.key_5g,
            disabled_5g: current_payload.disabled_5g,
            supports_2g,
            supports_5g,
        };

        res.status = 200;
        res.body = serde_json::to_vec(&resp).unwrap();
        res.content_type = String::from("application/json");
    });
}
