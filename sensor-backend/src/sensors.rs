use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use sensor_lib::Environment;
use serde::Serialize;
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::coords::transform_local_to_global;
use crate::rssi::{calculate_rssi_median, trilaterate};

const MIN_MEASUREMENT_ENTRIES: usize = 10;
const MAX_MEASUREMENT_AGE: Duration = Duration::from_secs(60);

#[derive(Clone, Serialize)]
pub struct Sensor {
    pub id: u8,
    pub x: f64,
    pub y: f64,
    pub lat: f64,
    pub lon: f64,
    pub environment: Environment,
}

pub struct SensorCandidate {
    pub y: f64,
    pub x: f64,
    pub environment: Environment,
    pub rssi: i8,
}

#[derive(Clone, Serialize)]
pub struct Trilateration {
    pub fingerprint: u64,
    pub y: f64,
    pub x: f64,
    pub lat: f64,
    pub lon: f64,
}

type MeasurementsMap = HashMap<u64, HashMap<u8, VecDeque<(i8, Instant)>>>;

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
        y: f64,
        x: f64,
        latitude: f64,
        longitude: f64,
        environment: Environment,
    ) {
        let mut lock = self.sensors.write().await;

        let sensor = Sensor {
            id,
            y,
            x,
            lat: latitude,
            lon: longitude,
            environment,
        };

        lock.insert(id, sensor);
    }

    pub async fn get_sensors(&self) -> Vec<Sensor> {
        let lock = self.sensors.read().await;

        lock.values().cloned().collect()
    }

    pub async fn add_measurement(&self, fingerprint: u64, sensor_id: u8, rssi: i8) {
        let mut lock = self.measurements.write().await;

        let now = Instant::now();

        let sensors = lock.entry(fingerprint.clone()).or_default();

        let queue = sensors.entry(sensor_id).or_default();
        queue.push_back((rssi, now));

        if sensors.len() == 3 && sensors.values().all(|q| q.len() >= MIN_MEASUREMENT_ENTRIES) {
            let s_lock = self.sensors.read().await;

            let candidates: Option<Vec<SensorCandidate>> = sensors
                .iter()
                .map(|(id, queue)| {
                    let rssi = calculate_rssi_median(queue);

                    s_lock.get(id).map(|sensor| SensorCandidate {
                        y: sensor.y,
                        x: sensor.x,
                        environment: sensor.environment,
                        rssi,
                    })
                })
                .collect();

            drop(s_lock);

            if let Some(candidates) = candidates {
                let (y, x) = trilaterate(&candidates[0], &candidates[1], &candidates[2]).await;

                let s_lock = self.sensors.read().await;
                let sensors: Vec<&Sensor> = s_lock.values().collect();

                let (lat, lon) =
                    transform_local_to_global(x, y, sensors[0], sensors[1], sensors[2])
                        .unwrap_or((0.0, 0.0));

                drop(s_lock);

                let mut t_lock = self.trilaterations.write().await;

                let trilateration = Trilateration {
                    fingerprint,
                    y,
                    x,
                    lat,
                    lon,
                };

                t_lock.insert(fingerprint, trilateration);
                drop(t_lock);

                lock.remove(&fingerprint);
            }
        }

        lock.retain(|_, sensors| {
            if let Some(timestamp) = sensors
                .values()
                .filter_map(|queue| queue.back().map(|(_, timestamp)| *timestamp))
                .max()
            {
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
