use nalgebra::{Matrix3, RowVector3, Vector3};

use crate::sensors::Sensor;

pub fn transform_local_to_global(
    local_x: f64,
    local_y: f64,
    s1: &Sensor,
    s2: &Sensor,
    s3: &Sensor,
) -> Option<(f64, f64)> {
    let mean_lat = (s1.lat + s2.lat + s3.lat) / 3.0;

    let meters_per_deg_lat = 111_320.0;
    let meters_per_deg_lon = 111_320.0 * mean_lat.to_radians().cos();

    let global_x1 = (s1.lon - s1.lon) * meters_per_deg_lon;
    let global_y1 = (s1.lat - s1.lat) * meters_per_deg_lat;

    let global_x2 = (s2.lon - s1.lon) * meters_per_deg_lon;
    let global_y2 = (s2.lat - s1.lat) * meters_per_deg_lat;

    let global_x3 = (s3.lon - s1.lon) * meters_per_deg_lon;
    let global_y3 = (s3.lat - s1.lat) * meters_per_deg_lat;

    let mat_local = Matrix3::from_rows(&[
        RowVector3::new(s1.x, s1.y, 1.0),
        RowVector3::new(s2.x, s2.y, 1.0),
        RowVector3::new(s3.x, s3.y, 1.0),
    ]);

    let vec_global_x = Vector3::new(global_x1, global_x2, global_x3);
    let vec_global_y = Vector3::new(global_y1, global_y2, global_y3);

    let inv_local = mat_local.try_inverse()?;

    let coeff_x = inv_local * vec_global_x;
    let coeff_y = inv_local * vec_global_y;

    let gx = coeff_x[0] * local_x + coeff_x[1] * local_y + coeff_x[2];
    let gy = coeff_y[0] * local_x + coeff_y[1] * local_y + coeff_y[2];

    let lon = s1.lon + gx / meters_per_deg_lon;
    let lat = s1.lat + gy / meters_per_deg_lat;

    Some((lat, lon))
}
