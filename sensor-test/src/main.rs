use sensor_lib::Environment::FreeSpace;
use sensor_lib::SensorPacket;
use std::net::UdpSocket;

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("0.0.0.0:3000")?;

    let packet = SensorPacket {
        sensor_id: 1,
        latitude: 0.0,
        longitude: 0.0,
        rssi: -50,
        fingerprint: 0xABC123,
        environment: FreeSpace,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    let packet = SensorPacket {
        sensor_id: 2,
        latitude: 0.0,
        longitude: 4.0,
        rssi: -50,
        fingerprint: 0xABC123,
        environment: FreeSpace,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    let packet = SensorPacket {
        sensor_id: 3,
        latitude: 3.0,
        longitude: 2.0,
        rssi: -50,
        fingerprint: 0xABC123,
        environment: FreeSpace,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    Ok(())
}
