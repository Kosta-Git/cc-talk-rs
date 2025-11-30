use std::time::Duration;

use cc_talk_cli::{
    Cli,
    Commands::{Hopper, Selector},
    coinselector, hopper,
};
use cc_talk_tokio_host::transport::{retry::RetryConfig, tokio_transport::CcTalkTokioTransport};
use clap::Parser;
use tokio::sync::mpsc;
use tracing::info;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_file(false)
        .with_line_number(false)
        .with_thread_ids(false)
        .with_target(false)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("tracing subscriber should work");

    let cli = Cli::parse();
    let timeout = Duration::from_millis(cli.timeout);

    let (tx, rx) = mpsc::channel(8);
    let transport = CcTalkTokioTransport::new(
        rx,
        cli.sock.clone(),
        timeout,
        timeout,
        RetryConfig::default(),
        !cli.no_echo,
    );

    info!(
        "Transport initialized using sock: '{}' with {}ms timeout and echo support '{}'",
        cli.sock, cli.timeout, !cli.no_echo
    );

    let handle = tokio::spawn(async move {
        if let Err(e) = transport.run().await {
            tracing::error!("Error running transport: {}", e);
        }
    });
    tokio::time::sleep(timeout).await;
    {
        match &cli.command {
            Hopper { address, action } => hopper::handler(tx, *address, action).await,
            Selector { address, action } => coinselector::handler(tx, *address, action).await,
        }
        handle.abort();
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}
