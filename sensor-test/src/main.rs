use std::net::UdpSocket;

use sensor_lib::Environment::FreeSpace;
use sensor_lib::SensorPacket;

fn main() -> anyhow::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("0.0.0.0:3000")?;

    let packet = SensorPacket {
        sensor_id: 1,
        y: 0.0,
        x: 0.0,
        latitude: 50.56484445024739,
        longitude: 9.684520461933687,
        rssi: -50,
        fingerprint: 0xABC123,
        environment: FreeSpace,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    let packet = SensorPacket {
        sensor_id: 2,
        y: 0.0,
        x: 4.0,
        latitude: 50.56494466501721,
        longitude: 9.684520461933687,
        rssi: -50,
        fingerprint: 0xABC123,
        environment: FreeSpace,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    let packet = SensorPacket {
        sensor_id: 3,
        y: 3.0,
        x: 2.0,
        latitude: 50.5648945576323,
        longitude: 9.684697512562593,
        rssi: -50,
        fingerprint: 0xABC123,
        environment: FreeSpace,
    };
    let bytes = postcard::to_allocvec(&packet)?;

    socket.send(&bytes)?;

    Ok(())
}
