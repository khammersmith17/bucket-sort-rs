use super::{barrier::Barrier, message::WorldSender};
use std::sync::{Arc, mpsc::Receiver};

#[derive(Clone)]
pub(crate) struct RankResolver {
    bin_edges: Arc<Vec<i32>>,
    world_size: usize,
}

impl RankResolver {
    pub(crate) fn new(bin_edges: Vec<i32>, world_size: usize) -> RankResolver {
        Self {
            bin_edges: Arc::new(bin_edges),
            world_size,
        }
    }

    fn resolve(&self, item: i32) -> usize {
        self.bin_edges
            .partition_point(|edge| item >= *edge)
            .min(self.world_size - 1)
    }
}

pub(crate) fn rank_worker(
    rank: usize,
    receiver: Receiver<i32>,
    world: WorldSender,
    resolver: RankResolver,
    starting_sample: &[i32],
    barrier: Barrier,
) -> Vec<i32> {
    /*
     * 2 Phases:
     *   1. Distribute data
     *   2. Sort and return
     * */

    let approx_per_rank_load = starting_sample.len();

    // Distribute
    for sample in starting_sample.iter() {
        let dest_rank = resolver.resolve(*sample);
        world.send(dest_rank, *sample);
    }
    drop(world);

    barrier.signal_done(rank);

    let mut result: Vec<i32> = Vec::with_capacity(approx_per_rank_load);

    while let Ok(item) = receiver.recv() {
        result.push(item);
    }

    result.sort();
    result
}
