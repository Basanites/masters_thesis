use decorum::R64;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign};
use std::time::Duration;

pub trait Supervisor<MessageType: Message> {}

pub trait Message {
    type EwType;
    type NwType;
    fn get_info(&self) -> MessageInfo<Self::NwType, Self::EwType>;
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MessageInfo<Nw, Ew> {
    pub evaluations: usize,
    pub cpu_time: Duration,
    pub n_improvements: usize,
    pub changes: usize,
    pub phase: usize,
    pub distance: Ew,
    pub heuristic_score: R64,
    pub visited_nodes: usize,
    pub visited_nodes_with_val: usize,
    pub collected_val: Nw,
}

impl<Nw, Ew> MessageInfo<Nw, Ew> {
    pub fn new(
        evaluations: usize,
        n_improvements: usize,
        changes: usize,
        phase: usize,
        cpu_time: Duration,
        distance: Ew,
        heuristic_score: R64,
        visited_nodes: usize,
        visited_nodes_with_val: usize,
        collected_val: Nw,
    ) -> Self {
        Self {
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
}

impl<Nw: Add<Output = Nw>, Ew: Add<Output = Ew>> Add for MessageInfo<Nw, Ew> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            evaluations: self.evaluations + other.evaluations,
            n_improvements: self.n_improvements + other.n_improvements,
            changes: self.changes + other.changes,
            phase: other.phase,
            cpu_time: self.cpu_time + other.cpu_time,
            distance: self.distance + other.distance,
            heuristic_score: self.heuristic_score + other.heuristic_score,
            visited_nodes: self.visited_nodes + other.visited_nodes,
            visited_nodes_with_val: self.visited_nodes_with_val + other.visited_nodes_with_val,
            collected_val: self.collected_val + other.collected_val,
        }
    }
}

impl<Nw: Copy + Add<Output = Nw>, Ew: Copy + Add<Output = Ew>> AddAssign for MessageInfo<Nw, Ew> {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            evaluations: self.evaluations + other.evaluations,
            n_improvements: self.n_improvements + other.n_improvements,
            changes: self.changes + other.changes,
            phase: other.phase,
            cpu_time: self.cpu_time + other.cpu_time,
            distance: self.distance + other.distance,
            heuristic_score: self.heuristic_score + other.heuristic_score,
            visited_nodes: self.visited_nodes + other.visited_nodes,
            visited_nodes_with_val: self.visited_nodes_with_val + other.visited_nodes_with_val,
            collected_val: self.collected_val + other.collected_val,
        };
    }
}
