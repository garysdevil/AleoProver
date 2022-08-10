use std::sync::Arc;
use std::time::Instant;

use rayon::{ThreadPool, ThreadPoolBuilder};

mod utils;
mod zkp;

fn main() {
    utils::time_spend("posw_rayon_multi.rs", || -> () {
        let thread_pools = get_thread_pools();
        for _ in 0..100 {
            let thread_pools = thread_pools.clone();
            mine(thread_pools);
        }
    });
}

fn get_thread_pools() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();
    for index in 0..4 {
        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(20)
            .thread_name(move |idx| format!("ap-cpu-{}-{}", index, idx))
            .build()
            .unwrap();
        thread_pools.push(Arc::new(pool));
    }
    thread_pools
}

fn mine(thread_pools: Vec<Arc<ThreadPool>>) {
    let block_template = zkp::get_genesis_template();
    let mut joins = Vec::new();
    for tp in thread_pools.iter() {
        let tp = tp.clone();
        let block_template = block_template.clone();
        joins.push(std::thread::spawn(move || {
            tp.install(|| {
                let start = Instant::now();
                zkp::get_proof(block_template, rand::random::<u64>());
                let duration = start.elapsed();
                println!(
                    "{}. Time elapsed in generating a valid proof() is: {:?}",
                    "-", duration
                );
            })
        }));
    }
    for thread in joins {
        thread.join().unwrap();
    }
}
