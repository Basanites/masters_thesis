use decorum::{R32, R64};
use std::ops::{Add, Mul};

pub trait SmallVal {
    fn small() -> Self;
}

impl SmallVal for f64 {
    fn small() -> Self {
        1.0
    }
}

impl SmallVal for f32 {
    fn small() -> Self {
        1.0
    }
}

impl SmallVal for R32 {
    fn small() -> Self {
        R32::from_inner(1.0)
    }
}

impl SmallVal for R64 {
    fn small() -> Self {
        R64::from_inner(1.0)
    }
}

impl SmallVal for usize {
    fn small() -> Self {
        1
    }
}

impl SmallVal for u32 {
    fn small() -> Self {
        1
    }
}

impl SmallVal for u64 {
    fn small() -> Self {
        1
    }
}

impl SmallVal for i32 {
    fn small() -> Self {
        1
    }
}

impl SmallVal for i64 {
    fn small() -> Self {
        1
    }
}

pub trait One: Sized + Mul<Self, Output = Self> {
    fn one() -> Self;
}

impl<T: num_traits::One> One for T {
    fn one() -> Self {
        <T as num_traits::One>::one()
    }
}

pub trait Zero: Sized + Add<Self, Output = Self> {
    fn zero() -> Self;
}

impl<T: num_traits::Zero> Zero for T {
    fn zero() -> Self {
        <T as num_traits::Zero>::zero()
    }
}
