use std::net::SocketAddr;
use tokio::sync::oneshot;
use tokio_tungstenite::{connect_async, connect_async_with_config};
use tungstenite::http::Request;
use blimp_ground_ws_interface::{BlimpGroundWebsocketServer, MessageV2G};

#[tokio::test]
async fn test_choose_protocol() {
    let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();

    let server = {
        tokio::spawn(async move {
            let mut server = BlimpGroundWebsocketServer::new("localhost:0");
            server.bind().await.expect("Failed address bind");
            address_tx
                .send(server.get_address().unwrap())
                .expect("Did not send the address properly");
            server
                .run(move |_| { async move {} })
                .await
                .expect("Server failed");
        })
    };

    let address = address_rx.await.expect("Failed to receive target address");
    let url = format!("ws://{}", address);
    
    let request = Request::builder()
        .uri(url)
        .header("Host", "")
        .header("Origin", "")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Protocol", "spacecoffee.blimp.v2137.json, spacecoffee.blimp.v1.postcard")
        .header("Sec-WebSocket-Key", tungstenite::handshake::client::generate_key())
        .body(())
        .expect("Failed to build the request");
    
    let (mut stream, response) = connect_async(request).await.expect("Failed to connect");
    stream.close(None).await.expect("Failed to close");
    assert_eq!(response.headers().get("Sec-WebSocket-Protocol").unwrap().to_str().unwrap(), "spacecoffee.blimp.v1.postcard");
}
