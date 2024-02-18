use decorum::{R32, R64};

macro_rules! min_max {
    ($t:ty) => {
        impl Min for $t {
            fn min() -> Self {
                <$t>::MIN
            }
        }

        impl Max for $t {
            fn max() -> Self {
                <$t>::MAX
            }
        }
    };
}

macro_rules! min_max_decorum {
    ($t:ty, $other_t:ty) => {
        impl Min for $t {
            fn min() -> Self {
                Self::from_inner(<$other_t>::MIN)
            }
        }

        impl Max for $t {
            fn max() -> Self {
                Self::from_inner(<$other_t>::MAX)
            }
        }
    };
}

pub trait Min {
    fn min() -> Self;
}

pub trait Max {
    fn max() -> Self;
}

min_max! {f64}
min_max! {f32}
min_max! {usize}
min_max! {i64}
min_max! {i32}
min_max_decorum! {R32, f32}
min_max_decorum! {R64, f64}
