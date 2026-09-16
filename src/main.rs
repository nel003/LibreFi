mod network;
mod routes;
mod utils;
use network::server::Server;
use routes::index::index;

use crate::routes::{
    admin::admin, coin::coin, debug::debug, init::init, play_pause::play_pause, rates::rates,
    redeem::redeem, ws::ws_handler,
};
fn main() {
    let mut server = Server::new(80);
    utils::db::init_db("/etc/librefi/data.redb");
    let wan_iface = utils::setup_captive_portal::detect_wan_iface();
    utils::setup_captive_portal::setup_captive_portal("br-lan", &wan_iface, "10.0.0.1", "5353");
    utils::setup_captive_portal::authorize_active_users();
    utils::setup_captive_portal::restore_qos_settings();

    if std::env::var("LIBREFI_DEBUG").unwrap_or_default() == "true" {
        debug(&mut server);
    }

    index(&mut server);
    play_pause(&mut server);
    init(&mut server);
    admin(&mut server);
    rates(&mut server);
    redeem(&mut server);
    coin(&mut server);
    server.ws("/ws", ws_handler);

    server.run();
}
