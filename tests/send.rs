use std::sync::Arc;
use futures_util::SinkExt;
use tokio::sync::{Mutex, oneshot};
use tokio::time::{timeout, Duration};
use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, BlimpGroundWebsocketServer, MessageV2G, VizInterest};

#[tokio::test]
async fn test_client_send() {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let server = {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut server = BlimpGroundWebsocketServer::new("localhost:9999");
            server.bind().await.expect("Failed address bind");
            server.run(move |message| {
                let tx = tx.clone();
                async move {
                    tx.lock().await.take().expect("Tx already taken").send(message).expect("Failed to send");
                }
            }).await.expect("Server failed");
        })
    };

    let mut client = BlimpGroundWebsocketClient::new("ws://localhost:9999/");
    let sent = MessageV2G::DeclareInterest(VizInterest { motors: true, servos: true, sensors: true });

    client.connect().await.expect("Failed to connect the client");
    client.send(sent.clone()).await.expect("Failed to send the client message");

    let received = timeout(Duration::from_secs(1), rx).await.expect("Timed out waiting for client message").expect("Receive failed");

    server.abort();

    assert_eq!(sent, received);
}
