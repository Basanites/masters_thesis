pub trait SmallVal<T> {
    fn small() -> T;
}

impl SmallVal<f64> for f64 {
    fn small() -> f64 {
        1.0
    }
}

impl SmallVal<usize> for usize {
    fn small() -> usize {
        1
    }
}
