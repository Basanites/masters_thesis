use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use std::time::Duration;

#[derive(Debug)]
pub struct Message {
    ant_id: usize,
    evaluations: usize,
    cpu_time: Duration,
}

impl Message {
    pub fn new(ant_id: usize, evaluations: usize, cpu_time: Duration) -> Self {
        Self {
            ant_id,
            evaluations,
            cpu_time,
        }
    }

    pub fn id(&self) -> usize {
        self.ant_id
    }
}

impl supervisor::Message for Message {
    fn get_info(&self) -> MessageInfo {
        MessageInfo::new(self.evaluations, self.cpu_time)
    }
}
