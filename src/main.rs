use std::future::IntoFuture;
use blimp_ground_ws_interface::websocket_loop;
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() {
    let (shutdown_tx,  shutdown_rx) = oneshot::channel();
    let (message_tx, message_rx) = mpsc::channel(8);
    let mut server = tokio::spawn(websocket_loop("ws://localhost:8080/", shutdown_rx, message_rx));


    let a = tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            shutdown_tx.send(()).unwrap();
            server.await.unwrap()
        }
        result = &mut server => {
            result.unwrap()
        }
    };

    println!("Server shutdown");
}