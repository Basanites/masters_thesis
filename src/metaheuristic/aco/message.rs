use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use std::time::Duration;

#[derive(Debug)]
pub struct Message<Nw, Ew> {
    pub ant_id: usize,
    pub iteration: usize,
    pub evaluations: usize,
    pub cpu_time: Duration,
    pub n_improvements: usize,
    pub changes: usize,
    pub phase: usize,
    pub distance: Ew,
    pub heuristic_score: f64,
    pub visited_nodes: usize,
    pub visited_nodes_with_val: usize,
    pub collected_val: Nw,
}

impl<Nw, Ew> Message<Nw, Ew> {
    pub fn new(
        ant_id: usize,
        iteration: usize,
        evaluations: usize,
        n_improvements: usize,
        changes: usize,
        phase: usize,
        cpu_time: Duration,
        distance: Ew,
        heuristic_score: f64,
        visited_nodes: usize,
        visited_nodes_with_val: usize,
        collected_val: Nw,
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
            heuristic_score,
            visited_nodes,
            visited_nodes_with_val,
            collected_val,
        }
    }

    pub fn from_info(ant_id: usize, iteration: usize, info: MessageInfo<Nw, Ew>) -> Self {
        Self {
            ant_id,
            iteration,
            evaluations: info.evaluations,
            n_improvements: info.n_improvements,
            changes: info.changes,
            phase: info.phase,
            cpu_time: info.cpu_time,
            distance: info.distance,
            heuristic_score: info.heuristic_score,
            visited_nodes: info.visited_nodes,
            visited_nodes_with_val: info.visited_nodes_with_val,
            collected_val: info.collected_val,
        }
    }

    pub fn id(&self) -> usize {
        self.ant_id
    }
}

impl<Nw: Copy, Ew: Copy> supervisor::Message for Message<Nw, Ew> {
    type EwType = Ew;
    type NwType = Nw;

    fn get_info(&self) -> MessageInfo<Nw, Ew> {
        MessageInfo::new(
            self.evaluations,
            self.n_improvements,
            self.changes,
            self.phase,
            self.cpu_time,
            self.distance,
            self.heuristic_score,
            self.visited_nodes,
            self.visited_nodes_with_val,
            self.collected_val,
        )
    }
}
