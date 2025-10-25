use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SensorPacket {
    pub sensor_id: u8,
    pub rssi: i16,
    pub fingerprint: u32,
}
