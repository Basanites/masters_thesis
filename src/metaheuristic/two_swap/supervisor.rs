use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::{Message, MessageInfo};
use crate::metaheuristic::two_swap;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Supervisor {
    sender: Sender<two_swap::Message>,
    receiver: Receiver<two_swap::Message>,
    messages: Vec<MessageInfo>,
    aggregation_rate: usize,
}

impl Supervisor {
    pub fn new(aggregation_rate: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        Supervisor {
            sender: tx,
            receiver: rx,
            messages: Vec::default(),
            aggregation_rate,
        }
    }

    pub fn sender(&self) -> Sender<two_swap::Message> {
        self.sender.clone()
    }

    pub fn aggregate_receive(&mut self) {
        while let Ok(message) = self.receiver.recv_timeout(Duration::from_millis(1)) {
            let idx = message.iteration / self.aggregation_rate;
            if idx >= self.messages.len() {
                self.messages.resize_with(idx + 1, Default::default);
            }
            self.messages[idx] += message.get_info();
        }

        eprintln!("\n{:?}\n", self.messages);
    }
}

impl supervisor::Supervisor<two_swap::Message> for Supervisor {}

impl Default for Supervisor {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Supervisor {
            sender: tx,
            receiver: rx,
            messages: Vec::default(),
            aggregation_rate: 1,
        }
    }
}
