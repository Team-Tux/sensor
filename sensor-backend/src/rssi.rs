use sensor_lib::Environment;

use crate::sensors::SensorCandidate;

const CALIBRATED_RSSI_AT_1M: f64 = -40.0;

fn rssi_to_distance(rssi: f64, environment: Environment) -> f64 {
    let exponent = (CALIBRATED_RSSI_AT_1M - rssi) / (10.0 * environment.as_f64());

    10.0f64.powf(exponent)
}

pub async fn trilaterate(
    s1: &SensorCandidate,
    s2: &SensorCandidate,
    s3: &SensorCandidate,
) -> (f64, f64) {
    let d1 = rssi_to_distance(s1.rssi as f64, s1.environment);
    let d2 = rssi_to_distance(s2.rssi as f64, s2.environment);
    let d3 = rssi_to_distance(s3.rssi as f64, s3.environment);

    let a = 2.0 * (s2.longitude - s1.longitude);
    let b = 2.0 * (s2.latitude - s1.latitude);
    let c = d1.powi(2) - d2.powi(2) + s2.longitude.powi(2) - s1.longitude.powi(2)
        + s2.latitude.powi(2)
        - s1.latitude.powi(2);

    let d = 2.0 * (s3.longitude - s2.longitude);
    let e = 2.0 * (s3.latitude - s2.latitude);
    let f = d2.powi(2) - d3.powi(2) + s3.longitude.powi(2) - s2.longitude.powi(2)
        + s3.latitude.powi(2)
        - s2.latitude.powi(2);

    let denom = (a * e) - (b * d);

    let longitude = (c * e - b * f) / denom;
    let latitude = (a * f - c * d) / denom;

    (latitude, longitude)
}
