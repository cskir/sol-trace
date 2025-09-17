//use futures_util::TryFutureExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::async_trait;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;
//use uuid::Uuid;

use crate::proto::{
    CallRequest, CallResponse, InitRequest, InitResponse, SubscribeRequest, SubscribeResponse,
    UnsubscribeRequest, UnsubscribeResponse,
    cli_service_server::{CliService, CliServiceServer},
};
use crate::state::{AppState, ClientState, SubscriptionState};

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

    async fn init(&self, request: Request<InitRequest>) -> Result<Response<InitResponse>, Status> {
        let new_id = Uuid::new_v4();

        self.state.clients.write().await.insert(
            new_id.clone(),
            ClientState::build(request.into_inner(), self.state.ws_client_factory.clone()),
        );

        println!("Registered new client with ID: {}", new_id);

        Ok(Response::new(InitResponse {
            client_id: new_id.to_string(),
        }))
    }

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<<WalletService as CliService>::SubscribeStream>, Status> {
        let client_id = extract_client_id(&request)?;

        println!("subscribe requested: {}", client_id);

        let (tx, rx) = mpsc::channel(10);

        let mut clients = self.state.clients.write().await;

        match clients.get_mut(&client_id) {
            Some(client_state) => {
                if client_state.subscription.is_some() {
                    return Err(Status::failed_precondition("Subscription already exists"));
                }

                println!("call ws client subscribe: ");

                if let Ok(sub_id) = client_state
                    .ws_client
                    .write()
                    .await
                    .subscribe(client_state.subscription_input.clone(), tx.clone())
                    .await
                {
                    client_state.subscription = Some(SubscriptionState { sub_id: sub_id });
                }
            }
            None => {
                return Err(Status::not_found("Client not found"));
            }
        }

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn unsubscribe(
        &self,
        request: Request<UnsubscribeRequest>,
    ) -> Result<Response<UnsubscribeResponse>, Status> {
        let client_id = extract_client_id(&request)?;
        //let state_clone = self.state.clone();
        println!("unsubscribe requested: {}", client_id);
        let mut clients = self.state.clients.write().await;

        match clients.get_mut(&client_id) {
            Some(client_state) => {
                if let Some(subscription) = &client_state.subscription {
                    let _ = client_state
                        .ws_client
                        .write()
                        .await
                        .unsubscribe(subscription.sub_id)
                        .await;
                }

                client_state.subscription = None;
            }
            None => {
                return Err(Status::not_found("Client not found"));
            }
        }

        Ok(Response::new(UnsubscribeResponse {
            message: "Unsubscribed successfully".to_string(),
        }))
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
    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(CliServiceServer::new(svc))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
