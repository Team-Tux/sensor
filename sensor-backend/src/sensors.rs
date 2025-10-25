use crate::rssi::trilaterate;
use sensor_lib::Environment;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;

const MAX_MEASUREMENT_AGE: Duration = Duration::from_secs(60);

#[derive(Clone, Serialize)]
pub struct Sensor {
    pub id: u8,
    pub latitude: f64,
    pub longitude: f64,
    pub environment: Environment,
}

pub struct SensorCandidate {
    pub latitude: f64,
    pub longitude: f64,
    pub environment: Environment,
    pub rssi: i32,
}

#[derive(Clone, Serialize)]
pub struct Trilateration {
    pub fingerprint: u64,
    pub latitude: f64,
    pub longitude: f64,
}

type MeasurementsMap = HashMap<u64, HashMap<u8, (i32, Instant)>>;

pub struct SensorService {
    sensors: RwLock<HashMap<u8, Sensor>>,
    measurements: RwLock<MeasurementsMap>,
    trilaterations: RwLock<HashMap<u64, Trilateration>>,
}

impl SensorService {
    pub fn new() -> Self {
        Self {
            sensors: RwLock::new(HashMap::new()),
            measurements: RwLock::new(HashMap::new()),
            trilaterations: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_sensor(
        &self,
        id: u8,
        latitude: f64,
        longitude: f64,
        environment: Environment,
    ) {
        let mut lock = self.sensors.write().await;

        let sensor = Sensor {
            id,
            latitude,
            longitude,
            environment,
        };

        lock.insert(id, sensor);
    }

    pub async fn get_sensors(&self) -> Vec<Sensor> {
        let lock = self.sensors.read().await;

        lock.values().cloned().collect()
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
                        s_lock.get(id).map(|sensor| SensorCandidate {
                            latitude: sensor.latitude,
                            longitude: sensor.longitude,
                            environment: sensor.environment,
                            rssi: *rssi,
                        })
                    })
                    .collect();

                drop(s_lock);

                if let Some(candidates) = candidates {
                    let (latitude, longitude) =
                        trilaterate(&candidates[0], &candidates[1], &candidates[2]).await;

                    let mut t_lock = self.trilaterations.write().await;

                    let trilateration = Trilateration {
                        fingerprint,
                        latitude,
                        longitude,
                    };

                    t_lock.insert(fingerprint, trilateration);
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

        lock.values().cloned().collect()
    }
}
