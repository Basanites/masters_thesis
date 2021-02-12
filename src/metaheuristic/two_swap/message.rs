use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use std::time::Duration;

#[derive(Debug)]
pub struct Message {
    pub iteration: usize,
    evaluations: usize,
    cpu_time: Duration,
}

impl Message {
    pub fn new(iteration: usize, evaluations: usize, cpu_time: Duration) -> Self {
        Self {
            iteration,
            evaluations,
            cpu_time,
        }
    }
}

impl supervisor::Message for Message {
    fn get_info(&self) -> MessageInfo {
        MessageInfo::new(self.evaluations, self.cpu_time)
    }
}
