use std::future::{Future, IntoFuture};
use tokio::sync::{broadcast, oneshot};
use tokio_tungstenite::connect_async;
use url::Url;
use futures_util::StreamExt;


pub async fn websocket_loop(
    url: &str,
    mut shutdown: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(url)?;


    let (ws_stream, _) = connect_async(request).await?;
    println!("WebSocket connected");

    let (mut _write, mut read) = ws_stream.split();

    tokio::select! {
        _ = async {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(message) => println!("Received: {}", message),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        } => {},
        _ = shutdown => {
            println!("Shutdown signal received");
        }
    }
    Ok(())
}
