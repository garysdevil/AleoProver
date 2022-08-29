use std::sync::Arc;
use std::time::Instant;

use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;

use zkp_testnet2_gpu::posw;

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
    return get_thread_pools_gpu();
}

const CUDA_NUMS: i16 = 3;
const CUDA_JOBS: i16 = 6;

fn get_thread_pools_gpu() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

    // let CUDA_NUMS = 3;
    // let CUDA_JOBS = 6;
    let total_jobs = CUDA_JOBS * CUDA_NUMS;
    for index in 0..total_jobs {
        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(2)
            .thread_name(move |idx| format!("ap-cuda-{}-{}", index, idx))
            .build()
            .unwrap();
        thread_pools.push(Arc::new(pool));
    }
    println!("Pools  CUDA_NUMS={}, CUDA_JOBS={}", CUDA_NUMS, CUDA_JOBS);
    thread_pools
}

async fn mine(thread_pools: Vec<Arc<ThreadPool>>) {
    let mut joins = Vec::new();
    let block_template = posw::get_genesis_template();
    let mut total_jobs = CUDA_JOBS * CUDA_NUMS;
    for tp in thread_pools.iter() {
        let tp = tp.clone();
        let block_template = block_template.clone();
        total_jobs -= 1;
        let gpu_index = total_jobs%CUDA_NUMS;
        joins.push(task::spawn_blocking(move || {
            tp.install(|| {
                let start = Instant::now();
                posw::get_proof_gpu(block_template, rand::random::<u64>(), gpu_index);
                dbg!(total_jobs);
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
