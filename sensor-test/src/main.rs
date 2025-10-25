use sensor_lib::SensorPacket;
use std::net::UdpSocket;

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("0.0.0.0:3000")?;

    let packet = SensorPacket {
        sensor_id: 1,
        pos_x: 0.0,
        pos_y: 0.0,
        rssi: -50,
        fingerprint: 0xABC123,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    let packet = SensorPacket {
        sensor_id: 2,
        pos_x: 4.0,
        pos_y: 0.0,
        rssi: -50,
        fingerprint: 0xABC123,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    let packet = SensorPacket {
        sensor_id: 3,
        pos_x: 2.0,
        pos_y: 3.0,
        rssi: -50,
        fingerprint: 0xABC123,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    Ok(())
}
