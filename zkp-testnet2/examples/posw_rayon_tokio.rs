use std::sync::Arc;
use std::time::Instant;

use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;

use zkp_testnet2::posw;

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

async fn mine(thread_pools: Vec<Arc<ThreadPool>>) {
    let mut joins = Vec::new();
    let block_template = posw::get_genesis_template();
    for tp in thread_pools.iter() {
        let tp = tp.clone();
        let block_template = block_template.clone();
        // joins.push(task::spawn(async move {
        joins.push(task::spawn_blocking(move || {
            // task::spawn_blocking 比 task::spawn 快0.0427s
            tp.install(|| {
                let start = Instant::now();
                posw::get_proof(block_template, rand::random::<u64>());
                let duration = start.elapsed();
                println!(
                    "{}. Time elapsed in generating a valid proof() is: {:?}",
                    "-", duration
                );
            })
        }));
    }
    futures::future::join_all(joins).await;
}
