use crate::network::server::Server;
use crate::utils::db::{CONFIG_TABLE, get_db};
use redb::ReadableDatabase;
use serde::{Deserialize, Serialize};

use super::{LAN_SUBNET, auth::parse_admin_payload};

#[derive(Serialize, Deserialize, Default)]
pub struct QosPayload {
    pub download: f64,
    pub upload: f64,
    #[serde(default)]
    pub global_download: f64,
    #[serde(default)]
    pub global_upload: f64,
}

pub fn apply_qos(payload: &QosPayload) -> Result<(), String> {
    let dl_kbps = (payload.download * 125.0).round() as u32;
    let ul_kbps = (payload.upload * 125.0).round() as u32;
    let dl_global_kbps = (payload.global_download * 125.0).round() as u32;
    let ul_global_kbps = (payload.global_upload * 125.0).round() as u32;

    let script = format!(
        r#"
mkdir -p /etc/nftables.d
cat << 'EOF' > /etc/nftables.d/10-rate-limits.nft
chain forward_qos_ingress {{
    type filter hook forward priority filter - 1; policy accept;
EOF

ip -o -f inet addr show | awk '/br-lan/ {{print $2, $4}}' | while read -r iface subnet; do
    dl_kbps={0}
    dl_global={2}
    if [ "$dl_kbps" -gt 0 ]; then
        meter_name=$(echo "$iface" | tr '.' '_')
        echo "    oifname \"$iface\" ip daddr $subnet meter client_dl_$meter_name {{ ip daddr limit rate over ${{dl_kbps}} kbytes/second }} drop" >> /etc/nftables.d/10-rate-limits.nft
    fi
    if [ "$dl_global" -gt 0 ]; then
        echo "    oifname \"$iface\" limit rate over ${{dl_global}} kbytes/second drop" >> /etc/nftables.d/10-rate-limits.nft
    fi
done

cat << 'EOF' >> /etc/nftables.d/10-rate-limits.nft
}}

chain forward_qos_egress {{
    type filter hook forward priority filter - 1; policy accept;
EOF

ip -o -f inet addr show | awk '/br-lan/ {{print $2, $4}}' | while read -r iface subnet; do
    ul_kbps={1}
    ul_global={3}
    if [ "$ul_kbps" -gt 0 ]; then
        meter_name=$(echo "$iface" | tr '.' '_')
        echo "    iifname \"$iface\" ip saddr $subnet meter client_ul_$meter_name {{ ip saddr limit rate over ${{ul_kbps}} kbytes/second }} drop" >> /etc/nftables.d/10-rate-limits.nft
    fi
    if [ "$ul_global" -gt 0 ]; then
        echo "    iifname \"$iface\" limit rate over ${{ul_global}} kbytes/second drop" >> /etc/nftables.d/10-rate-limits.nft
    fi
done

cat << 'EOF' >> /etc/nftables.d/10-rate-limits.nft
}}
EOF

if command -v uci >/dev/null 2>&1; then
    if uci show firewall | grep -q "/etc/nftables.d/10-rate-limits.nft"; then
        /etc/init.d/firewall restart
    else
        uci add firewall include
        uci set firewall.@include[-1].path='/etc/nftables.d/10-rate-limits.nft'
        uci set firewall.@include[-1].reload='1'
        uci commit firewall
        /etc/init.d/firewall restart
    fi
fi
"#,
        dl_kbps, ul_kbps, dl_global_kbps, ul_global_kbps
    );

    match std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "Failed to apply QoS: {}",
                    err.replace("\"", "\\\"")
                ));
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to execute sh: {}", e)),
    }
}

pub fn handle_qos(server: &mut Server) {
    server.post("http://localhost:8000/api/admin/qos", |req, res| {
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

        if let Err(e) = apply_qos(&payload) {
            res.status = 500;
            res.body = format!("{{\"error\":\"{}\"}}", e).into_bytes();
            res.content_type = String::from("application/json");
            return;
        }

        crate::debug_println!(
            "-> QoS updated: Download = {} Mbps, Upload = {} Mbps",
            payload.download,
            payload.upload
        );

        res.status = 200;
        res.body = b"{\"ok\":true}".to_vec();
        res.content_type = String::from("application/json");
    });

    server.get("http://localhost:8000/api/admin/qos", |req, res| {
        if req.ip.starts_with(LAN_SUBNET) {
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
