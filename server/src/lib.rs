//pub mod world;

use std::net::{IpAddr, SocketAddr};

mod agent;
mod extension;
mod world;

mod savegame;
#[cfg(test)]
mod tests;

async fn a() {
    let endpoint =
        quinn::Endpoint::client(SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 0)).unwrap();

    let connecing = endpoint
        .connect(
            SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 12042),
            "localhost",
        )
        .unwrap();
    let (connection, zrtt) = connecing.into_0rtt().unwrap();
    zrtt.await;
    connection.open_bi();
}
