use std::future::IntoFuture;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, BlimpGroundWebsocketServer, BlimpGroundWebsocketStreamPair, Controls};

async fn handler(pair: BlimpGroundWebsocketStreamPair<TcpStream>) {
    println!("connected");
    sleep(Duration::from_millis(5000)).await;
    println!("disconnecting")
}
#[tokio::main]
async fn main() {
    let mut server = BlimpGroundWebsocketServer::new("localhost:8080");
    server.bind().await.unwrap();
    tokio::select! {
        _ = server.run(handler) => {}
        _ = tokio::signal::ctrl_c() => {}
    }
    println!("Stopped serving")
}