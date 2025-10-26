use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use tokio::time::interval;
use tracing::error;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let sensors = state.sensor_service.get_sensors().await;

    (StatusCode::OK, Json(sensors))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut interval = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let sensors = state.sensor_service.get_sensors().await;

                let data = match serde_json::to_string(&sensors) {
                    Ok(json) => Utf8Bytes::from(json),
                    Err(e) => {
                        error!("Failed to serialize sensors: {}", e);
                        continue;
                    }
                };

                if let Err(e) = socket.send(Message::Text(data)).await {
                    error!("Failed to send sensors: {}", e);
                    break;
                }
            }

            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Close(_)) | Err(_) => {
                        break;
                    }
                    Ok(_) => {
                        continue;
                    }
                }
            }

            else => {
                break;
            }
        }
    }
}
