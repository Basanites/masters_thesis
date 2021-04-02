use crate::metaheuristic::random_search;
use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::{Message, MessageInfo};

use csv::Writer;
use serde::Serialize;
use std::default::Default;
use std::io::{stderr, Stderr, Write};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Supervisor<W: Write, Nw: Serialize + Sized, Ew: Serialize + Sized> {
    sender: Sender<random_search::Message<Nw, Ew>>,
    receiver: Receiver<random_search::Message<Nw, Ew>>,
    messages: Vec<MessageInfo<Nw, Ew>>,
    writer: Writer<W>,
    aggregation_rate: usize,
}

impl<W, Nw, Ew> Supervisor<W, Nw, Ew>
where
    W: Write,
    Nw: Serialize + Default + Add<Output = Nw> + Copy,
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

    pub fn sender(&self) -> Sender<random_search::Message<Nw, Ew>> {
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
            let record = random_search::Message::new(
                i * self.aggregation_rate,
                msg_info.evaluations,
                msg_info.n_improvements,
                msg_info.changes,
                msg_info.phase,
                msg_info.cpu_time,
                msg_info.distance,
                msg_info.heuristic_score,
                msg_info.visited_nodes,
                msg_info.visited_nodes_with_val,
                msg_info.collected_val,
            );
            let res = self.writer.serialize(record);
            if let Err(err) = res {
                eprintln!("{:?}", err);
            }
        }
    }
}

impl<W, Nw: Copy, Ew: Copy> supervisor::Supervisor<random_search::Message<Nw, Ew>>
    for Supervisor<W, Nw, Ew>
where
    W: Write,
    Nw: Serialize + Default + Add<Output = Nw>,
    Ew: Serialize + Default + Add<Output = Ew>,
{
}

impl<Nw, Ew> Default for Supervisor<Stderr, Nw, Ew>
where
    Nw: Serialize + Default + Add<Output = Nw>,
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
