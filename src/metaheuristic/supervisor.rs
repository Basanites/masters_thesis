use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};
use std::time::Duration;

pub trait Supervisor<MessageType: Message> {}

pub trait Message {
    type EwType;
    fn get_info(&self) -> MessageInfo<Self::EwType>;
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MessageInfo<Ew> {
    pub evaluations: usize,
    pub cpu_time: Duration,
    pub n_improvements: usize,
    pub changes: usize,
    pub phase: usize,
    pub distance: Ew,
    pub score: f64,
}

impl<Ew> MessageInfo<Ew> {
    pub fn new(
        evaluations: usize,
        n_improvements: usize,
        changes: usize,
        phase: usize,
        cpu_time: Duration,
        distance: Ew,
        score: f64,
    ) -> Self {
        Self {
            evaluations,
            n_improvements,
            changes,
            phase,
            cpu_time,
            distance,
            score,
        }
    }
}

impl<Ew: Add<Output = Ew>> Add for MessageInfo<Ew> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            evaluations: self.evaluations + other.evaluations,
            n_improvements: self.n_improvements + other.n_improvements,
            changes: self.changes + other.changes,
            phase: other.phase,
            cpu_time: self.cpu_time + other.cpu_time,
            distance: self.distance + other.distance,
            score: self.score + other.score,
        }
    }
}

impl<Ew: Add<Output = Ew> + Copy> AddAssign for MessageInfo<Ew> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            evaluations: self.evaluations + other.evaluations,
            n_improvements: self.n_improvements + other.n_improvements,
            changes: self.changes + other.changes,
            phase: other.phase,
            cpu_time: self.cpu_time + other.cpu_time,
            distance: self.distance + other.distance,
            score: self.score + other.score,
        };
    }
}
