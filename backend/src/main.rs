use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::AppState,
    models::{
        asset::build_assets_router, comment::build_comments_router, ticket::build_tickets_router,
    },
};

#[tokio::main]
async fn main() {
    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(build_router().await.into_make_service())
        .await
        .unwrap();
}

async fn build_router() -> Router {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tinytickets_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let state = AppState::new().await;
    Router::new()
        .route(
            "/api/app-title",
            get(|| async {
                std::env::var("APP_TITLE").unwrap_or_else(|_| String::from("Tiny Tickets"))
            }),
        )
        .nest("/api/assets", build_assets_router())
        .nest("/api/comments", build_comments_router())
        .nest("/api/tickets", build_tickets_router())
        .with_state(state)
}
