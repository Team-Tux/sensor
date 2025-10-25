use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::rssi::trilaterate;

const MAX_MEASUREMENT_AGE: Duration = Duration::from_secs(60);

#[derive(Serialize)]
pub struct Sensor {
    pub id: u8,
    pub latitude: f64,
    pub longitude: f64,
}

pub struct SensorCandidate {
    pub latitude: f64,
    pub longitude: f64,
    pub rssi: i32,
}

#[derive(Serialize)]
pub struct Trilateration {
    pub fingerprint: u64,
    pub latitude: f64,
    pub longitude: f64,
}

type MeasurementsMap = HashMap<u64, HashMap<u8, (i32, Instant)>>;

pub struct SensorService {
    sensors: RwLock<HashMap<u8, (f64, f64)>>,
    measurements: RwLock<MeasurementsMap>,
    trilaterations: RwLock<HashMap<u64, (f64, f64)>>,
}

impl SensorService {
    pub fn new() -> Self {
        Self {
            sensors: RwLock::new(HashMap::new()),
            measurements: RwLock::new(HashMap::new()),
            trilaterations: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_sensor(&self, id: u8, latitude: f64, longitude: f64) {
        let mut lock = self.sensors.write().await;

        lock.insert(id, (latitude, longitude));
    }

    pub async fn get_sensors(&self) -> Vec<Sensor> {
        let lock = self.sensors.read().await;

        lock.iter()
            .map(|(&id, &(latitude, longitude))| Sensor {
                id,
                latitude,
                longitude,
            })
            .collect()
    }

    pub async fn add_measurement(&self, fingerprint: u64, sensor_id: u8, rssi: i32) {
        let mut lock = self.measurements.write().await;

        let now = Instant::now();

        lock.entry(fingerprint.clone())
            .or_default()
            .insert(sensor_id, (rssi, now));

        if let Some(sensors) = lock.get(&fingerprint) {
            if sensors.len() == 3 {
                let s_lock = self.sensors.read().await;

                let candidates: Option<Vec<SensorCandidate>> = sensors
                    .iter()
                    .map(|(id, (rssi, _))| {
                        s_lock.get(id).map(|pos| SensorCandidate {
                            latitude: pos.0,
                            longitude: pos.1,
                            rssi: *rssi,
                        })
                    })
                    .collect();

                drop(s_lock);

                if let Some(candidates) = candidates {
                    let (latitude, longitude) =
                        trilaterate(&candidates[0], &candidates[1], &candidates[2]).await;

                    let mut t_lock = self.trilaterations.write().await;
                    t_lock.insert(fingerprint, (latitude, longitude));
                    drop(t_lock);

                    lock.remove(&fingerprint);
                }
            }
        }

        lock.retain(|_, sensors| {
            if let Some(timestamp) = sensors.values().map(|(_, timestamp)| *timestamp).max() {
                if timestamp < now.checked_sub(MAX_MEASUREMENT_AGE).unwrap_or(now) {
                    return false;
                }
            }
            return true;
        });
    }

    pub async fn get_trilaterations(&self) -> Vec<Trilateration> {
        let lock = self.trilaterations.read().await;

        lock.iter()
            .map(|(&fingerprint, &(latitude, longitude))| Trilateration {
                fingerprint,
                latitude,
                longitude,
            })
            .collect()
    }
}
