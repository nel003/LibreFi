mod network;
mod routes;
mod utils;
use network::server::Server;
use routes::index::index;

use crate::routes::{admin::admin, debug::debug, init::init, play_pause::play_pause, rates::rates, redeem::redeem, status::status};
fn main() {
    let mut server = Server::new(80);
    utils::db::init_db("data.redb");
    
    index(&mut server);
    status(&mut server);
    play_pause(&mut server);
    debug(&mut server);
    init(&mut server);
    admin(&mut server);
    rates(&mut server);
    redeem(&mut server);

    server.run();
}
