pub mod auth;
pub mod coinslot_key;
pub mod dashboard;
pub mod qos;
pub mod rates;
pub mod users;
pub mod vouchers;
pub mod wifi;

use crate::network::server::Server;
use coinslot_key::handle_coinslot_key;
use dashboard::handle_dashboard;
use qos::handle_qos;
use rates::handle_rates;
use users::handle_users;
use vouchers::handle_vouchers;
use wifi::handle_wifi;

pub const LAN_SUBNET: &str = "10.0.";

pub fn key_bytes() -> [u8; 32] {
    let secret = std::env::var("LIBREFI_KEY").unwrap_or_else(|_| {
        eprintln!("[FATAL] LIBREFI_KEY environment variable is not set. Set it to a strong secret key.");
        std::process::exit(1);
    });
    if secret.len() < 16 {
        eprintln!("[FATAL] LIBREFI_KEY is too short (minimum 16 characters required).");
        std::process::exit(1);
    }
    let mut key = [0u8; 32];
    let src = secret.as_bytes();
    let len = src.len().min(32);
    key[..len].copy_from_slice(&src[..len]);
    key
}

pub fn admin(server: &mut Server) {
    handle_rates(server);
    handle_vouchers(server);
    handle_qos(server);
    handle_users(server);
    handle_wifi(server);
    handle_coinslot_key(server);
    handle_dashboard(server);
}
