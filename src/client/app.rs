use crate::{
    client::{AppState, Config, Panel, SharedState, scroll_down, scroll_up, ui},
    proto::{
        GetTradeRequest, HoldingsRequest, InitRequest, SubscribeRequest, UnsubscribeRequest,
        cli_service_client::CliServiceClient,
    },
};
use clap::Parser;
use std::sync::Arc;
use tokio::{
    sync::{Mutex, mpsc},
    task,
    time::{Duration, sleep},
};
use tokio_util::sync::CancellationToken;
use tonic::{Request, metadata::MetadataValue};
use uuid::Uuid;

use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

#[derive(Parser)]
#[command(name = "Sol-trace client")]
#[command(about = "Solana wallet tracker client with REPL", long_about = None)]
pub struct CliArgs {
    #[arg(long, short, default_value = "http://127.0.0.1:50051")]
    pub addr: String,

    #[arg(long, short)]
    pub config: String,
}

enum ClientEvent {
    InputChar(char),
    Backspace,
    Enter,
    ReplInput(String),
    SubscriptionMsg(String),
    Log(String),
    Tab,
    ScrollDown,
    ScrollUp,
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

    let (tx, mut rx) = mpsc::channel::<ClientEvent>(100);

    let tx_log = tx.clone();
    let _ = tx_log
        .send(ClientEvent::Log(format!(
            "Client initialized with ID: {}",
            client_id
        )))
        .await;

    let shared_state = Arc::new(Mutex::new(SharedState {
        current_cancel: None,
    }));

