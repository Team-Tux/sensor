use sensor_lib::SensorPacket;
use std::net::UdpSocket;

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("0.0.0.0:3000")?;

    let packet = SensorPacket {
        sensor_id: 1,
        rssi: -26,
        fingerprint: 0xABC123,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    Ok(())
}
