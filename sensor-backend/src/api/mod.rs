use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;

mod sensors;

use crate::AppState;

pub fn api(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/sensors", sensors::routes())
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "Healthy")
}
