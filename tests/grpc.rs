// use sol_trace::proto::{CallRequest, SubscribeRequest, cli_service_client::CliServiceClient};
// use sol_trace::server::run_server;
// use sol_trace::state::AppState;
// use tokio::time::{Duration, sleep};
// use tonic::Request;

// #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
// async fn integration_test_example_proto() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "127.0.0.1:50053";
//     let state = AppState::new();

//     let state_clone = state.clone();
//     tokio::spawn(async move {
//         run_server(addr, state_clone).await.expect("Server failed");
//     });

//     sleep(Duration::from_millis(500)).await;

//     let mut client = CliServiceClient::connect(format!("http://{}", addr)).await?;

//     let wallet = "integration_wallet".to_string();
//     let mut stream = client
//         .subscribe(Request::new(SubscribeRequest {
//             wallet: wallet.clone(),
//             tokens: vec!["token1".to_string(), "token2".to_string()],
//         }))
//         .await?
//         .into_inner();

//     sleep(Duration::from_millis(500)).await;

//     if let Some(msg) = stream.message().await? {
//         println!("Received subscription message: {}", msg.message);
//         assert!(msg.message.contains(&wallet));
//     } else {
//         panic!("No subscription message received");
//     }

//     Ok(())
// }
