use super::GeoPoint;

/// Calculates the distance between two geopoints.
/// Done using formula from https://en.wikipedia.org/wiki/Haversine_formula.
/// This is only problematic when the points are antipodal to one another.
pub fn geodistance_haversine(point_a: GeoPoint, point_b: GeoPoint) -> f64 {
    // average earth radius is assumed to be 6371km
    2.0 * 6371.0
        * (((point_b.lat_rad() - point_a.lat_rad()) / 2.0)
            .sin()
            .powi(2)
            + point_a.lat_rad().cos()
                * point_b.lat_rad().cos()
                * ((point_b.lon_rad() - point_a.lon_rad()) / 2.0)
                    .sin()
                    .powi(2))
        .sqrt()
        .asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::GeoPoint;

    #[test]
    fn geodistance_haversine_works() {
        let p1 = GeoPoint::from_degrees(51.350205, 12.4973972);
        let p2 = GeoPoint::from_degrees(51.3308595, 12.3130661);
        let dist = geodistance_haversine(p1, p2);

        assert!(dist >= 12.983);
        assert!(dist <= 12.984);
    }
}
