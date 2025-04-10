use futures_util::stream::SplitStream;
use tokio::sync::{broadcast, oneshot, mpsc};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use crate::schema::{MessageV2G, MessageG2V};

async fn handle_reading(mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {},
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

pub async fn websocket_loop(
    url: &str,
    mut shutdown: oneshot::Receiver<()>,
    mut messages: mpsc::Receiver<MessageV2G>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(url)?;


    let (ws_stream, _) = connect_async(request).await?;
    println!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();
    tokio::select! {
        _ = handle_reading(read) => {},
        _ = shutdown => {
            println!("Shutdown signal received");
        }
    }
    Ok(())
}
