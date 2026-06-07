use std::sync::{Arc, mpsc::Sender};

/// All senders encapsulated into a single container, that is cheap to copy.
#[derive(Clone)]
pub(crate) struct WorldSender {
    senders: Arc<Vec<Sender<i32>>>,
}

impl WorldSender {
    pub(crate) fn from_senders(senders: Vec<Sender<i32>>) -> WorldSender {
        Self {
            senders: Arc::new(senders),
        }
    }

    pub(crate) fn send(&self, rank: usize, item: i32) {
        self.senders[rank].send(item).unwrap()
    }
}

