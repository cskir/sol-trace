use crate::{
    client::Config,
    proto::{
        CallRequest, InitRequest, SubscribeRequest, UnsubscribeRequest,
        cli_service_client::CliServiceClient,
    },
};
use clap::Parser;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::Mutex;
use tokio::{
    task,
    time::{Duration, sleep},
};
use tokio_util::sync::CancellationToken;
use tonic::{Request, metadata::MetadataValue};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "Sol-trace client")]
#[command(about = "Solana wallet tracker client with REPL", long_about = None)]
pub struct CliArgs {
    #[arg(long, short, default_value = "http://127.0.0.1:50051")]
    pub addr: String,

    #[arg(long, short)]
    pub config: String,
}

struct State {
    current_cancel: Option<CancellationToken>,
}

pub async fn run_cli_client(cli: CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CliServiceClient::connect(cli.addr.clone()).await?;

    let init_request = InitRequest::build(Config::load(cli.config.as_str())?)?;

    let client_id = match client.init(Request::new(init_request)).await {
        Ok(response) => {
            let client_id = Uuid::parse_str(response.into_inner().client_id.as_str())
                .map_err(|e| format!("Failed to parse client_id: {}", e))?;
            client_id
        }
        Err(e) => return Err(format!("Error: {:?}", e).into()),
    };

    println!("Client initialized with ID: {}", client_id);

    let state = Arc::new(Mutex::new(State {
        current_cancel: None,
    }));

    let mut client_clone = client.clone();

    //TODO add crossterm & rustyline for better UX

    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.eq_ignore_ascii_case("quit") || line.eq_ignore_ascii_case("exit") {
            println!("Exiting...");
            break;
        } else if line.eq_ignore_ascii_case("sub") {
            if state.lock().await.current_cancel.is_some() {
                println!("Already subscribed. Please unsubscribe first.");
                continue;
            }

            let cancel = CancellationToken::new();
            {
                let mut s = state.lock().await;
                s.current_cancel = Some(cancel.clone());
            }

            let mut subscribe_request = Request::new(SubscribeRequest {});
            subscribe_request.metadata_mut().insert(
                "client-id",
                MetadataValue::try_from(client_id.clone().to_string())?,
            );

            //note: grpc stream will close if all the sender (mpsc-tx in the server) dropped
            let mut stream = client.subscribe(subscribe_request).await?.into_inner();

            task::spawn(async move {
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => {
                            println!("Subscription stopped by user");
                            break;
                        }
                        _ = sleep(Duration::from_millis(500)) => {
                            while let Ok(Some(item)) = stream.message().await {
                             println!("[sub] {}", item.message);
                         }
                        }
                    }
                }
            });
        } else if line.eq_ignore_ascii_case("unsub") {
            let mut s = state.lock().await;
            if let Some(cancel) = s.current_cancel.take() {
                let mut unsub_request = Request::new(UnsubscribeRequest {});
                unsub_request.metadata_mut().insert(
                    "client-id",
                    MetadataValue::try_from(client_id.clone().to_string())?,
                );

                match client_clone.unsubscribe(unsub_request).await {
                    Ok(resp) => println!("[unsub] {}", resp.into_inner().message),
                    Err(e) => eprintln!("Error: {:?}", e),
                }

                cancel.cancel();
            } else {
                println!("No active subscription to unsubscribe.");
            }
        } else if let Some(rest) = line.strip_prefix("call ") {
            let payload = rest.to_string();
            match client_clone
                .call(Request::new(CallRequest { payload }))
                .await
            {
                Ok(resp) => println!("[call response] {}", resp.into_inner().reply),
                Err(e) => eprintln!("Error: {:?}", e),
            }
        } else {
            println!("Unknown command. Use: sub | unsub | call <msg> | quit");
        }
    }

    Ok(())
}
