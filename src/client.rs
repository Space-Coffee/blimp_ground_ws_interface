use std::future::{Future, IntoFuture};
use futures_util::stream::{SplitSink, SplitStream};
use tokio::sync::{broadcast, oneshot};
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};
use url::Url;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use crate::MessageG2V;
use crate::schema::MessageV2G;

struct BlimpGroundWebsocketStreamPair {
    read_stream: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    write_stream: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>
}

impl BlimpGroundWebsocketStreamPair {
    fn from_stream(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        let (write_stream, read_stream) = stream.split();
        Self {read_stream, write_stream}
    }
    async fn send(&mut self, message: impl Serialize) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = postcard::to_stdvec(&message)?;
        self.write_stream.send(
            tungstenite::Message::Binary(tungstenite::Bytes::from(serialized))
        ).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<MessageG2V, Box<dyn std::error::Error>> {
        todo!();
    }


    async fn close(&mut self) -> Result<(), tungstenite::Error> {
        self.write_stream.close().await
    }
}
pub struct BlimpGroundWebsocketClient {
    url: String,
    stream: Option<BlimpGroundWebsocketStreamPair>,
}
impl BlimpGroundWebsocketClient {
    pub fn new(url: &str) -> Self {
        Self {url: url.to_string(), stream: None}
    }
    fn get_request(&self) -> Result<Request, tungstenite::Error> {
        tungstenite::client::IntoClientRequest::into_client_request(&self.url)
    }
    pub async fn connect(&mut self) -> Result<(), tungstenite::Error> {
        let request = self.get_request()?;
        let (stream, response) = connect_async(request).await?;
        self.stream = Some(BlimpGroundWebsocketStreamPair::from_stream(stream));
        Ok(())
    }
    pub async fn disconnect(&mut self) -> Result<(), tungstenite::Error> {
        self.stream.as_mut().unwrap().close().await?;
        self.stream = None;
        Ok(())
    }
    pub async fn send(&mut self, message: MessageV2G) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.as_mut().unwrap().send(message).await
    }
}