    let mut state = AppState::default();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let tx_repl = tx.clone();
    let cancel_repl = CancellationToken::new();
    let cancel_repl_task = cancel_repl.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                            _ = cancel_repl_task.cancelled() => break,
                            _ = async {
                                if event::poll(Duration::from_millis(50)).unwrap() {
                                    if let Ok(event) = event::read() {
                                        if let Event::Key(key) = event {
                                            if key.kind == KeyEventKind::Press {
                                            //tx_repl.send(ClientEvent::Log(format!("Error: {:?}", key))).await.unwrap();
                                                match key.code {
                                                    KeyCode::Char(c)  => { tx_repl.send(ClientEvent::InputChar(c)).await.unwrap()}
                                                    KeyCode::Backspace if key.kind == KeyEventKind::Press => {tx_repl.send(ClientEvent::Backspace).await.unwrap()}
                                                    KeyCode::Enter if key.kind == KeyEventKind::Press => tx_repl.send(ClientEvent::Enter).await.unwrap(),
                                                    KeyCode::Tab if key.kind == KeyEventKind::Press => tx_repl.send(ClientEvent::Tab).await.unwrap(),
                                                    KeyCode::Up => tx_repl.send(ClientEvent::ScrollUp).await.unwrap(),
                                                    KeyCode::Down => tx_repl.send(ClientEvent::ScrollDown).await.unwrap(),
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            } => {}
            }
        }
    });

    loop {
        terminal.draw(|f| ui(f, &mut state))?;

        let tx_stream = tx.clone();

        tokio::select! {
            Some(ev) = rx.recv() => {
                match ev {
                    ClientEvent::Tab => {
                        state.focused = match state.focused {
                            Panel::Stream => Panel::History,
                            Panel::History => Panel::Log,
                            Panel::Log => Panel::Repl,
                            Panel::Repl => Panel::Stream,
                        };
                    },
                    ClientEvent::ScrollUp => {
                        match state.focused {
                            Panel::Stream => scroll_up(&mut state.stream_scroll),
                            Panel::Log => scroll_up(&mut state.log_scroll),
                            Panel::History => scroll_up(&mut state.history_scroll),
                            _ => {}
                        }
                    },
                    ClientEvent::ScrollDown => {
                        match state.focused {
                            Panel::Stream => scroll_down(&mut state.stream_scroll, state.stream_list.len(), state.last_heights.stream),
                            Panel::Log => scroll_down(&mut state.log_scroll, state.logs.len(), state.last_heights.log),
                            Panel::History => scroll_down(&mut state.history_scroll, state.history_list.len(), state.last_heights.history),
                            _ => {}
                        }
                    },
                    ClientEvent::InputChar(c) => {
                        state.repl.push(c);
                    },
                    ClientEvent::Backspace => {
                        state.repl.pop();
                    },
                    ClientEvent::Enter => {
                        let line = state.repl.drain(..).collect::<String>();
                        tx.send(ClientEvent::ReplInput(line)).await.unwrap();
                        state.repl.clear();
                    },
                    ClientEvent::ReplInput(line) => {
                        state.history_list.push(line.clone());
                        let tx_log = tx.clone();
                        const TX_PREFIX: &str = "tx ";
                        if line.as_str().starts_with(TX_PREFIX) {
                            let signature = &line.as_str()[TX_PREFIX.len()..];
                            let mut client_clone = client.clone();

                            let mut get_tx_request = Request::new(GetTradeRequest {signature: signature.to_string()});
                            get_tx_request.metadata_mut().insert(
                                "client-id",
                                MetadataValue::try_from(client_id.clone().to_string())?,
                            );

                            match client_clone.get_trade(get_tx_request).await {
                                Ok(resp ) => {
                                    match resp.into_inner().trade {
                                        Some(trade) => {
                                            for item in trade.to_string_lines().into_iter() {
                                                    state.history_list.push(item);
                                                }
                                        }
                                        None => state.history_list.push("No trade detected.".to_string())
                                    }
                                },
                                Err(e) => tx_log.send(ClientEvent::Log(format!("Error: {e}"))).await?,
                            }
                        }
                        else {
                            match line.as_str() {
                                "exit" | "quit" => {
                                    let _ = tx_log.send(ClientEvent::Log("Exiting...".to_string())).await;
                                    break;
                                }
                                "sub" => {
                                    if shared_state.lock().await.current_cancel.is_some() {
                                        let _ = tx_log.send(ClientEvent::Log("Already subscribed. Please unsubscribe first.".to_string())).await;
                                    } else {
                                        let cancel = CancellationToken::new();
                                        {
                                            let mut s = shared_state.lock().await;
                                            s.current_cancel = Some(cancel.clone());
                                        }

                                        let _ = tx_log.send(ClientEvent::Log("Subscription request has been sent".to_string())).await;
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
                                                        tx_log.send(ClientEvent::Log("Subscription stopped by user".to_string())).await.unwrap();
                                                        break;
                                                    }
                                                    _ = sleep(Duration::from_millis(500)) => {
                                                        while let Ok(Some(item)) = stream.message().await {
                                                            tx_stream.send(ClientEvent::SubscriptionMsg(item.message)).await.unwrap();
                                                    }
                                                    }
                                                }
                                            }
                                        });
                                    }
                                } "unsub" => {
                                    let _ = tx_log.send(ClientEvent::Log("Unsubscribing...".to_string())).await;
                                    let mut client_clone = client.clone();
                                    let mut s = shared_state.lock().await;
                                    if let Some(cancel) = s.current_cancel.take() {
                                        let mut unsub_request = Request::new(UnsubscribeRequest {});
                                        unsub_request.metadata_mut().insert(
                                            "client-id",
                                            MetadataValue::try_from(client_id.clone().to_string())?,
                                        );

                                        match client_clone.unsubscribe(unsub_request).await {
                                            Ok(resp) => tx_log.send(ClientEvent::Log(resp.into_inner().message)).await?,
                                            Err(e) => tx_log.send(ClientEvent::Log(format!("Error: {e}"))).await?,
                                        }

                                        cancel.cancel();
                                    } else {
                                        tx_log.send(ClientEvent::Log("No active subscription to unsubscribe.".to_string())).await?;
                                    }
                                }
                                "hold" => {
                                    let _ = tx_log.send(ClientEvent::Log("Holdings request has been sent".to_string())).await;
                                    let mut client_clone = client.clone();

                                    let mut holdings_request = Request::new(HoldingsRequest {});
                                    holdings_request.metadata_mut().insert(
                                        "client-id",
                                        MetadataValue::try_from(client_id.clone().to_string())?,
                                    );

                                    match client_clone.holdings(holdings_request).await {
                                        Ok(resp ) => {
                                            let balances :Vec<String> = resp.into_inner().holdings.into_iter().map(|h| {
                                                h.to_string()
                                            }).collect();
                                            if balances.is_empty() {
                                                state.history_list.push("No holdings found.".to_string());
                                            } else {
                                                state.history_list.push("Holdings:".to_string());
                                                for item in balances.into_iter() {
                                                    state.history_list.push(item);
                                                }
                                            };
                                        },
                                        Err(e) => tx_log.send(ClientEvent::Log(format!("Error: {e}"))).await?,
                                    }
                                }
                                _ => {
                                    let _ = tx_log.send(ClientEvent::Log("Unknown command. Use: sub | unsub | hold | exit | quit".to_string())).await;
                                }
                            }
                        }
                    },
                    ClientEvent::SubscriptionMsg(msg) => state.stream_list.push(msg),
                    ClientEvent::Log(msg) => state.logs.push(msg),
                }
            }
        }
    }

    cancel_repl.cancel();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show)?;
    terminal.show_cursor().ok();

    Ok(())
}
