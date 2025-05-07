use std::cmp::PartialEq;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use phf::phf_map;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::RwLock as TRwLock;
use tokio_tungstenite::{tungstenite, WebSocketStream};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlimpSubprotocol {
    JsonV1,
    PostcardV1,
}

pub static ALLOWED_PROTOCOLS: phf::Map<&'static str, BlimpSubprotocol> = phf_map! {
    "spacecoffee.blimp.v1.json" => BlimpSubprotocol::JsonV1,
    "spacecoffee.blimp.v1.postcard" => BlimpSubprotocol::PostcardV1,
};
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
        if subprotocol != BlimpSubprotocol::PostcardV1 {
            unimplemented!("Only Postcard protocol is implemented for now")
        }
        Self {
            read_stream: TRwLock::new(read_stream),
            write_stream: TRwLock::new(write_stream),
            subprotocol
        }
    }
    pub async fn send<S: Serialize>(&self, message: S) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = postcard::to_stdvec(&message)?;
        self.write_stream
            .write()
            .await
            .send(tungstenite::Message::Binary(tungstenite::Bytes::from(
                serialized,
            )))
            .await?;
        Ok(())
    }

    pub async fn recv<R: DeserializeOwned>(
        &self,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        while let Some(msg) = self.read_stream.write().await.next().await {
            match msg {
                Ok(tungstenite::Message::Binary(data)) => {
                    return match postcard::from_bytes(&data) {
                        Ok(data) => Ok(data),
                        Err(error) => Err(format!("Failed to deserialize: {}", error).into()),
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

    pub async fn close(&self) -> Result<(), tungstenite::Error> {
        self.write_stream.write().await.close().await
    }
}
