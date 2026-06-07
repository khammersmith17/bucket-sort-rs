use std::sync::{Arc, Condvar, Mutex};

struct BarrierInner {
    waiting_threads: usize,
    rank_status: Vec<bool>,
}

impl BarrierInner {
    fn mark_done(&mut self, rank: usize) -> bool {
        if !self.rank_status[rank] {
            self.waiting_threads -= 1;
            self.rank_status[rank] = true;
        }
        self.is_done()
    }

    fn is_done(&self) -> bool {
        self.waiting_threads == 0
    }
}

#[derive(Clone)]
pub(crate) struct Barrier {
    inner: Arc<Mutex<BarrierInner>>,
    cond: Arc<Condvar>,
}

impl Barrier {
    pub(crate) fn new(thread_count: usize) -> Barrier {
        let rank_status = vec![false; thread_count];
        let inner = Arc::new(Mutex::new(BarrierInner {
            waiting_threads: thread_count,
            rank_status,
        }));
        Self {
            inner,
            cond: Condvar::new().into(),
        }
    }

    /// Takes in the rank, marks it as done and sleeps the thread until it is woken up when all
    /// threads hit the barrier. Handles spurious wake ups from thread::park.
    pub(crate) fn signal_done(&self, rank: usize) {
        let mut inner = self.inner.lock().unwrap();
        let done = inner.mark_done(rank);

        if done {
            self.cond.notify_all();
        } else {
            inner = self.cond.wait_while(inner, |i| !i.is_done()).unwrap();
        }
    }
}
