mod handler;
mod state;

pub use state::AppState;

use axum::{routing::get, Router};

/// Build the axum router for the WebSocket server.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/ws", get(handler::ws_handler))
        .with_state(state)
}

/// Start the WebSocket server on the given port.
pub async fn serve(state: AppState, port: u16) -> std::io::Result<()> {
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
