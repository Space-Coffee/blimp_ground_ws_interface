use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Notify};
use tokio::time::timeout;
use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, BlimpGroundWebsocketServer, MessageV2G, VizInterest};

#[tokio::test]
async fn test_client_send() {
    let received = Arc::new(Mutex::new(None));
    let notify = Arc::new(Notify::new());

    let moved_received = received.clone();
    let moved_notify = notify.clone();

    let running = tokio::spawn(async move {
        let mut server = BlimpGroundWebsocketServer::new("localhost:9999");
        server.bind().await.expect("Failed address bind");
        server.run(move |message| {
            let value = moved_received.clone();
            let notify = moved_notify.clone();
            async move {
                *value.lock().await = Some(message);
                notify.notify_one();
            }
        }).await.expect("Server failed");
    });
    let mut client = BlimpGroundWebsocketClient::new("ws://localhost:9999/");
    let sent = MessageV2G::DeclareInterest(VizInterest{motors: true, servos: true, sensors: true});
    client.connect().await.expect("Failed to connect the client");
    client.send(sent).await.expect("Failed to send the client message");

    timeout(Duration::from_secs(1), notify.notified()).await.expect("Timed out waiting for client message");

    running.abort();
    assert_eq!(sent, received.lock().await.unwrap());
}