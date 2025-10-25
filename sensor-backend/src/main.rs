use axum::Router;
use axum::routing::get;
use sensor_lib::SensorPacket;
use tokio::net::{TcpListener, UdpSocket};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tokio::spawn(async {
        if let Err(e) = run_packet_listener().await {
            error!("Failed to run UDP listener: {}", e);
        }
    });

    let app = Router::new().route("/health", get(|| async { "Healthy" }));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    info!("Starting HTTP server on {}", listener.local_addr()?);

    Ok(axum::serve(listener, app).await?)
}

async fn run_packet_listener() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:3000").await?;
    info!("Running UDP listener on {}", socket.local_addr()?);

    let mut buf = [0u8; 1024];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;

        let data = &buf[..len];

        match postcard::from_bytes::<SensorPacket>(data) {
            Ok(packet) => {
                info!(
                    "Received packet from sensor {}: RSSI {}, Fingerprint {}",
                    packet.sensor_id, packet.rssi, packet.fingerprint
                );
            }
            Err(e) => error!("Failed to deserialize packet: {}", e),
        }
    }
}
