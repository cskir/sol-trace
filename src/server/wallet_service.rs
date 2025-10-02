//use futures_util::TryFutureExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::proto::{
    CallRequest, CallResponse, InitRequest, InitResponse, SubscribeRequest, SubscribeResponse,
    UnsubscribeRequest, UnsubscribeResponse,
    cli_service_server::{CliService, CliServiceServer},
};
use crate::proto::{HoldingsRequest, HoldingsResponse};
use crate::server::states::{AppState, ClientState, SubscriptionState};
use crate::server::utils::constants::WSOL;
use crate::server::utils::{query_holdings, store_tokens, validate_init_data};

pub struct WalletService {
    state: Arc<AppState>,
}

impl WalletService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl CliService for WalletService {
    type SubscribeStream = ReceiverStream<Result<SubscribeResponse, Status>>;

    #[tracing::instrument(name = "Init", skip_all)]
    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        tracing::info!("New client request received");
        let new_id = Uuid::new_v4();

        let mut init_request = request.into_inner();

        if !init_request.tokens.contains(&WSOL.to_string()) {
            init_request.tokens.push(WSOL.to_string());
        }

        validate_init_data(&init_request)?;

        store_tokens(
            &init_request.tokens,
            self.state.off_chain_rpc_client.clone(),
            self.state.token_store.clone(),
        )
        .await?;

        self.state.clients.write().await.insert(
            new_id.clone(),
            ClientState::build(init_request, self.state.ws_client_factory.clone()),
        );

        tracing::info!("Registered new client with ID: {}", new_id);

        Ok(Response::new(InitResponse {
            client_id: new_id.to_string(),
        }))
    }

    #[tracing::instrument(name = "Subscribe", skip_all)]
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<<WalletService as CliService>::SubscribeStream>, Status> {
        let client_id = extract_client_id(&request)?;

        let (tx, rx) = mpsc::channel(10);

        let mut clients = self.state.clients.write().await;

        match clients.get_mut(&client_id) {
            Some(client_state) => {
                if client_state.logs_subscription.is_some() {
                    tracing::warn!("Client {} already has an active subscription", client_id);
                    return Err(Status::failed_precondition("Subscription already exists"));
                }

                tracing::info!("call logs subscribe for: {}", client_id);

                if let Ok(subscription_id) = client_state
                    .ws_client
                    .write()
                    .await
                    .logs_subscribe(
                        client_state.subscription_input.clone(),
                        self.state.off_chain_rpc_client.clone(),
                        self.state.token_store.clone(),
                        self.state.on_chain_rpc_client.clone(),
                        tx.clone(),
                    )
                    .await
                {
                    client_state.logs_subscription = Some(SubscriptionState { subscription_id });
                    tracing::info!("Subscription was successful with id: {}", subscription_id);
                }
            }
            None => {
                tracing::warn!("Client {} not found", client_id);
                return Err(Status::not_found("Client not found"));
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    #[tracing::instrument(name = "Unsubscribe", skip_all)]
    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeResponse>, Status> {
        let client_id = extract_client_id(&request)?;
        //let state_clone = self.state.clone();
        let mut clients = self.state.clients.write().await;

        match clients.get_mut(&client_id) {
            Some(client_state) => {
                if let Some(subscription) = &client_state.logs_subscription {
                    let _ = client_state
                        .ws_client
                        .write()
                        .await
                        .logs_unsubscribe(subscription.subscription_id)
                        .await;
                }

                client_state.logs_subscription = None;
                tracing::info!("Unsubscription was successful for client: {}", client_id);
            }
            None => {
                tracing::warn!("Client {} not found", client_id);
                return Err(Status::not_found("Client not found"));
            }
        }

        Ok(Response::new(UnsubscribeResponse {
            message: "Unsubscribed successfully".to_string(),
        }))
    }

    #[tracing::instrument(name = "Holdings", skip_all)]
    async fn holdings(
        &self,
        request: Request<HoldingsRequest>,
    ) -> Result<Response<HoldingsResponse>, Status> {
        let client_id = extract_client_id(&request)?;

        let clients = self.state.clients.read().await;

        match clients.get(&client_id) {
            Some(client_state) => {
                let holdings_response = query_holdings(
                    &client_state.subscription_input.wallet,
                    client_state.token_account_map.clone(),
                    self.state.token_store.clone(),
                    self.state.on_chain_rpc_client.clone(),
                    self.state.off_chain_rpc_client.clone(),
                )
                .await
                .map_err(|e| {
                    tracing::error!("Failed to query holdings: {}", e);
                    Status::internal("Failed to query holdings")
                })?;

                Ok(Response::new(holdings_response))
            }
            None => {
                tracing::warn!("Client {} not found", client_id);
                Err(Status::not_found("Client not found"))
            }
        }
    }

    async fn call(&self, request: Request<CallRequest>) -> Result<Response<CallResponse>, Status> {
        let payload = request.into_inner().payload;
        let reply = format!("Processed: {}", payload);
        Ok(Response::new(CallResponse { reply }))
    }
}

fn extract_client_id<T>(req: &Request<T>) -> Result<Uuid, Status> {
    let client_id = req
        .metadata()
        .get("client-id")
        .ok_or_else(|| Status::unauthenticated("missing client id"))?
        .to_str()
        .map_err(|_| Status::invalid_argument("invalid client id"))?;

    Uuid::parse_str(client_id).map_err(|_| Status::invalid_argument("malformed uuid"))
}

pub async fn run_server(addr: &str, state: AppState) -> Result<(), Box<dyn std::error::Error>> {
    let svc = WalletService::new(Arc::new(state));

    tracing::info!("Server listening on {}", addr);

    // INFO: adding TraceLayer gave trait bound error for the grpc stream sercvices
    Server::builder()
        .add_service(CliServiceServer::new(svc))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
