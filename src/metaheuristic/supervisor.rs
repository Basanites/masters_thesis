use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};
use std::time::Duration;

pub trait Supervisor<MessageType: Message> {}

pub trait Message {
    fn get_info(&self) -> MessageInfo;
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MessageInfo {
    pub evaluations: usize,
    pub cpu_time: Duration,
}

impl MessageInfo {
    pub fn new(evaluations: usize, cpu_time: Duration) -> Self {
        Self {
            evaluations,
            cpu_time,
        }
    }
}

impl Add for MessageInfo {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            evaluations: self.evaluations + other.evaluations,
            cpu_time: self.cpu_time + other.cpu_time,
        }
    }
}

impl AddAssign for MessageInfo {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            evaluations: self.evaluations + other.evaluations,
            cpu_time: self.cpu_time + other.cpu_time,
        };
    }
}
