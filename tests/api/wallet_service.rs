use tokio::time::{Duration, sleep};
use tonic::Status;
use uuid::Uuid;

use crate::helpers::init_server_client;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_run_subscription_unsubscription() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = init_server_client().await;

    let mut stream = client.sub().await.unwrap();

    sleep(Duration::from_millis(100)).await;

    if let Some(msg) = stream.message().await? {
        assert_eq!("Subscription stream data", msg.message);
    } else {
        panic!("No subscription message received");
    }

    client.unsub().await.unwrap();

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_client_not_found_for_sub_with_wrong_client_id()
-> Result<(), Box<dyn std::error::Error>> {
    let mut client = init_server_client().await;

    //update client_id on client side
    client.client_id = Uuid::new_v4();

    let result = client.sub().await;
    assert!(result.is_err());
    let binding = result.unwrap_err();
    let your_error = binding.downcast_ref::<Status>();
    assert!(your_error.is_some());
    assert_eq!("Client not found", your_error.unwrap().message());

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn should_client_not_found_for_unsub_with_wrong_client_id()
-> Result<(), Box<dyn std::error::Error>> {
    let mut client = init_server_client().await;

    client.sub().await.unwrap();

    client.client_id = Uuid::new_v4();

    let result = client.unsub().await;

    assert!(result.is_err());
    let binding = result.unwrap_err();
    let your_error = binding.downcast_ref::<Status>();
    assert!(your_error.is_some());
    assert_eq!("Client not found", your_error.unwrap().message());

    Ok(())
}

/*
Other test cases:
    Sub -> Sub is handled in the product client with CancellationToken
    Unsub w/o Sub is handled in the product client with CancellationToken
*/
