#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorPacket {
    pub sensor_id: u8,
    pub latitude: f64,
    pub longitude: f64,
    pub rssi: i32,
    pub fingerprint: u64,
}
