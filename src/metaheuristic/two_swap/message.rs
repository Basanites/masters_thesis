use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::time::Duration;

#[derive(Debug)]
pub struct Message<Ew> {
    pub iteration: usize,
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

    pub fn from_info(iteration: usize, info: MessageInfo<Ew>) -> Self {
        Self {
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

impl<Ew: Serialize> Serialize for Message<Ew> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        let mut state = serializer.serialize_struct("Message", 3)?;
        state.serialize_field("iteration", &self.iteration)?;
        state.serialize_field("evaluations", &self.evaluations)?;
        state.serialize_field("n_improvements", &self.n_improvements)?;
        state.serialize_field("changes", &self.changes)?;
        state.serialize_field("phase", &self.phase)?;
        state.serialize_field("cpu_time_mus", &self.cpu_time.as_micros())?;
        state.serialize_field("time", &self.distance)?;
        state.serialize_field("value", &self.score)?;
        state.end()
    }
}
