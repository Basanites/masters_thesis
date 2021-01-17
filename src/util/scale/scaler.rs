pub struct Scaler<T> {
    pub min: T,
    pub max: T,
    diff: T,
}

impl<T> Scaler<T>
    where T: Copy + std::ops::Sub<Output = T> + std::ops::Div<Output = T>
{
    pub fn new(min: T, max: T) -> Self {
        Scaler {
            min,
            max,
            diff: max - min
        }
    }

    pub fn scale(&self, val: T) -> T {
        (val - self.min) / self.diff
    }
}
