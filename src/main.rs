use tokio::net::TcpListener;
use tracing::info;

use crate::{handlers::connections::handle_connection, types::AppState};

mod handlers;
mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let addr = "127.0.0.1:4000";
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket server running on ws://{}", addr);
    let app_state = AppState::new();

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);
        tokio::spawn(handle_connection(stream, app_state.clone()));
    }

    Ok(())
}
