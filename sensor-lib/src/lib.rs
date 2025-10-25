#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorPacket {
    pub sensor_id: u8,
    pub latitude: f64,
    pub longitude: f64,
    pub environment: Environment,
    pub fingerprint: u64,
    pub rssi: i32,
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
