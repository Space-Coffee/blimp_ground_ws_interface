use std::future::Future;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use crate::stream::BlimpGroundWebsocketStreamPair;
use crate::subprotocol::BlimpSubprotocol;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_tungstenite::accept_hdr_async;
use tungstenite;
use tungstenite::http::{HeaderValue, StatusCode};

pub struct BlimpGroundWebsocketServer {
    url: String,
    listener: Option<TcpListener>,
}

impl BlimpGroundWebsocketServer {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            listener: None,
        }
    }

    pub fn get_address(&self) -> Result<SocketAddr, Box<dyn std::error::Error>> {
        Ok(self.listener.as_ref().ok_or("aa")?.local_addr()?)
    }
    pub async fn bind(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.url.clone()).await?;
        self.listener = Some(listener);
        Ok(())
    }

    pub async fn unbind(&mut self) {
        self.listener = None;
    }

    fn get_subprotocol(request: &tungstenite::handshake::server::Request) -> Option<BlimpSubprotocol> {
        let subprotocols: Option<Vec<&str>> = request.headers()
            .get("Sec-WebSocket-Protocol")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.split(',').map(str::trim).collect());
        
        for subprotocol_name in subprotocols? {
            if let Ok(subprotocol) = BlimpSubprotocol::from_str(subprotocol_name) {
                return Some(subprotocol)
            }
        }
        None
    }
    pub async fn run<F, Fut>(
        &mut self,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(BlimpGroundWebsocketStreamPair<TcpStream>) -> Fut + Sync + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let owned_handler = Arc::new(handler);
        loop {
            let (tcp_stream, _address) = self
                .listener
                .as_mut()
                .expect("Socket hasn't been bound")
                .accept()
                .await?;

            let (subprotocol_tx, subprotocol_rx) = oneshot::channel();
            let hdr_handler =
                move |req: &tungstenite::handshake::server::Request,
                      mut res: tungstenite::handshake::server::Response| {
                    let subprotocol = Self::get_subprotocol(req);
                    match subprotocol {
                        None => {
                            Err(tungstenite::handshake::server::Response::builder().status(StatusCode::BAD_REQUEST).body(Some("No provided valid subprotocols".to_string())).unwrap())
                        }
                        Some(protocol) => {
                            res.headers_mut().insert("Sec-WebSocket-Protocol", HeaderValue::from_str(protocol.to_string().as_str()).unwrap());
                            subprotocol_tx.send(protocol).unwrap();
                            Ok(res)
                        }
                    }
                };

            let websocket_stream = accept_hdr_async(tcp_stream, hdr_handler).await?;

            let subprotocol = subprotocol_rx.await?;

            let pair = BlimpGroundWebsocketStreamPair::from_stream(websocket_stream, subprotocol);
            let handler = Arc::clone(&owned_handler);
            tokio::spawn(async move {
                handler(pair).await;
            });
        }
    }
}
