use std::collections::HashMap;
use std::ops::{Add, AddAssign};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub trait Supervisor<MessageType: Message> {}
pub trait Message {
    fn get_info(&self) -> MessageInfo;
}

pub struct AcoMessage {
    ant_id: usize,
    evaluations: usize,
    cpu_time: Duration,
}

impl Message for AcoMessage {
    fn get_info(&self) -> MessageInfo {
        MessageInfo::new(self.evaluations, self.cpu_time)
    }
}

#[derive(Default, Debug)]
pub struct MessageInfo {
    evaluations: usize,
    cpu_time: Duration,
}

impl MessageInfo {
    pub fn new(evaluations: usize, cpu_time: Duration) -> Self {
        Self {
            evaluations,
            cpu_time,
        }
    }
}

impl Add for MessageInfo {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            evaluations: self.evaluations + other.evaluations,
            cpu_time: self.cpu_time + other.cpu_time,
        }
    }
}

impl AddAssign for MessageInfo {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            evaluations: self.evaluations + other.evaluations,
            cpu_time: self.cpu_time + other.cpu_time,
        };
    }
}

impl AcoMessage {
    pub fn new(ant_id: usize, evaluations: usize, cpu_time: Duration) -> Self {
        Self {
            ant_id,
            evaluations,
            cpu_time,
        }
    }

    pub fn id(&self) -> usize {
        self.ant_id
    }
}

pub struct AcoSupervisor<T: Send> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    ants: usize,
    messages: HashMap<usize, Vec<MessageInfo>>,
    counters: HashMap<usize, usize>,
    aggregation_rate: usize,
}

impl AcoSupervisor<AcoMessage> {
    pub fn new(aggregation_rate: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        AcoSupervisor {
            sender: tx,
            receiver: rx,
            ants: 0,
            messages: HashMap::default(),
            counters: HashMap::default(),
            aggregation_rate,
        }
    }

    pub fn new_ant(&mut self) -> (Sender<AcoMessage>, usize) {
        let id = self.ants;
        self.ants += 1;

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
    }
}

impl Supervisor<AcoMessage> for AcoSupervisor<AcoMessage> {}

impl Default for AcoSupervisor<AcoMessage> {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        AcoSupervisor {
            sender: tx,
            receiver: rx,
            ants: 0,
            messages: HashMap::default(),
            counters: HashMap::default(),
            aggregation_rate: 1,
        }
    }
}

#[derive(Debug)]
pub struct TwoSwapMessage {
    iteration: usize,
    evaluations: usize,
    cpu_time: Duration,
}

impl TwoSwapMessage {
    pub fn new(iteration: usize, evaluations: usize, cpu_time: Duration) -> Self {
        Self {
            iteration,
            evaluations,
            cpu_time,
        }
    }
}

impl Message for TwoSwapMessage {
    fn get_info(&self) -> MessageInfo {
        MessageInfo::new(self.evaluations, self.cpu_time)
    }
}

pub struct TwoSwapSupervisor<T: Send> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    messages: Vec<MessageInfo>,
    aggregation_rate: usize,
}

impl TwoSwapSupervisor<TwoSwapMessage> {
    pub fn new(aggregation_rate: usize) -> Self {
        let (tx, rx) = mpsc::channel();
        TwoSwapSupervisor {
            sender: tx,
            receiver: rx,
            messages: Vec::default(),
            aggregation_rate,
        }
    }

    pub fn sender(&self) -> Sender<TwoSwapMessage> {
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

impl Supervisor<TwoSwapMessage> for TwoSwapSupervisor<TwoSwapMessage> {}

impl Default for TwoSwapSupervisor<TwoSwapMessage> {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        TwoSwapSupervisor {
            sender: tx,
            receiver: rx,
            messages: Vec::default(),
            aggregation_rate: 1,
        }
    }
}
