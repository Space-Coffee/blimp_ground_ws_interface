use crate::subprotocol::{BlimpSubprotocol, BlimpSubprotocolFlavour};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::RwLock as TRwLock;
use tokio_tungstenite::{tungstenite, WebSocketStream};

pub struct BlimpGroundWebsocketStreamPair<T> {
    read_stream: TRwLock<SplitStream<WebSocketStream<T>>>,
    write_stream: TRwLock<SplitSink<WebSocketStream<T>, tungstenite::Message>>,
    subprotocol: BlimpSubprotocol
}

impl<T> BlimpGroundWebsocketStreamPair<T>
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    pub(crate) fn from_stream(stream: WebSocketStream<T>, subprotocol: BlimpSubprotocol) -> Self {
        let (write_stream, read_stream) = stream.split();
        Self {
            read_stream: TRwLock::new(read_stream),
            write_stream: TRwLock::new(write_stream),
            subprotocol
        }
    }
    pub async fn send<S: Serialize>(&self, message: S) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_stream
            .write()
            .await
            .send(self.subprotocol.export_packet(message)?)
            .await?;
        Ok(())
    }

    pub async fn recv<R: DeserializeOwned>(
        &self,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        while let Some(msg) = self.read_stream.write().await.next().await {
            match msg {
                Ok(tungstenite::Message::Close(_)) => {
                    return Err("Connection closed by peer".into());
                }
                Ok(tungstenite::Message::Binary(data)) => {
                    return self.subprotocol.import_packet(tungstenite::Message::Binary(data));
                }
                Ok(tungstenite::Message::Text(data)) => {
                    return self.subprotocol.import_packet(tungstenite::Message::Text(data));
                }
                Ok(_) => continue,
                Err(e) => {
                    return Err(format!("Error reading message: {}", e).into());
                }
            }
        }

        Err("Stream ended without receiving a data packet".into())
    }

    pub async fn close(&self) -> Result<(), tungstenite::Error> {
        self.write_stream.write().await.close().await
    }
}
