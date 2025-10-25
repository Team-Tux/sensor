use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(index))
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let sensors = state.sensor_service.get_sensors().await;

    (StatusCode::OK, Json(sensors))
}
