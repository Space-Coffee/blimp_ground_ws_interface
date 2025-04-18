use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use crate::MessageV2G;
use crate::stream::BlimpGroundWebsocketStreamPair;

pub struct BlimpGroundWebsocketServer {
    url: String,
    listener: Option<TcpListener>
}

impl BlimpGroundWebsocketServer {
    pub fn new(url: &str) -> Self {
        Self {url: url.to_string(), listener: None}
    }

    pub async fn bind(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.url.clone()).await?;
        self.listener = Some(listener);
        Ok(())
    }

    pub async fn unbind(&mut self) {
        self.listener = None;
    }

    async fn handle_connection<F, Fut>(mut connection: BlimpGroundWebsocketStreamPair<TcpStream>, handler: F) -> ()
    where F: Fn(MessageV2G) -> Fut, Fut: Future<Output=()> + Send {
        println!("connected");
        loop {
            match connection.recv::<MessageV2G>().await {
                Ok(data) => {
                    handler(data).await;
                }
                Err(error) => break
            };
        }
        println!("disconnected")

    }

    pub async fn run<F, Fut>(&mut self, handler: F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where F: Fn(MessageV2G) -> Fut + Sync + Send + 'static, Fut: Future<Output=()> + Send {
        let owned_handler = Arc::new(handler);
        loop {
            let (tcp_stream, address) = self.listener.as_mut().expect("Socket hasn't been bound").accept().await?;

            let websocket_stream = accept_async(tcp_stream).await?;

            let mut pair = BlimpGroundWebsocketStreamPair::from_stream(websocket_stream);
            let handler= Arc::clone(&owned_handler);
            tokio::spawn(async move {
                Self::handle_connection(pair, move |message| handler(message)).await;
            });
        }
    }
}

