pub mod config;
pub mod errors;
pub mod mail;
pub mod models;

use axum::{routing::get, Router};
use mail::Mailer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::AppState,
    models::{
        asset::build_assets_router, comment::build_comments_router, ticket::build_tickets_router,
    },
};

pub async fn build_router(mailer: Option<Mailer>) -> Router {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,endtoend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let state = AppState::new(mailer).await;
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
