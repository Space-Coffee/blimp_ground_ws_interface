use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{oneshot, Mutex as TMutex, Notify};
use tokio::time::{timeout, Duration};

use blimp_ground_ws_interface::{BlimpGroundWebsocketClient, BlimpGroundWebsocketServer, BlimpGroundWebsocketStreamPair, BlimpSubprotocol, BlimpSubprotocolFlavour, MessageG2V, MessageV2G, VizInterest};

async fn run_server<F, Fut>(address_tx: oneshot::Sender<SocketAddr>, handler: F)
where
    F: Fn(BlimpGroundWebsocketStreamPair<TcpStream>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut server = BlimpGroundWebsocketServer::new("localhost:0");
    server.bind().await.expect("Failed address bind");
    address_tx
        .send(server.get_address().unwrap())
        .expect("Did not send the address properly");
    server.run(handler).await.expect("Server failed");
}
async fn run_client<F, Fut>(address_rx: oneshot::Receiver<SocketAddr>, handler: F)
where
    F: Fn(&BlimpGroundWebsocketClient) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    let address = address_rx.await.expect("Failed to receive target address");
    let mut client = BlimpGroundWebsocketClient::new(format!("ws://{}", address).as_str());
    client.connect().await.expect("Failed to connect the client");

    handler(&client).await;

    client.disconnect().await.expect("Failed to disconnect the client");
}

const CLIENT_MESSAGE: MessageV2G = MessageV2G::DeclareInterest(VizInterest {
    motors: true,
    servos: true,
    sensors: true,
});
#[tokio::test]
async fn test_client_send() {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(TMutex::new(Some(tx)));

    let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();

    let server = {
        let tx = tx.clone();
        tokio::spawn(run_server(address_tx, move |pair| {
            let tx = tx.clone();
            async move {
                let message = pair
                    .recv::<MessageV2G>()
                    .await
                    .expect("Failed to receive client message");
                tx.lock()
                    .await
                    .take()
                    .expect("Tx already taken")
                    .send(message)
                    .expect("Failed to send");
            }}
        ))
    };

    let address = address_rx.await.expect("Failed to receive target address");
    let mut client = BlimpGroundWebsocketClient::new(format!("ws://{}", address).as_str());

    client
        .connect()
        .await
        .expect("Failed to connect the client");
    client
        .send(CLIENT_MESSAGE)
        .await
        .expect("Failed to send the client message");

    let received = timeout(Duration::from_secs(1), rx)
        .await
        .expect("Timed out waiting for client message")
        .expect("Receive failed");

    assert_eq!(CLIENT_MESSAGE, received);

    server.abort();
}

#[tokio::test]
async fn test_client_send_json() {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(TMutex::new(Some(tx)));

    let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();

    let server = {
        let tx = tx.clone();
        tokio::spawn(run_server(address_tx, move |pair| {
            let tx = tx.clone();
            async move {
                let message = pair
                    .recv::<MessageV2G>()
                    .await
                    .expect("Failed to receive client message");
                tx.lock()
                    .await
                    .take()
                    .expect("Tx already taken")
                    .send(message)
                    .expect("Failed to send");
            }}
        ))
    };

    let address = address_rx.await.expect("Failed to receive target address");
    let mut client = BlimpGroundWebsocketClient::new(format!("ws://{}", address).as_str());
    client.subprotocol = BlimpSubprotocol{version: 1, flavour: BlimpSubprotocolFlavour::Json};

    client
        .connect()
        .await
        .expect("Failed to connect the client");
    client
        .send(CLIENT_MESSAGE)
        .await
        .expect("Failed to send the client message");

    let received = timeout(Duration::from_secs(1), rx)
        .await
        .expect("Timed out waiting for client message")
        .expect("Receive failed");

    server.abort();

    assert_eq!(CLIENT_MESSAGE, received);
}

const SERVER_MESSAGE: MessageG2V = MessageG2V::MotorSpeed { id: 0, speed: 0 };
#[tokio::test]
async fn test_server_send() {
    let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();
    let server = {
        tokio::spawn(run_server(address_tx, move |pair| async move {
                pair.send(SERVER_MESSAGE).await.expect("Failed to send server message");
                pair.close().await.expect("Failed to close");
            }
        ))
    };

    let address = address_rx.await.expect("Failed to receive target address");
    let mut client = BlimpGroundWebsocketClient::new(format!("ws://{}", address).as_str());

    client
        .connect()
        .await
        .expect("Failed to connect the client");

    let received = timeout(Duration::from_secs(1), client.recv())
        .await
        .expect("Timed out waiting for client message")
        .expect("Receive failed");

    server.abort();

    assert_eq!(SERVER_MESSAGE, received);
}
#[tokio::test]
async fn test_server_send_json() {
    let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();
    let server = {
        tokio::spawn(run_server(address_tx, move |pair| async move {
            pair.send(SERVER_MESSAGE).await.expect("Failed to send server message");
            pair.close().await.expect("Failed to close");
        }
        ))
    };

    let address = address_rx.await.expect("Failed to receive target address");
    let mut client = BlimpGroundWebsocketClient::new(format!("ws://{}", address).as_str());
    client.subprotocol = BlimpSubprotocol{version: 1, flavour: BlimpSubprotocolFlavour::Json};

    client
        .connect()
        .await
        .expect("Failed to connect the client");

    let received = timeout(Duration::from_secs(1), client.recv())
        .await
        .expect("Timed out waiting for client message")
        .expect("Receive failed");

    server.abort();

    assert_eq!(SERVER_MESSAGE, received);
}

#[tokio::test]
async fn test_concurrent_sends() {
    const MSGS_COUNT: usize = 64;

    let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();
    let server_recv_notify = Arc::new(Notify::new());
    let client_recv_notify = Arc::new(Notify::new());

    let server = {
        let server_recv_notify = server_recv_notify.clone();
        tokio::spawn(run_server(address_tx, move |pair| {
            let server_recv_notify = server_recv_notify.clone();
            async move {
                let pair = Arc::new(pair);
                let recv_task = {
                    let pair = pair.clone();
                    tokio::spawn(async move {
                        for _i in 0..MSGS_COUNT {
                            let msg = pair.recv::<MessageV2G>().await.unwrap();
                            assert_eq!(msg, CLIENT_MESSAGE);
                        }
                        server_recv_notify.notify_one();
                    })
                };

                for _i in 0..MSGS_COUNT {
                    pair.send(SERVER_MESSAGE)
                        .await
                        .expect("Failed to send server message");
                }

                recv_task.await.unwrap();
            }
        }))
    };

    let address = address_rx.await.expect("Failed to receive target address");
    let mut client = BlimpGroundWebsocketClient::new(format!("ws://{}", address).as_str());

    client
        .connect()
        .await
        .expect("Failed to connect the client");

    let client = Arc::new(client);
    {
        let client = client.clone();
        let client_recv_notify = client_recv_notify.clone();
        tokio::spawn(async move {
            for _i in 0..MSGS_COUNT {
                let msg = client.recv().await.unwrap();
                assert_eq!(msg, SERVER_MESSAGE);
            }
            client_recv_notify.notify_one();
        });
    }

    for _i in 0..MSGS_COUNT {
        client
            .send(CLIENT_MESSAGE)
            .await
            .expect("Failed to send the client message");
    }

    let mut notify_join_set = tokio::task::JoinSet::new();
    notify_join_set.spawn(async move { server_recv_notify.notified().await });
    notify_join_set.spawn(async move { client_recv_notify.notified().await });
    timeout(Duration::from_secs(4), notify_join_set.join_all())
        .await
        .expect("Timed out waiting for transmission completion");
}
