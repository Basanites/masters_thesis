use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::MessageInfo;

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::time::Duration;

#[derive(Debug)]
pub struct Message {
    pub iteration: usize,
    evaluations: usize,
    n_improvements: usize,
    changes: usize,
    phase: usize,
    cpu_time: Duration,
}

impl Message {
    pub fn new(
        iteration: usize,
        evaluations: usize,
        n_improvements: usize,
        changes: usize,
        phase: usize,
        cpu_time: Duration,
    ) -> Self {
        Self {
            iteration,
            evaluations,
            n_improvements,
            changes,
            phase,
            cpu_time,
        }
    }
}

impl supervisor::Message for Message {
    fn get_info(&self) -> MessageInfo {
        MessageInfo::new(self.evaluations, self.cpu_time)
    }
}

impl Serialize for Message {
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
        state.end()
    }
}
