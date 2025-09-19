use std::{net::SocketAddr, sync::Arc};

use sol_trace::{
    proto::{
        InitRequest, SubscribeRequest, SubscribeResponse, UnsubscribeRequest,
        cli_service_client::CliServiceClient, cli_service_server::CliServiceServer,
    },
    server::{
        domain::{SubscriptionInput, WSCResult, WebSocketClient},
        services::HashmapTokenStore,
        states::AppState,
        wallet_service::WalletService,
    },
};

use async_trait::async_trait;
use tokio::{net::TcpListener, sync::mpsc};
use tokio::{
    sync::RwLock,
    time::{Duration, sleep},
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, Status, metadata::MetadataValue, transport::Server};
use uuid::Uuid;

pub struct MockWebSocketClient {}

#[async_trait]
impl WebSocketClient for MockWebSocketClient {
    async fn logs_subscribe(
        &mut self,
        _subscription_input: Arc<SubscriptionInput>,
        tx: mpsc::Sender<Result<SubscribeResponse, Status>>,
    ) -> WSCResult<u64> {
        let sub_id: u64 = 11111;

        sleep(Duration::from_millis(500)).await;
        tx.send(Ok(SubscribeResponse {
            message: "Subscription stream data".to_string(),
        }))
        .await?;

        Ok(sub_id)
    }

    async fn logs_unsubscribe(&mut self, _sub_id: u64) -> WSCResult<()> {
        Ok(())
    }
}

async fn run_test_server(incoming: TcpListenerStream) -> Result<(), Box<dyn std::error::Error>> {
    let token_store = Arc::new(RwLock::new(HashmapTokenStore::default()));
    let ws_client_factory: Arc<dyn Fn() -> Box<dyn WebSocketClient + Send + Sync> + Send + Sync> =
        Arc::new(move || Box::new(MockWebSocketClient {}));
    let state = AppState::new(token_store, ws_client_factory);
    let svc = WalletService::new(Arc::new(state));

    Server::builder()
        .add_service(CliServiceServer::new(svc))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}

pub async fn init_server_client() -> TestClientApp {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let incoming = TcpListenerStream::new(listener);

    tokio::spawn(async move {
        run_test_server(incoming).await.expect("Server failed");
    });

    sleep(Duration::from_millis(100)).await;

    let client = TestClientApp::build(addr).await;
    client
}

pub struct TestClientApp {
    pub client: CliServiceClient<tonic::transport::Channel>,
    pub client_id: Uuid,
}

impl TestClientApp {
    pub async fn build(addr: SocketAddr) -> Self {
        let mut client: CliServiceClient<tonic::transport::Channel> =
            CliServiceClient::connect(format!("http://{}", addr))
                .await
                .unwrap();

        let init_request = InitRequest {
            wallet: "Wallet1".to_owned(),
            tokens: vec!["token1".to_owned()],
        };
        let init_response = client.init(init_request).await.unwrap().into_inner();

        let client_id = Uuid::parse_str(init_response.client_id.as_str()).unwrap();

        Self { client, client_id }
    }

    pub async fn sub(
        &mut self,
    ) -> Result<tonic::Streaming<SubscribeResponse>, Box<dyn std::error::Error>> {
        let mut subscribe_request = Request::new(SubscribeRequest {});
        subscribe_request.metadata_mut().insert(
            "client-id",
            MetadataValue::try_from(self.client_id.clone().to_string())?,
        );

        let stream: tonic::Streaming<SubscribeResponse> =
            self.client.subscribe(subscribe_request).await?.into_inner();

        Ok(stream)
    }

    pub async fn unsub(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut unsub_request = Request::new(UnsubscribeRequest {});
        unsub_request.metadata_mut().insert(
            "client-id",
            MetadataValue::try_from(self.client_id.clone().to_string())?,
        );

        self.client.unsubscribe(unsub_request).await?.into_inner();

        Ok(())
    }
}
