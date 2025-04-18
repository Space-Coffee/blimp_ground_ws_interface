use std::future::IntoFuture;
use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, BlimpGroundWebsocketServer, Controls};
use blimp_ground_ws_interface::MessageV2G;

async fn handler(message: MessageV2G) {
    println!("{:?}", message)
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