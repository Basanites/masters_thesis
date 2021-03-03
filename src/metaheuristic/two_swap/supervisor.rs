use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::{Message, MessageInfo};
use crate::metaheuristic::two_swap;

use csv::Writer;
use serde::Serialize;
use std::default::Default;
use std::io::{stderr, Stderr, Write};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Supervisor<W: Write, Ew: Serialize + Sized> {
    sender: Sender<two_swap::Message<Ew>>,
    receiver: Receiver<two_swap::Message<Ew>>,
    messages: Vec<MessageInfo<Ew>>,
    writer: Writer<W>,
    aggregation_rate: usize,
}

impl<W, Ew> Supervisor<W, Ew>
where
    W: Write,
    Ew: Serialize + Default + Add<Output = Ew> + Copy,
{
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

    pub fn sender(&self) -> Sender<two_swap::Message<Ew>> {
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
                msg_info.n_improvements,
                msg_info.changes,
                msg_info.phase,
                msg_info.cpu_time,
                msg_info.distance,
                msg_info.score,
            );
            let res = self.writer.serialize(record);
            if let Err(err) = res {
                eprintln!("{:?}", err);
            }
        }
    }
}

impl<W, Ew: Copy> supervisor::Supervisor<two_swap::Message<Ew>> for Supervisor<W, Ew>
where
    W: Write,
    Ew: Serialize + Default + Add<Output = Ew>,
{
}

impl<Ew> Default for Supervisor<Stderr, Ew>
where
    Ew: Serialize + Default + Add<Output = Ew>,
{
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
