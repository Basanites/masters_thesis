use crate::metaheuristic::aco;
use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::{Message, MessageInfo};

use csv::Writer;
use std::collections::HashMap;
use std::io::{stderr, Stderr, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub struct Supervisor<W: Write> {
    pub sender: Sender<aco::Message>,
    receiver: Receiver<aco::Message>,
    ants: usize,
    messages: HashMap<usize, Vec<MessageInfo>>,
    counters: HashMap<usize, usize>,
    aggregation_rate: usize,
    writer: Writer<W>,
}

impl<W: Write> Supervisor<W> {
    pub fn new(aggregation_rate: usize, writer: Writer<W>) -> Self {
        let (tx, rx) = mpsc::channel();
        Supervisor {
            sender: tx,
            receiver: rx,
            ants: 0,
            messages: HashMap::default(),
            counters: HashMap::default(),
            aggregation_rate,
            writer,
        }
    }

    pub fn new_ant(&mut self) -> (Sender<aco::Message>, usize) {
        self.ants += 1;
        let id = self.ants;

        (self.sender.clone(), id)
    }

    pub fn aggregate_receive(&mut self) {
        while let Ok(message) = self.receiver.recv() {
            let ant_id = message.id();
            let mut i = 1;
            if let Some(count) = self.counters.get_mut(&ant_id) {
                *count += 1;
                i = *count;
            } else {
                self.counters.insert(ant_id, i);
            }

            let idx = i % self.aggregation_rate;
            if let Some(messages) = self.messages.get_mut(&ant_id) {
                if idx >= messages.len() {
                    messages.resize_with(idx + 1, Default::default);
                }
                messages[idx] += message.get_info();
            }
        }
    }

    pub fn reset(&mut self) {
        self.ants = 0;
        self.messages = HashMap::default();
        self.counters = HashMap::default();
        let (tx, rx) = mpsc::channel();
        self.sender = tx;
        self.receiver = rx;
    }
}

impl<W: Write> supervisor::Supervisor<aco::Message> for Supervisor<W> {}

impl Default for Supervisor<Stderr> {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Supervisor {
            sender: tx,
            receiver: rx,
            ants: 0,
            messages: HashMap::default(),
            counters: HashMap::default(),
            aggregation_rate: 1,
            writer: Writer::from_writer(stderr()),
        }
    }
}
