use std::f64::consts::PI;

pub struct GeoPoint {
    lat: f64,
    lon: f64,
    lat_rad: f64,
    lon_rad: f64
}

impl GeoPoint {
    pub fn from_degrees(lat: f64, lon: f64) -> Self {
        GeoPoint {
            lat,
            lon,
            lat_rad: degrees_to_radians(lat),
            lon_rad: degrees_to_radians(lon)
        }
    }

    pub fn from_radians(lat: f64, lon: f64) -> Self {
        GeoPoint {
            lat: degrees_to_radians(lat),
            lon: degrees_to_radians(lon),
            lat_rad: lat,
            lon_rad: lon
        }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn lat_rad(&self) -> f64 {
        self.lat_rad
    }

    pub fn lon_rad(&self) -> f64 {
        self.lon_rad
    }
}

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / PI
}
