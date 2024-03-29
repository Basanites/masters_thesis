use crate::geo::GeoPoint;
use crate::util::scale::Scaler;

pub struct GeoPointScaler {
    pub lat_scaler: Scaler<i32>,
    pub lon_scaler: Scaler<i32>,
}

impl GeoPointScaler {
    pub fn new(min_lat: i32, min_lon: i32, max_lat: i32, max_lon: i32) -> Self {
        GeoPointScaler {
            lat_scaler: Scaler::new(min_lat, max_lat),
            lon_scaler: Scaler::new(min_lon, max_lon),
        }
    }

    pub fn from_point_iterator(points: impl Iterator<Item = GeoPoint>) -> Self {
        let extremes = points.fold((0, 0, i32::MAX, i32::MAX), |acc, point| {
            (
                i32::max(acc.0, point.micro_lat()),
                i32::max(acc.1, point.micro_lon()),
                i32::min(acc.2, point.micro_lat()),
                i32::min(acc.3, point.micro_lon()),
            )
        });

        GeoPointScaler::new(extremes.0, extremes.1, extremes.2, extremes.3)
    }

    pub fn scale_point(&self, point: &GeoPoint) -> GeoPoint {
        GeoPoint::from_micro_degrees(
            self.lat_scaler.scale(point.micro_lat()),
            self.lon_scaler.scale(point.micro_lon()),
        )
    }
}
