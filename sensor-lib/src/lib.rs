#![no_std]

use core::net::Ipv4Addr;
use serde::{Deserialize, Serialize};

const MAX_SSID_LENGTH: usize = 32;
const MAX_WIFI_PASSWORD_LENGTH: usize = 63;

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorConfig {
    pub collector_network_ssid: heapless::String<MAX_SSID_LENGTH>,
    pub collector_network_password: heapless::String<MAX_WIFI_PASSWORD_LENGTH>,

    pub collector_service_ip: Ipv4Addr,
    pub collector_service_port: u16,

    pub sensor_id: u8,
    pub y: f64,
    pub x: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub environment: Environment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorPacket {
    pub sensor_id: u8,
    pub y: f64,
    pub x: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub environment: Environment,
    pub fingerprint: u64,
    pub rssi: i8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Environment {
    FreeSpace,
    UrbanArea,
    ShadowedUrban,
    InBuildingLOS,
    ObstructedInBuilding,
    ObstructedInFactory,
    Custom(f64),
}

impl Environment {
    pub fn as_f64(&self) -> f64 {
        match self {
            Environment::FreeSpace => 2.0,
            Environment::UrbanArea => 2.7,
            Environment::ShadowedUrban => 3.0,
            Environment::InBuildingLOS => 1.6,
            Environment::ObstructedInBuilding => 4.0,
            Environment::ObstructedInFactory => 2.0,
            Environment::Custom(n) => *n,
        }
    }
}
