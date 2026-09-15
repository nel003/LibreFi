use crate::network::server::Request;
use base64::Engine;
use serde::Deserialize;

use super::{key_bytes, LAN_SUBNET};
use crate::utils::crypto::decrypt_payload;

#[derive(Deserialize)]
pub struct AdminBody {
    pub payload: String,
}

pub fn parse_admin_payload(req: &Request) -> Result<String, (u16, Vec<u8>)> {
    if req.ip.starts_with(LAN_SUBNET) {
        return Err((
            403,
            b"{\"error\":\"Admin access denied from LAN\"}".to_vec(),
        ));
    }

    let payload_str = if req.body.is_empty() && !req.query.is_empty() {
        let mut found = String::new();
        for param in req.query.split('&') {
            if param.starts_with("payload=") {
                found = param["payload=".len()..].to_string();
                break;
            }
        }
        
        if found.is_empty() {
             return Err((400, b"{\"error\":\"Missing payload in body or query\"}".to_vec()));
        }

        found.replace("%2B", "+").replace("%2F", "/").replace("%3D", "=")
    } else {
        let body_str = match String::from_utf8(req.body.clone()) {
            Ok(s) => s,
            Err(_) => return Err((400, b"{\"error\":\"Body is not valid UTF-8\"}".to_vec())),
        };

        let admin_body: AdminBody = match serde_json::from_str(&body_str) {
            Ok(b) => b,
            Err(e) => {
                return Err((
                    400,
                    format!(
                        "{{\"error\":\"Expected {{\\\"payload\\\":\\\"...\\\"}}: {}\"}}",
                        e
                    )
                    .into_bytes(),
                ));
            }
        };
        admin_body.payload
    };

    let encrypted = match base64::engine::general_purpose::STANDARD.decode(&payload_str) {
        Ok(b) => b,
        Err(e) => {
            return Err((
                400,
                format!("{{\"error\":\"Base64 decode failed: {}\"}}", e).into_bytes(),
            ));
        }
    };

    match decrypt_payload(&encrypted, &key_bytes()) {
        Ok(j) => Ok(j),
        Err(_) => Err((401, b"{\"error\":\"Unauthorized\"}".to_vec())),
    }
}
