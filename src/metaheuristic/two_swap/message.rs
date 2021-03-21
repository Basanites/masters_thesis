use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::time::Duration;

#[derive(Debug)]
pub struct Message<Nw, Ew> {
    pub iteration: usize,
    pub evaluations: usize,
    pub n_improvements: usize,
    pub changes: usize,
    pub phase: usize,
    pub cpu_time: Duration,
    pub distance: Ew,
    pub heuristic_score: f64,
    pub visited_nodes: usize,
    pub visited_nodes_with_val: usize,
    pub collected_val: Nw,
}

impl<Nw, Ew> Message<Nw, Ew> {
    pub fn new(
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

    pub fn from_info(iteration: usize, info: MessageInfo<Nw, Ew>) -> Self {
        Self {
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

impl<Nw: Serialize, Ew: Serialize> Serialize for Message<Nw, Ew> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 11 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Message", 11)?;
        state.serialize_field("iteration", &self.iteration)?;
        state.serialize_field("evaluations", &self.evaluations)?;
        state.serialize_field("n_improvements", &self.n_improvements)?;
        state.serialize_field("changes", &self.changes)?;
        state.serialize_field("phase", &self.phase)?;
        state.serialize_field("cpu_time_mus", &self.cpu_time.as_micros())?;
        state.serialize_field("distance", &self.distance)?;
        state.serialize_field("heuristic_score", &self.heuristic_score)?;
        state.serialize_field("visited_nodes", &self.visited_nodes)?;
        state.serialize_field("visited_nodes_with_val", &self.visited_nodes_with_val)?;
        state.serialize_field("collected_val", &self.collected_val)?;
        state.end()
    }
}
