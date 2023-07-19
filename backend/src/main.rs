use std::net::SocketAddr;
use tinytickets_backend::build_router;

#[tokio::main]
async fn main() {
    // run it with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let app = build_router(None).await.into_make_service();
    tracing::info!("Tiny tickets backend is listening on {}", addr);
    axum::Server::bind(&addr).serve(app).await.unwrap();
}
