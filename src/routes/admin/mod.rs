pub mod auth;
pub mod qos;
pub mod rates;
pub mod users;
pub mod vouchers;
pub mod wifi;

use crate::network::server::Server;
pub use qos::QosPayload;
use qos::handle_qos;
use rates::handle_rates;
use users::handle_users;
use vouchers::handle_vouchers;
use wifi::handle_wifi;

pub fn key_bytes() -> [u8; 32] {
    let mut key = [0u8; 32];
    let secret = std::env::var("LIBREFI_KEY").unwrap_or_else(|_| "keykey".to_string());
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
}
