use std::error::Error;
use std::future::IntoFuture;
use std::time::Duration;
use url::Url;
use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, Controls};
use blimp_ground_ws_interface::MessageV2G;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let mut client = BlimpGroundWebsocketClient::new("ws://localhost:8080/");
    client.connect().await.expect("Failed to connect");
    client.send(MessageV2G::Controls(Controls{throttle: 0, elevation: 12, yaw: 0})).await;
    client.disconnect().await.expect("Failed to disconnect");
    sleep(Duration::from_millis(10000)).await;

    // let (shutdown_tx,  shutdown_rx) = oneshot::channel();
    // let mut client = tokio::spawn(websocket_loop("ws://localhost:8080/", shutdown_rx));

    // let result = tokio::select! {
    //     _ = tokio::signal::ctrl_c() => {
    //         shutdown_tx.send(()).unwrap();
    //         server.await.unwrap()
    //     }
    //     result = &mut server => {
    //         result.unwrap()
    //     }
    // };
    // match result.err() {
    //     None => {
    //         println!("quit successfully");
    //     }
    //     Some(error) => {
    //         println!("quit with error: {:?}", error);
    //     }
    // }
}