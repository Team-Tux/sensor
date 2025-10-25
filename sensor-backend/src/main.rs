use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::api::api;
use crate::listener::run_packet_listener;
use crate::sensors::SensorService;

mod api;
mod listener;
mod rssi;
mod sensors;

#[derive(Clone)]
pub struct AppState {
    sensor_service: Arc<SensorService>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let sensor_service = Arc::new(SensorService::new());

    let sensor_service_clone = sensor_service.clone();
    tokio::spawn(async {
        if let Err(e) = run_packet_listener(sensor_service_clone).await {
            error!("Failed to run UDP listener: {}", e);
        }
    });

    let state = AppState { sensor_service };

    let app = Router::new().nest("/api", api(state));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    info!("Starting HTTP server on {}", listener.local_addr()?);

    Ok(axum::serve(listener, app).await?)
}
