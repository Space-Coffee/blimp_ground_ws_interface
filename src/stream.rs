use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};

pub struct BlimpGroundWebsocketStreamPair<T>
{
    read_stream: SplitStream<WebSocketStream<T>>,
    write_stream: SplitSink<WebSocketStream<T>, tungstenite::Message>,
}

impl<T> BlimpGroundWebsocketStreamPair<T>
where T: AsyncRead + AsyncWrite + Unpin + Send + 'static {
    pub(crate) fn from_stream(stream: WebSocketStream<T>) -> Self {
        let (write_stream, read_stream) = stream.split();
        Self {read_stream, write_stream}
    }
    pub async fn send<S: Serialize>(&mut self, message: S) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = postcard::to_stdvec(&message)?;
        self.write_stream.send(
            tungstenite::Message::Binary(tungstenite::Bytes::from(serialized))
        ).await?;
        Ok(())
    }

    pub async fn recv<R: DeserializeOwned>(&mut self) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        while let Some(msg) = self.read_stream.next().await {
            match msg {
                Ok(tungstenite::Message::Binary(data)) => {
                    return match postcard::from_bytes(&data) {
                        Ok(data) => Ok(data),
                        Err(error) => Err(format!("Failed to deserialize: {}", error).into())
                    }
                }
                Ok(tungstenite::Message::Close(_)) => {
                    return Err("Connection closed by peer".into());
                }
                Ok(_) => continue,
                Err(e) => {
                    return Err(format!("Error reading message: {}", e).into());
                }
            }
        }

        Err("Stream ended without receiving a data packet".into())
    }


    pub async fn close(&mut self) -> Result<(), tungstenite::Error> {
        self.write_stream.close().await
    }
}
