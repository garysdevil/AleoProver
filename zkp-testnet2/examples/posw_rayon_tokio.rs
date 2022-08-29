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

    #[cfg(feature = "cuda")]
    return get_thread_pools_gpu();

    get_thread_pools_cpu()
}

#[cfg(feature = "cuda")]
fn get_thread_pools_gpu() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

    let cuda_num = 1;
    let cuda_jobs = 6;
    let total_jobs = cuda_jobs * cuda_num;
    for index in 0..total_jobs {
        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(2)
            .thread_name(move |idx| format!("ap-cuda-{}-{}", index, idx))
            .build()
            .unwrap();
        thread_pools.push(Arc::new(pool));
    }
    println!(
        "Pools  cuda_num={}, cuda_jobs={}",
        cuda_num, cuda_jobs
    );
    thread_pools
}

fn get_thread_pools_cpu() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

    let available_threads = num_cpus::get() as u16;
    let mut pool_count = 0;
    let mut pool_threads = 0;
    if available_threads % 12 == 0 {
        pool_count = available_threads / 12;
        pool_threads = 12;
    } else if available_threads % 10 == 0 {
        pool_count = available_threads / 10;
        pool_threads = 10;
    } else if available_threads % 8 == 0 {
        pool_count = available_threads / 8;
        pool_threads = 8;
    } else {
        pool_count = available_threads / 6;
        pool_threads = 6;
    }
    println!(
        "Pools  pool_count={}, pool_threads={}",
        pool_count, pool_threads
    );
    for index in 0..pool_count {
        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(pool_threads)
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
