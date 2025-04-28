use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{Mutex, oneshot};
use tokio::time::{sleep, timeout, Duration};
use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, BlimpGroundWebsocketServer, MessageG2V, MessageV2G, VizInterest};

const CLIENT_MESSAGE: MessageV2G = MessageV2G::DeclareInterest(VizInterest { motors: true, servos: true, sensors: true });
#[tokio::test]
async fn test_client_send() {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let server = {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut server = BlimpGroundWebsocketServer::new("localhost:9999");
            server.bind().await.expect("Failed address bind");
            server.run(move |mut pair| {
                let tx = tx.clone();
                async move {
                    let message= pair.recv::<MessageV2G>().await.expect("Failed to receive client message");
                    tx.lock().await.take().expect("Tx already taken").send(message).expect("Failed to send");
                }
            }).await.expect("Server failed");
        })
    };

    let mut client = BlimpGroundWebsocketClient::new("ws://localhost:9999/");

    client.connect().await.expect("Failed to connect the client");
    client.send(CLIENT_MESSAGE).await.expect("Failed to send the client message");

    let received = timeout(Duration::from_secs(1), rx).await.expect("Timed out waiting for client message").expect("Receive failed");

    server.abort();

    assert_eq!(CLIENT_MESSAGE, received);
}

const SERVER_MESSAGE: MessageG2V = MessageG2V::MotorSpeed { id: 0, speed: 0 };
#[tokio::test]
async fn test_server_send() {
    let server = {
        tokio::spawn(async move {
            let mut server = BlimpGroundWebsocketServer::new("localhost:9998");
            server.bind().await.expect("Failed address bind");
            server.run(|mut pair| {
                async move {
                    pair.send(SERVER_MESSAGE).await.expect("Failed to send server message");
                    pair.close().await.expect("Failed to close");
                }
            }).await.expect("Server failed");
        })
    };

    let mut client = BlimpGroundWebsocketClient::new("ws://localhost:9998/");

    client.connect().await.expect("Failed to connect the client");

    let received = timeout(Duration::from_secs(1), client.recv()).await.expect("Timed out waiting for client message").expect("Receive failed");

    server.abort();

    assert_eq!(SERVER_MESSAGE, received);
}
