use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;
use tonic::Status;
use tungstenite::protocol::Message;

use crate::proto::SubscribeResponse;
use crate::server::domain::SubscriptionInput;
use crate::server::domain::solana_api_messages::LogSubscribeWsMessage;
use crate::server::domain::ws_client::WSCResult;
use crate::server::domain::ws_client::WebSocketClient;

pub struct SolanaWebSocketClient {
    url: String,
    next_req_id: u64,
    write_streams: HashMap<u64, SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

impl SolanaWebSocketClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            next_req_id: 1,
            write_streams: HashMap::new(),
        }
    }
}

#[async_trait]
impl WebSocketClient for SolanaWebSocketClient {
    async fn logs_subscribe(
        &mut self,
        subscription_input: Arc<SubscriptionInput>,
        tx: mpsc::Sender<Result<SubscribeResponse, Status>>,
    ) -> WSCResult<u64> {
        println!("wsc connect requested");
        let (ws_stream, _) = connect_async(&self.url).await?;
        println!("wsc connected");
        let req_id = self.next_req_id;
        self.next_req_id += 1;

        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "logsSubscribe",
            "params": [
                    { "mentions": [subscription_input.wallet.clone()] },
                    { "commitment": "finalized" }
                ]
        });

        println!("wsc logsSubscribe req: {}", req);

        let (mut write_stream, mut read_stream) = ws_stream.split();

        write_stream.send(Message::Text(req.to_string())).await?;

        println!("wsc logsSubscribe sent");

        let mut sub_id: Option<u64> = None;

        if let Some(msg) = read_stream.next().await {
            if let Ok(tungstenite::Message::Text(txt)) = msg {
                if let Ok(LogSubscribeWsMessage::Subscribed(resp)) =
                    serde_json::from_str::<LogSubscribeWsMessage>(&txt)
                {
                    let subscription_id = resp.result;
                    __self.write_streams.insert(subscription_id, write_stream);
                    sub_id = Some(subscription_id);
                }
            }
        }

        match sub_id {
            Some(sub_id) => {
                println!("wsc sub_id {} ", sub_id);
                tokio::spawn(async move {
                    while let Some(msg) = read_stream.next().await {
                        match msg {
                            Ok(tungstenite::Message::Text(txt)) => {
                                let mut stream_message: Option<String> = None;
                                match serde_json::from_str::<LogSubscribeWsMessage>(&txt) {
                                    Ok(LogSubscribeWsMessage::Notification(resp)) => {
                                        if resp.params.result.value.err.is_none() {
                                            let _signature = resp.params.result.value.signature;
                                        }
                                    }
                                    Ok(LogSubscribeWsMessage::UnSubscribed(resp)) => {
                                        // not sure we'll get it, the stream might end sooner
                                        stream_message = Some(format!(
                                            "Unsubscription success: {}",
                                            resp.result
                                        ));
                                    }
                                    Ok(LogSubscribeWsMessage::Error(resp)) => {
                                        // ? ignore the transient stream error
                                        stream_message =
                                            Some(format!("Error response: {}", resp.error.message));
                                    }
                                    Ok(LogSubscribeWsMessage::Subscribed(_resp)) => {
                                        //not possible, ignore it for now
                                    }
                                    Err(_) => {}
                                }

                                if let Some(message) = stream_message {
                                    if tx.send(Ok(SubscribeResponse { message })).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("WebSocket error {:?}", e);
                                break;
                            }
                        }
                    }
                });
                Ok(sub_id)
            }
            None => Err("logsSubscribe subscription request failed".into()),
        }
    }

    async fn logs_unsubscribe(&mut self, sub_id: u64) -> WSCResult<()> {
        if let Some(mut write_stream) = self.write_streams.remove(&sub_id) {
            let req_id = self.next_req_id;
            self.next_req_id += 1;

            let req = json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "method": "logsUnsubscribe",
                "params": [sub_id],
            });

            write_stream.send(Message::Text(req.to_string())).await?;

            // this will close the read_stream as well
            write_stream.send(Message::Close(None)).await?;
        }
        Ok(())
    }
}
