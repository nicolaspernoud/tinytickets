use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::AppState,
    models::{
        asset::build_assets_router, comment::build_comments_router, ticket::build_tickets_router,
    },
};

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
        .nest("/api/assets", build_assets_router())
        .nest("/api/comments", build_comments_router())
        .nest("/api/tickets", build_tickets_router())
        .with_state(state);

    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
