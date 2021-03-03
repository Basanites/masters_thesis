use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use std::time::Duration;

#[derive(Debug)]
pub struct Message<Ew> {
    ant_id: usize,
    iteration: usize,
    evaluations: usize,
    n_improvements: usize,
    changes: usize,
    phase: usize,
    cpu_time: Duration,
    distance: Ew,
    score: f64,
}

impl<Ew> Message<Ew> {
    pub fn new(
        ant_id: usize,
        iteration: usize,
        evaluations: usize,
        n_improvements: usize,
        changes: usize,
        phase: usize,
        cpu_time: Duration,
        distance: Ew,
        score: f64,
    ) -> Self {
        Self {
            ant_id,
            iteration,
            evaluations,
            n_improvements,
            changes,
            phase,
            cpu_time,
            distance,
            score,
        }
    }

    pub fn from_info(ant_id: usize, iteration: usize, info: MessageInfo<Ew>) -> Self {
        Self {
            ant_id,
            iteration,
            evaluations: info.evaluations,
            n_improvements: info.n_improvements,
            changes: info.changes,
            phase: info.phase,
            cpu_time: info.cpu_time,
            distance: info.distance,
            score: info.score,
        }
    }

    pub fn id(&self) -> usize {
        self.ant_id
    }
}

impl<Ew: Copy> supervisor::Message for Message<Ew> {
    type EwType = Ew;

    fn get_info(&self) -> MessageInfo<Ew> {
        MessageInfo::new(
            self.evaluations,
            self.n_improvements,
            self.changes,
            self.phase,
            self.cpu_time,
            self.distance,
            self.score,
        )
    }
}
