use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppState;

mod config;
mod errors;
mod mail;
mod models;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tinytickets_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new().await;

    // build our application with some routes
    let app = Router::new()
        .route(
            "/api/app-title",
            get(|| async {
                std::env::var("APP_TITLE").unwrap_or_else(|_| String::from("Tiny Tickets"))
            }),
        )
        .with_state(state);

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
