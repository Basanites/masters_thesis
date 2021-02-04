use getrandom::getrandom;
use oorandom::Rand64;

pub fn preseeded_rng64() -> Rand64 {
    Rand64::new(os_random_seed())
}

pub fn rng64(seed: u128) -> Rand64 {
    Rand64::new(seed)
}

pub fn os_random_seed() -> u128 {
    let mut buf = [0; 16];
    let _res = getrandom(&mut buf);
    u128::from_le_bytes(buf)
}
