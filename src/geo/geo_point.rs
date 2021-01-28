use serde::Serialize;
use std::cmp::Ordering;
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Serialize)]
pub struct GeoPoint {
    micro_lat: i64,
    micro_lon: i64,
}

impl GeoPoint {
    pub fn from_micro_degrees(micro_lat: i64, micro_lon: i64) -> Self {
        GeoPoint {
            micro_lat,
            micro_lon,
        }
    }

    pub fn from_degrees(lat: f64, lon: f64) -> Self {
        GeoPoint {
            micro_lat: to_micro_scale(lat),
            micro_lon: to_micro_scale(lon),
        }
    }

    pub fn from_radians(lat_rad: f64, lon_rad: f64) -> Self {
        GeoPoint {
            micro_lat: to_micro_scale(degrees_to_radians(lat_rad)),
            micro_lon: to_micro_scale(degrees_to_radians(lon_rad)),
        }
    }

    pub fn lat(&self) -> f64 {
        from_micro_scale(self.micro_lat)
    }

    pub fn lon(&self) -> f64 {
        from_micro_scale(self.micro_lon)
    }

    pub fn lat_rad(&self) -> f64 {
        degrees_to_radians(from_micro_scale(self.micro_lat()))
    }

    pub fn lon_rad(&self) -> f64 {
        degrees_to_radians(from_micro_scale(self.micro_lon()))
    }

    pub fn micro_lat(&self) -> i64 {
        self.micro_lat
    }

    pub fn micro_lon(&self) -> i64 {
        self.micro_lon
    }
}

impl Hash for GeoPoint {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (self.micro_lat(), self.micro_lon()).hash(hasher)
    }
}

impl PartialEq for GeoPoint {
    fn eq(&self, other: &Self) -> bool {
        self.micro_lat() == other.micro_lat() && self.micro_lon() == other.micro_lon()
    }
}

impl PartialOrd for GeoPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GeoPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.micro_lat
            .cmp(&other.micro_lat)
            .then(self.micro_lon.cmp(&other.micro_lon))
    }
}

impl Eq for GeoPoint {}

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

#[allow(dead_code)]
fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / PI
}

fn to_micro_scale(val: f64) -> i64 {
    (val * 1000000.0) as i64
}

fn from_micro_scale(val: i64) -> f64 {
    (val as f64) / 1000000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn from_micro_degrees_works() {
        let point = GeoPoint::from_micro_degrees(12345000, 54321000);

        assert_eq!(point.micro_lat, 12345000);
        assert_eq!(point.micro_lon, 54321000);
    }

    #[test]
    fn from_degrees_works() {
        let point = GeoPoint::from_degrees(12.345, 54.321);

        assert_eq!(point.micro_lat, 12345000);
        assert_eq!(point.micro_lon, 54321000);
    }

    #[test]
    fn from_rads_works() {}

    #[test]
    fn lat_works() {
        let point = GeoPoint::from_micro_degrees(12345000, 54321000);

        assert!(approx_eq!(f64, point.lat(), 12.345));
    }

    #[test]
    fn lon_works() {
        let point = GeoPoint::from_micro_degrees(12345000, 54321000);

        assert!(approx_eq!(f64, point.lon(), 54.321));
    }

    #[test]
    fn micro_lat_works() {
        let point = GeoPoint::from_micro_degrees(12345000, 54321000);

        assert_eq!(point.micro_lat(), 12345000);
    }

    #[test]
    fn micro_lon_works() {
        let point = GeoPoint::from_micro_degrees(12345000, 54321000);

        assert_eq!(point.micro_lon(), 54321000);
    }

    #[test]
    fn from_micro_scale_works() {
        assert_eq!(from_micro_scale(12670000), 12.67);
    }

    #[test]
    fn to_micro_scale_works() {
        assert_eq!(to_micro_scale(12.67), 12670000)
    }
}
