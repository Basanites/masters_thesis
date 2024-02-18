use crate::geo::{geodistance_haversine, GeoPoint};
use decorum::R64;

pub trait Distance<T> {
    fn distance(p1: T, p2: T) -> R64;
}

impl Distance<GeoPoint> for GeoPoint {
    fn distance(p1: GeoPoint, p2: GeoPoint) -> R64 {
        R64::from_inner(geodistance_haversine(p1, p2))
    }
}

impl Distance<usize> for usize {
    fn distance(p1: usize, p2: usize) -> R64 {
        R64::from_inner(0.0)
    }
}
