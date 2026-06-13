mod barrier;
mod message;
mod worker;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

fn get_thread_count() -> usize {
    let Ok(thread_count) = thread::available_parallelism() else {
        panic!("Unable to determine the amount of parallel threads")
    };
    thread_count.get()
}

fn construct_channels(world_size: usize) -> (Vec<Sender<i32>>, Vec<Receiver<i32>>) {
    let mut senders: Vec<Sender<i32>> = Vec::with_capacity(world_size);
    let mut receivers: Vec<Receiver<i32>> = Vec::with_capacity(world_size);
    for _ in 0..world_size {
        let (sender, receiver) = channel();
        senders.push(sender);
        receivers.push(receiver);
    }

    (senders, receivers)
}

fn define_bin_edges(world_size: usize, slice: &[i32]) -> Vec<i32> {
    let mut max_value = i32::MIN;
    let mut min_value = i32::MAX;

    slice.iter().for_each(|item| {
        max_value = max_value.max(*item);
        min_value = min_value.min(*item);
    });

    let step_size = ((max_value as i64 - min_value as i64) / world_size as i64) as i32;
    let mut curr = min_value;
    let mut bins = vec![0_i32; world_size];
    bins.iter_mut().for_each(|item| {
        *item = curr;
        curr += step_size;
    });

    bins
}

/// Sort an i32 slice. Makes a copy.
pub fn sort_i32(slice: &[i32]) -> Vec<i32> {
    let world_size = get_thread_count();
    let (senders, receivers) = construct_channels(world_size);
    let start_chunk_size = (slice.len() as f64 / world_size as f64).ceil() as usize;

    let bin_edges = define_bin_edges(world_size, slice);
    let bin_resolver = worker::RankResolver::new(bin_edges, world_size);

    let result = thread::scope(|s| {
        let world_sender = message::WorldSender::from_senders(senders);
        let barrier = barrier::Barrier::new(world_size);
        let handles: Vec<_> = slice
            .chunks(start_chunk_size)
            .zip(receivers.into_iter())
            .enumerate()
            .map(|(rank, (chunk, recv))| {
                let world = world_sender.clone();
                let bin_resolver = bin_resolver.clone();
                let barrier = barrier.clone();
                s.spawn(move || {
                    worker::rank_worker(rank, recv, world, bin_resolver, chunk, barrier)
                })
            })
            .collect();

        drop(world_sender);

        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .flatten()
            .collect()
    });
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xorshift32(state: &mut u32) -> u32 {
        *state ^= *state << 13;
        *state ^= *state >> 17;
        *state ^= *state << 5;
        *state
    }

    #[test]
    fn test_sort_one_million() {
        let mut state: u32 = 0xdeadbeef;
        let mut data: Vec<i32> = (0..1_000_000)
            .map(|_| xorshift32(&mut state) as i32)
            .collect();

        let sorted = sort_i32(&data);
        data.sort();

        assert_eq!(sorted.len(), data.len());
        assert_eq!(sorted, data);
    }

    #[test]
    fn test_ten_million_vs_builtin() {
        let mut state: u32 = 0xcafebabe;
        let data: Vec<i32> = (0..10_000_000)
            .map(|_| xorshift32(&mut state) as i32)
            .collect();
        let mut reference = data.clone();

        let t0 = std::time::Instant::now();
        let sorted = sort_i32(&data);
        let bucket_time = t0.elapsed();

        let t1 = std::time::Instant::now();
        reference.sort();
        let std_time = t1.elapsed();

        println!("bucket sort: {bucket_time:?}"); // ~2 seconds
        println!("std sort:    {std_time:?}"); // ~5 seconds

        assert_eq!(sorted, reference);
    }
}
