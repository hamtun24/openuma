pub mod handlers;

use axum::{
    routing::{get, post},
    Router,
};

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/v1/probe", get(handlers::probe))
        .route("/api/v1/configure", post(handlers::configure))
        .route("/api/v1/benchmark", post(handlers::run_benchmark))
        .route("/api/v1/profiles", get(handlers::list_profiles))
}
