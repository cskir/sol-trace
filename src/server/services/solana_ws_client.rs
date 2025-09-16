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
use crate::server::domain::ws_client::WSCResult;
use crate::server::domain::ws_client::WebSocketClient;
use crate::server::domain::{LogSubscribeResponse, LogsNotification};
use crate::state::SubscriptionInput;

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
    async fn subscribe(
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

        // TODO: refactor to handle the results in one enum: LogSubscribeResponse, LogsNotification, + api error
        if let Some(msg) = read_stream.next().await {
            if let Ok(tungstenite::Message::Text(txt)) = msg {
                if let Ok(subscription_response) =
                    serde_json::from_str::<LogSubscribeResponse>(&txt)
                {
                    let subscription_id = subscription_response.result;
                    self.write_streams.insert(subscription_id, write_stream);
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
                                if let Ok(logs_notification) =
                                    serde_json::from_str::<LogsNotification>(&txt)
                                {
                                    if logs_notification.params.result.value.err.is_none() {
                                        let signature =
                                            logs_notification.params.result.value.signature;

                                        println!("Signature: {:?}", signature);

                                        if tx
                                            .send(Ok(SubscribeResponse { message: signature }))
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                } else {
                                    println!("Received non-log message: {}", txt);
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

    async fn unsubscribe(&mut self, sub_id: u64) -> WSCResult<()> {
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
