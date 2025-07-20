use crate::config::{MDX_FILES, static_path};
use crate::handlers::{handle_lucky, handle_query};
use crate::indexing::indexing;

use axum::{
    Router,
    routing::{get, post},
};
use std::error::Error;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod indexing;
mod lucky;
mod mdict;
mod query;
mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 初始化日志系统
    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    indexing(MDX_FILES, false);

    let static_dir = ServeDir::new(static_path()?);

    let app = Router::new()
        .route("/query", post(handle_query))
        .route("/lucky", get(handle_lucky))
        .fallback_service(static_dir)
        .layer(TraceLayer::new_for_http());

    let port = 8181;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8181").await.unwrap();

    info!("app serve on http://localhost:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
