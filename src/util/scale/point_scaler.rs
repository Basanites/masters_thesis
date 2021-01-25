use crate::util::{scale::Scaler, Point};

pub struct PointScaler {
    pub x_scaler: Scaler<f64>,
    pub y_scaler: Scaler<f64>,
}

impl PointScaler {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        PointScaler {
            x_scaler: Scaler::new(min_x, max_x),
            y_scaler: Scaler::new(min_y, max_y),
        }
    }

    pub fn from_point_iterator(points: impl Iterator<Item = Point>) -> Self {
        let extremes = points.fold((0., 0., f64::MAX, f64::MAX), |acc, point| {
            (
                f64::max(acc.0, point.x),
                f64::max(acc.1, point.y),
                f64::min(acc.2, point.x),
                f64::min(acc.3, point.y),
            )
        });

        PointScaler::new(extremes.0, extremes.1, extremes.2, extremes.3)
    }

    pub fn scale_point(&self, point: &Point) -> Point {
        Point {
            x: self.x_scaler.scale(point.x),
            y: self.y_scaler.scale(point.y),
        }
    }
}
