use crate::{
    client::Config,
    proto::{
        InitRequest, SubscribeRequest, UnsubscribeRequest, cli_service_client::CliServiceClient,
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
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
};

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

enum ClientEvent {
    InputChar(char),
    Backspace,
    Enter,
    ReplInput(String),
    SubscriptionMsg(String),
    Log(String),
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

    let state = Arc::new(Mutex::new(State {
        current_cancel: None,
    }));

    let mut repl_history: Vec<String> = vec![];
    let mut stream_msgs: Vec<String> = vec![];
    let mut logs: Vec<String> = vec![];
    let mut current_input = String::new();

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
                                                match key.code {
                                                    KeyCode::Char(c) => { tx_repl.send(ClientEvent::InputChar(c)).await.unwrap()}
                                                    KeyCode::Backspace => {tx_repl.send(ClientEvent::Backspace).await.unwrap()}
                                                    KeyCode::Enter => tx_repl.send(ClientEvent::Enter).await.unwrap(),
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
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(f.size());

            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(top_chunks[1]);

            let stream_block = Paragraph::new(stream_msgs.join("\n"))
                .block(Block::default().title("Stream").borders(Borders::ALL));
            f.render_widget(stream_block, top_chunks[0]);

            let history_block = Paragraph::new(repl_history.join("\n"))
                .block(Block::default().title("Repl reponse").borders(Borders::ALL));
            f.render_widget(history_block, right_chunks[0]);

            let logs_block = Paragraph::new(logs.join("\n"))
                .block(Block::default().title("Logs").borders(Borders::ALL));
            f.render_widget(logs_block, right_chunks[1]);

            let input_line = Paragraph::new(Span::styled(
                format!("repl> {}", current_input),
                Style::default().fg(Color::Yellow),
            ))
            .block(Block::default().borders(Borders::TOP));
            f.render_widget(input_line, chunks[1]);
        })?;

        let tx_stream = tx.clone();
        tokio::select! {
            Some(ev) = rx.recv() => {
                match ev {
                    ClientEvent::InputChar(c) => {
                        current_input.push(c);
                    },
                    ClientEvent::Backspace => {
                        current_input.pop();
                    },
                    ClientEvent::Enter => {
                        let line = current_input.drain(..).collect::<String>();
                        tx.send(ClientEvent::ReplInput(line)).await.unwrap();
                        current_input.clear();
                    },
                    ClientEvent::ReplInput(line) => {
                        repl_history.push(line.clone());
                        let tx_log = tx.clone();
                        match line.as_str() {
                            "exit" | "quit" => {
                                let _ = tx_log.send(ClientEvent::Log("Exiting...".to_string())).await;
                                break;
                            }
                            "sub" => {
                                if state.lock().await.current_cancel.is_some() {
                                    let _ = tx_log.send(ClientEvent::Log("Already subscribed. Please unsubscribe first.".to_string())).await;
                                } else {
                                    let cancel = CancellationToken::new();
                                    {
                                        let mut s = state.lock().await;
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
                                let mut s = state.lock().await;
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
                            _ => {
                                let _ = tx_log.send(ClientEvent::Log("Unknown command. Use: sub | unsub | call <msg> | quit".to_string())).await;
                            }
                        }

                    },
                    ClientEvent::SubscriptionMsg(msg) => stream_msgs.push(msg),
                    ClientEvent::Log(msg) => logs.push(msg),
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
