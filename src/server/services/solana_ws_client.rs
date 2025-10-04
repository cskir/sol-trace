use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, interval};
use tokio_tungstenite::connect_async;
use tonic::Status;
use tungstenite::protocol::Message;

use crate::proto::SubscribeResponse;
use crate::server::domain::SubscriptionInput;
use crate::server::domain::solana_api_messages::LogSubscribeWsMessage;
use crate::server::domain::ws_client::WSCResult;
use crate::server::domain::ws_client::WebSocketClient;
use crate::server::states::app_state::{
    OffChainRpcClientType, OnChainRpcClientType, TokenStoreType,
};
use crate::server::utils::handle_transaction;

pub struct SolanaWebSocketClient {
    url: String,
    next_req_id: u64,
    write_channel: Arc<Mutex<HashMap<u64, mpsc::Sender<Message>>>>,
}

impl SolanaWebSocketClient {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            next_req_id: 1,
            write_channel: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WebSocketClient for SolanaWebSocketClient {
    #[tracing::instrument(name = "Logs subscribe", skip_all)]
    async fn logs_subscribe(
        &mut self,
        subscription_input: Arc<SubscriptionInput>,
        off_chain_rpc_client: OffChainRpcClientType,
        token_store: TokenStoreType,
        on_chain_rpc_client: OnChainRpcClientType,
        tx: mpsc::Sender<Result<SubscribeResponse, Status>>,
    ) -> WSCResult<u64> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        tracing::info!("WebSocket connected to {}", &self.url);
        let req_id = self.next_req_id;
        self.next_req_id += 1;

        let req = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "method": "logsSubscribe",
            "params": [
                    { "mentions": [subscription_input.clone().wallet.clone()] },
                    { "commitment": "finalized" }
                ]
        });

        let (mut write_stream, mut read_stream) = ws_stream.split();

        write_stream.send(Message::Text(req.to_string())).await?;

        let mut sub_id: Option<u64> = None;

        let (write_tx, mut write_rx) = mpsc::channel::<Message>(3);

        if let Some(msg) = read_stream.next().await {
            if let Ok(tungstenite::Message::Text(txt)) = msg {
                if let Ok(LogSubscribeWsMessage::Subscribed(resp)) =
                    serde_json::from_str::<LogSubscribeWsMessage>(&txt)
                {
                    let subscription_id = resp.result;
                    let write_tx_clone = write_tx.clone();

                    self.write_channel
                        .clone()
                        .lock()
                        .await
                        .insert(subscription_id, write_tx_clone);

                    sub_id = Some(subscription_id);

                    tokio::spawn(async move {
                        while let Some(msg) = write_rx.recv().await {
                            if let Err(_e) = write_stream.send(msg).await {
                                break;
                            }
                        }
                    });
                }
            }
        }

        match sub_id {
            Some(sub_id) => {
                tokio::spawn(async move {
                    while let Some(msg) = read_stream.next().await {
                        match msg {
                            Ok(tungstenite::Message::Text(txt)) => {
                                let mut stream_message: Option<String> = None;
                                match serde_json::from_str::<LogSubscribeWsMessage>(&txt) {
                                    Ok(LogSubscribeWsMessage::Notification(resp)) => {
                                        if resp.params.result.value.err.is_none() {
                                            let signature = resp.params.result.value.signature;

                                            handle_transaction(
                                                signature,
                                                subscription_input.clone(),
                                                off_chain_rpc_client.clone(),
                                                token_store.clone(),
                                                on_chain_rpc_client.clone(),
                                            )
                                            .await
                                            .ok()
                                            .flatten()
                                            .map(
                                                |trade| {
                                                    stream_message = Some(format!(
                                                        "Trade detected: {}",
                                                        trade.to_string()
                                                    ))
                                                },
                                            );
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
                                tracing::error!("WebSocket error {:?}", e);
                                break;
                            }
                        }
                    }
                });

                self.ping(sub_id).await;

                Ok(sub_id)
            }
            None => {
                tracing::error!("logs subscription request failed");
                Err("logsSubscribe subscription request failed".into())
            }
        }
    }

    #[tracing::instrument(name = "Logs unsubscribe", skip_all)]
    async fn logs_unsubscribe(&mut self, sub_id: u64) -> WSCResult<()> {
        if let Some(write_tx) = self.write_channel.lock().await.remove(&sub_id) {
            let req_id = self.next_req_id;
            self.next_req_id += 1;

            let req = json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "method": "logsUnsubscribe",
                "params": [sub_id],
            });

            write_tx.send(Message::Text(req.to_string())).await?;
            write_tx.send(Message::Close(None)).await?;
        }
        Ok(())
    }
}

impl SolanaWebSocketClient {
    #[tracing::instrument(name = "Ping", skip_all)]
    async fn ping(&mut self, sub_id: u64) {
        let write_channel_clone = self.write_channel.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(20));
            loop {
                ticker.tick().await;

                if let Some(write_tx) = write_channel_clone.lock().await.get(&sub_id) {
                    if let Err(e) = write_tx.send(Message::Ping(vec![])).await {
                        tracing::error!("Ping error: {:?}", e);
                        break;
                    } else {
                        //tracing::info!("Ping sent");
                    }
                } else {
                    break;
                }
            }
        });
    }
}
