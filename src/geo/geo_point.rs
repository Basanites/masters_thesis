use std::f64::consts::PI;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub struct GeoPoint {
    micro_lat: u32,
    micro_lon: u32,
    lat_rad: f64,
    lon_rad: f64,
}

impl GeoPoint {
    pub fn from_micro_degrees(micro_lat: u32, micro_lon: u32) -> Self {
        GeoPoint {
            micro_lat,
            micro_lon,
            lat_rad: degrees_to_radians(from_micro_scale(micro_lat)),
            lon_rad: degrees_to_radians(from_micro_scale(micro_lon)),
        }
    }

    pub fn from_degrees(lat: f64, lon: f64) -> Self {
        GeoPoint {
            micro_lat: to_micro_scale(lat),
            micro_lon: to_micro_scale(lon),
            lat_rad: degrees_to_radians(lat),
            lon_rad: degrees_to_radians(lon),
        }
    }

    pub fn from_radians(lat_rad: f64, lon_rad: f64) -> Self {
        GeoPoint {
            micro_lat: to_micro_scale(degrees_to_radians(lat_rad)),
            micro_lon: to_micro_scale(degrees_to_radians(lon_rad)),
            lat_rad,
            lon_rad,
        }
    }

    pub fn lat(&self) -> f64 {
        from_micro_scale(self.micro_lat)
    }

    pub fn lon(&self) -> f64 {
        from_micro_scale(self.micro_lon)
    }

    pub fn lat_rad(&self) -> f64 {
        self.lat_rad
    }

    pub fn lon_rad(&self) -> f64 {
        self.lon_rad
    }

    pub fn micro_lat(&self) -> u32 {
        self.micro_lat
    }

    pub fn micro_lon(&self) -> u32 {
        self.micro_lon
    }
}

impl Hash for GeoPoint {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (self.micro_lat, self.micro_lon).hash(hasher)
    }
}

impl PartialEq for GeoPoint {
    fn eq(&self, other: &Self) -> bool {
        self.micro_lat() == other.micro_lat() && self.micro_lon() == other.micro_lon()
    }
}

impl Eq for GeoPoint {}

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

fn radians_to_degrees(radians: f64) -> f64 {
    radians * 180.0 / PI
}

fn to_micro_scale(val: f64) -> u32 {
    (val * 1000000.0) as u32
}

fn from_micro_scale(val: u32) -> f64 {
    (val as f64) / 1000000.0
}
