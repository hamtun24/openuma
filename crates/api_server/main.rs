use api_server::create_router;
use axum::serve;
use std::net::SocketAddr;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = create_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    println!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    serve(listener, router).await.unwrap();
}
