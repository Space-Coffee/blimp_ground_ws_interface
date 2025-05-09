use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream};

use crate::schema::MessageV2G;
use crate::stream::{BlimpGroundWebsocketStreamPair};
use crate::MessageG2V;
use crate::subprotocol::BlimpSubprotocol;

pub struct BlimpGroundWebsocketClient {
    url: String,
    stream: Option<BlimpGroundWebsocketStreamPair<MaybeTlsStream<TcpStream>>>,
}
impl BlimpGroundWebsocketClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            stream: None,
        }
    }
    fn get_request(&self) -> Result<Request, tungstenite::Error> {
        Ok(Request::builder()
            .uri(self.url.as_str())
            .header("Sec-WebSocket-Protocol", BlimpSubprotocol::default().to_string())
            .header("Host", "")
            .header("Origin", "")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", tungstenite::handshake::client::generate_key())
            .body(())?
        )
    }
    pub async fn connect(&mut self) -> Result<(), tungstenite::Error> {
        let request = self.get_request()?;
        let (stream, _response) = connect_async(request).await?;
        self.stream = Some(BlimpGroundWebsocketStreamPair::from_stream(stream, BlimpSubprotocol::default()));
        Ok(())
    }
    pub async fn disconnect(&mut self) -> Result<(), tungstenite::Error> {
        self.stream.as_mut().unwrap().close().await?;
        self.stream = None;
        Ok(())
    }
    pub async fn send(&self, message: MessageV2G) -> Result<(), Box<dyn std::error::Error>> {
        self.stream
            .as_ref()
            .expect("Unconnected client attempted to send")
            .send(message)
            .await
    }

    pub async fn recv(&self) -> Result<MessageG2V, Box<dyn std::error::Error + Send + Sync>> {
        self.stream
            .as_ref()
            .expect("Unconnected client attempted to receive")
            .recv()
            .await
    }
}
