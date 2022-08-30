use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ansi_term::Colour::Cyan;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;

use zkp_testnet2::posw;

pub struct Prover {
    terminator: Arc<AtomicBool>,
    total_proofs: Arc<AtomicU32>,
}

#[tokio::main]
async fn main() {
    let prover = Arc::new(Prover {
        terminator: Default::default(), //Arc::new(AtomicBool::new(false)),
        total_proofs: Default::default(),
    });

    let thread_pools = get_thread_pools();
    let start = std::time::Instant::now();
    // for _ in 0..100 {
    //     let thread_pools = thread_pools.clone();
    //     prover.mine(thread_pools).await;
    // }
    prover.statistic().await;
    prover.mine_with_terminator(thread_pools).await;
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
const CUDA_NUMS: i16 = 3;
#[cfg(feature = "cuda")]
const CUDA_JOBS: i16 = 6;
#[cfg(feature = "cuda")]
static TOTAL_JOBS: i16 = CUDA_JOBS * CUDA_NUMS;
#[cfg(feature = "cuda")]
fn get_thread_pools_gpu() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

    let total_job = TOTAL_JOBS;
    for index in 0..total_job {
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

fn get_thread_pools_cpu() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

    let available_threads = num_cpus::get() as u16;
    let pool_count;
    let pool_threads;
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

impl Prover {
    async fn mine(&self, thread_pools: Vec<Arc<ThreadPool>>) {
        let mut joins = Vec::new();
        let block_template = posw::get_genesis_template();
        for tp in thread_pools.iter() {
            let total_proofs = self.total_proofs.clone();
            let tp = tp.clone();
            let block_template = block_template.clone();
            joins.push(task::spawn_blocking(move || {
                tp.install(|| {
                    let start = Instant::now();
                    posw::get_proof(block_template, rand::random::<u64>());
                    total_proofs.fetch_add(1, Ordering::SeqCst);
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
    async fn mine_with_terminator(&self, thread_pools: Vec<Arc<ThreadPool>>) {
        let mut joins = Vec::new();
        let block_template = posw::get_genesis_template();
        for tp in thread_pools.iter() {
            let total_proofs = self.total_proofs.clone();
            let terminator = self.terminator.clone();
            let tp = tp.clone();
            let block_template = block_template.clone();
            joins.push(task::spawn_blocking(move || {
                // joins.push(task::spawn(async move {
                while !terminator.load(Ordering::SeqCst) {
                    let total_proofs = total_proofs.clone();
                    // let terminator = terminator.clone();
                    let block_template = block_template.clone();
                    tp.install(|| {
                        time_spend("", || {
                            posw::get_proof(block_template, rand::random::<u64>());
                            total_proofs.fetch_add(1, Ordering::SeqCst);
                        });
                    })
                }
            }));
        }
        futures::future::join_all(joins).await;
    }

    async fn statistic(&self) {
        let total_proofs = self.total_proofs.clone();
        task::spawn(async move {
            fn calculate_proof_rate(now: u32, past: u32, interval: u32) -> Box<str> {
                if interval < 1 {
                    return Box::from("---");
                }
                if now <= past || past == 0 {
                    return Box::from("---");
                }
                let rate = (now - past) as f64 / (interval * 60) as f64;
                Box::from(format!("{:.2}", rate))
            }
            let mut log = VecDeque::<u32>::from(vec![0; 60]);
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let proofs = total_proofs.load(Ordering::SeqCst);
                log.push_back(proofs);
                let m1 = *log.get(59).unwrap_or(&0);
                let m5 = *log.get(55).unwrap_or(&0);
                let m15 = *log.get(45).unwrap_or(&0);
                let m30 = *log.get(30).unwrap_or(&0);
                let m60 = log.pop_front().unwrap_or_default();
                println!(
                    "{}",
                    Cyan.normal().paint(format!(
                    "Total proofs: {} (1m: {} p/s, 5m: {} p/s, 15m: {} p/s, 30m: {} p/s, 60m: {} p/s)",
                        proofs,
                        calculate_proof_rate(proofs, m1, 1),
                        calculate_proof_rate(proofs, m5, 5),
                        calculate_proof_rate(proofs, m15, 15),
                        calculate_proof_rate(proofs, m30, 30),
                        calculate_proof_rate(proofs, m60, 60),
                    ))
                );
            }
        });
    }
}

fn time_spend<F>(comment: &str, f: F)
where
    F: FnOnce(),
{
    let start = std::time::Instant::now();
    f();
    let duration = start.elapsed();
    println!("{}. Total time elapsed  {:?}", comment, duration);
}
