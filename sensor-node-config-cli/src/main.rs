use clap::Parser;
use sensor_lib::{Environment, SensorConfig, SensorPacket};
use std::error::Error;
use std::io::Write;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    #[clap(long)]
    ssid: String,
    #[clap(long)]
    password: String,
    #[clap(long)]
    host: Ipv4Addr,
    #[clap(long)]
    port: u16,

    #[clap(long)]
    sensor_id: u8,

    #[clap(long)]
    x: f64,
    #[clap(long)]
    y: f64,
    #[clap(long)]
    latitude: f64,
    #[clap(long)]
    longitude: f64,

    #[clap(long)]
    serial_port: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut serial_port = serialport::new(cli.serial_port, 9600).open()?;

    let ssid =
        heapless::String::<32>::from_str(&cli.ssid.as_str()[..cli.ssid.as_str().len().min(32)])?;
    let password = heapless::String::<63>::from_str(
        &cli.password.as_str()[..cli.password.as_str().len().min(63)],
    )?;

    let config_data = postcard::to_allocvec(&SensorConfig {
        collector_network_ssid: ssid,
        collector_network_password: password,
        collector_service_ip: cli.host,
        collector_service_port: cli.port,
        sensor_id: cli.sensor_id,
        x: cli.x,
        y: cli.y,
        latitude: cli.latitude,
        longitude: cli.longitude,
        environment: Environment::FreeSpace,
    })?;

    serial_port.write(&config_data)?;

    Ok(())
}
