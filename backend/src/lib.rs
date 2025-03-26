pub mod config;
pub mod errors;
pub mod mail;
pub mod models;

use axum::{
    Router,
    routing::{get, get_service},
};
use mail::Mailer;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::AppState,
    models::{
        asset::build_assets_router, comment::build_comments_router, ticket::build_tickets_router,
    },
};

pub async fn build_router(mailer: Option<Mailer>) -> Router {
    let debug_mode = std::env::var("DEBUG_MODE").unwrap_or_default() == "true";
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                if debug_mode {
                    "info,tinytickets_backend=debug"
                } else {
                    "info,endtoend=debug"
                }
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let state = AppState::new(mailer, debug_mode).await;
    let debug_mode = state.config.debug_mode;
    let router = Router::new()
        .route(
            "/api/app-title",
            get(|| async {
                std::env::var("APP_TITLE").unwrap_or_else(|_| String::from("Tiny Tickets"))
            }),
        )
        .nest("/api/assets", build_assets_router())
        .nest("/api/comments", build_comments_router())
        .nest("/api/tickets", build_tickets_router())
        .fallback_service(get_service(ServeDir::new("web")))
        .with_state(state);
    if debug_mode {
        router.layer(CorsLayer::permissive())
    } else {
        router
    }
}
