use std::sync::Arc;

use sensor_lib::SensorPacket;
use tokio::net::UdpSocket;
use tracing::{error, info};

use crate::sensors::SensorService;

pub async fn run_packet_listener(sensor_service: Arc<SensorService>) -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:3000").await?;
    info!("Running UDP listener on {}", socket.local_addr()?);

    let mut buf = [0u8; 1024];

    loop {
        let (len, _) = socket.recv_from(&mut buf).await?;

        let data = &buf[..len];

        match postcard::from_bytes::<SensorPacket>(data) {
            Ok(packet) => {
                info!(
                    "Received packet from sensor {} (Y: {}, X: {}, Latitude: {}, Longitude: {}): RSSI {}, Fingerprint {}",
                    packet.sensor_id,
                    packet.y,
                    packet.x,
                    packet.latitude,
                    packet.longitude,
                    packet.rssi,
                    packet.fingerprint
                );

                sensor_service
                    .add_sensor(
                        packet.sensor_id,
                        packet.y,
                        packet.x,
                        packet.latitude,
                        packet.longitude,
                        packet.environment,
                    )
                    .await;

                sensor_service
                    .add_measurement(packet.fingerprint, packet.sensor_id, packet.rssi)
                    .await;
            }
            Err(e) => error!("Failed to deserialize packet: {}", e),
        }
    }
}
