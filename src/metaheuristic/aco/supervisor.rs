use crate::metaheuristic::aco;
use crate::metaheuristic::supervisor;
use crate::metaheuristic::supervisor::{Message, MessageInfo};

use csv::Writer;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{stderr, Stderr, Write};
use std::ops::Add;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct Supervisor<W: Write, Nw: Serialize + Sized, Ew: Serialize + Sized> {
    pub sender: Sender<aco::Message<Nw, Ew>>,
    receiver: Receiver<aco::Message<Nw, Ew>>,
    ants: usize,
    messages: HashMap<usize, Vec<MessageInfo<Nw, Ew>>>,
    counters: HashMap<usize, usize>,
    aggregation_rate: usize,
    writer: Writer<W>,
}

impl<W, Nw, Ew> Supervisor<W, Nw, Ew>
where
    W: Write,
    Nw: Serialize + Default + Debug + Add<Output = Nw> + Copy,
    Ew: Serialize + Default + Debug + Add<Output = Ew> + Copy,
{
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

    pub fn new_ant(&mut self) -> (Sender<aco::Message<Nw, Ew>>, usize) {
        self.ants += 1;
        let id = self.ants;

        (self.sender.clone(), id)
    }

    pub fn aggregate_receive(&mut self) {
        while let Ok(message) = self.receiver.recv_timeout(Duration::from_millis(1)) {
            let ant_id = message.id();
            let mut i = 0;
            if let Some(count) = self.counters.get_mut(&ant_id) {
                *count += 1;
                i = *count;
            } else {
                self.counters.insert(ant_id, i);
            }

            let idx = i / self.aggregation_rate;
            if let Some(messages) = self.messages.get_mut(&ant_id) {
                if idx >= messages.len() {
                    messages.resize_with(idx + 1, Default::default);
                }
                messages[idx] += message.get_info();
            } else {
                self.messages.insert(ant_id, vec![message.get_info()]);
            }
        }

        for i in 0..self.messages.get(&0).unwrap().len() {
            let best_msg = self.messages.get(&0).unwrap().get(i).unwrap();
            let mut evals = 0;
            let mut cpu_time = Duration::from_micros(0);
            for ant_id in 1..self.messages.len() {
                let messages = self.messages.get(&ant_id).unwrap();
                let msg_info = messages.get(i).unwrap();
                evals += msg_info.evaluations;
                cpu_time += msg_info.cpu_time;
            }

            let record = aco::Message::new(
                0,
                i * self.aggregation_rate,
                evals,
                best_msg.n_improvements,
                best_msg.changes,
                best_msg.phase,
                cpu_time,
                best_msg.distance,
                best_msg.heuristic_score,
                best_msg.visited_nodes,
                best_msg.visited_nodes_with_val,
                best_msg.collected_val,
            );

            let res = self.writer.serialize(&record);
            if let Err(err) = res {
                eprintln!("{:?}", err);
            }
        }

        self.prepare_next();
    }

    pub fn prepare_next(&mut self) {
        self.ants = 0;
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

impl<W, Nw: Copy, Ew: Copy> supervisor::Supervisor<aco::Message<Nw, Ew>> for Supervisor<W, Nw, Ew>
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
            ants: 0,
            messages: HashMap::default(),
            counters: HashMap::default(),
            aggregation_rate: 1,
            writer: Writer::from_writer(stderr()),
        }
    }
}
