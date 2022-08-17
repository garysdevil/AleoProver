use std::sync::Arc;
use std::time::Instant;

use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;

use zkp_testnet3::zkp;

#[tokio::main]
async fn main() {
    let thread_pools = get_thread_pools();
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let thread_pools = thread_pools.clone();
        mine(thread_pools).await;
    }
    let duration = start.elapsed();
    println!(
        "{}. Total time elapsed  {:?}",
        "posw_rayon_tokio.rs", duration
    );
}

fn get_thread_pools() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();
    for index in 0..10 {
        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(7)
            .thread_name(move |idx| format!("ap-cpu-{}-{}", index, idx))
            .build()
            .unwrap();
        thread_pools.push(Arc::new(pool));
    }
    thread_pools
}

async fn mine(thread_pools: Vec<Arc<ThreadPool>>) {
    let mut joins = Vec::new();
    let mut i = 0;
    for tp in thread_pools.iter() {
        let tp = tp.clone();
        i += 1;
        joins.push(task::spawn_blocking(move || {
            tp.install(|| {
                let start = Instant::now();
                algorithm_marlin::snark_prove();
                let duration = start.elapsed();
                println!(
                    "{}. Time elapsed in generating a valid proof() is: {:?}",
                    i, duration
                );
            })
        }));
    }
    futures::future::join_all(joins).await;
}
// cargo run --release --example posw_rayon_tokio
