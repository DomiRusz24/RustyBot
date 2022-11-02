use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use url::Url;

use crate::model::Heartbeat1;
use crate::model::Identify2;
use crate::model::MetaMessage;
use crate::model::Operation;
use crate::model::Properties;
use crate::model::ServerMessages;

pub type SocketTransmitter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type SocketReciever = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

pub type SocketMap = Arc<RwLock<HashMap<String, Socket>>>;

pub struct SocketFactory {
    pub intents: i64,
    pub os: String,
    pub browser: String,
    pub device: String,
    pub sockets: SocketMap,
}

impl SocketFactory {

    pub fn new_with_intents(intents: i64) -> Self {
        SocketFactory {
            intents: intents,
            os: std::env::consts::OS.to_string(),
            browser: "None".to_string(),
            device: "Server".to_string(),
            sockets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new() -> Self {
        Self::new_with_intents(513)
    }

    pub async fn create_socket(self: &mut Self, token: &str) -> String {
        let socket = Socket::new(Identify2 {
            token: token.to_string(),
            intents: self.intents,
            properties: Properties {
                os: self.os.clone(),
                browser: self.browser.clone(),
                device: self.device.clone(),
            },
        })
        .await;

        self.sockets.write().await.insert(token.to_string(), socket);

        token.to_string()
    }
}

pub struct Socket {
    pub tx: SocketTransmitter,
    pub rx: SocketReciever,
    pub heartbeat: u32,
    pub prop: Identify2,
    pub sequence: Option<i32>
}

impl Socket {
    pub async fn new(prop: Identify2) -> Self {
        let (socket, _) =
            connect_async(Url::parse("wss://gateway.discord.gg/?v=10&encoding=json").unwrap())
                .await
                .unwrap();

        let (tx_socket, rx_socket) = socket.split();

        let mut socket = Socket {
            tx: tx_socket,
            rx: rx_socket,
            heartbeat: 0,
            prop: prop,
            sequence: Option::None
        };

        socket.read_message().await;

        socket
    }

    async fn send_message(&mut self, op: u32, data: &impl Serialize) {
        let op = Operation {
            op: op,
            d: serde_json::to_value(data).unwrap(),
        };

        self.send_message_full(&op).await;
    }

    async fn send_message_full(&mut self, data: &impl Serialize) {
        let _ = self
            .tx
            .send(Message::Text(serde_json::to_string(data).unwrap()))
            .await
            .unwrap();
    }

    pub async fn send_heartbeat(&mut self) {
        self.send_message_full(&Heartbeat1 { op: 1, d: self.sequence.clone() }).await;
    }

    pub async fn read_message(&mut self) -> Option<(MetaMessage, ServerMessages)> {
        let msg = self
			.rx
            .next()
            .await
            .unwrap()
            .expect("Error with message");
        
        if msg.is_close() {
        	println!("Close this mf");
        	return None;
        }    



        let (meta, msg) = ServerMessages::parse_server_message(&msg.to_text().unwrap());

        match msg.clone() {
            ServerMessages::Handshake(hello) => {
                self.send_message(2, &self.prop.clone()).await;

                self.heartbeat = hello.heartbeat_interval;
            }
            _ => {}
        }

        if self.sequence.is_none() {
        	self.sequence = meta.s;
        }

        Some((meta, msg))
    }
}
