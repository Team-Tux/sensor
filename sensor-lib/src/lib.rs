#![no_std]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorPacket {
    pub sensor_id: u8,
    pub pos_x: f64,
    pub pos_y: f64,
    pub rssi: i32,
    pub fingerprint: u64,
}
