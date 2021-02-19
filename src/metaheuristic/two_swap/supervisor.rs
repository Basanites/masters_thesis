use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::{Message, MessageInfo};
use crate::metaheuristic::two_swap;

use csv::Writer;
use serde::Serialize;
use std::io::{stderr, Stderr, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Supervisor<W: Write> {
    sender: Sender<two_swap::Message>,
    receiver: Receiver<two_swap::Message>,
    messages: Vec<MessageInfo>,
    writer: Writer<W>,
    aggregation_rate: usize,
}

impl<W: Write> Supervisor<W> {
    pub fn new(aggregation_rate: usize, writer: Writer<W>) -> Self {
        let (tx, rx) = mpsc::channel();
        Supervisor {
            sender: tx,
            receiver: rx,
            messages: Vec::default(),
            writer,
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

        for i in 0..self.messages.len() {
            let msg_info = self.messages.get(i).unwrap();
            let record = two_swap::Message::new(
                i * self.aggregation_rate,
                msg_info.evaluations,
                0,
                0,
                0,
                msg_info.cpu_time,
            );
            let res = self.writer.serialize(record);
            if let Err(err) = res {
                eprintln!("{:?}", err);
            }
        }
    }
}

impl<W: Write> supervisor::Supervisor<two_swap::Message> for Supervisor<W> {}

impl Default for Supervisor<Stderr> {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Supervisor {
            sender: tx,
            receiver: rx,
            messages: Vec::default(),
            writer: Writer::from_writer(stderr()),
            aggregation_rate: 1,
        }
    }
}
