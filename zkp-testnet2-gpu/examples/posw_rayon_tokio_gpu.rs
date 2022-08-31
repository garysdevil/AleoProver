use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration};

use ansi_term::Colour::Cyan;
use clap::Parser;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;

use zkp_testnet2_gpu::posw;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[cfg(feature = "cuda")]
    #[clap(verbatim_doc_comment)]
    /// Indexes of GPUs to use (starts from 0)
    /// Specify multiple times to use multiple GPUs
    /// Example: -g 0 -g 1 -g 2
    /// Note: Pure CPU proving will be disabled as each GPU job requires one CPU thread as well
    #[clap(short = 'g', long = "cuda")]
    cuda: Option<Vec<i16>>,

    #[cfg(feature = "cuda")]
    #[clap(verbatim_doc_comment)]
    /// Parallel jobs per GPU, defaults to 1
    /// Example: -g 0 -g 1 -j 4
    /// The above example will result in 8 jobs in total
    #[clap(short = 'j', long = "cuda-jobs")]
    jobs: Option<u8>,
}

pub struct Prover {
    terminator: Arc<AtomicBool>,
    total_proofs: Arc<AtomicU32>,
    cuda: Vec<i16>,
    jobs: u8,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let None = cli.cuda {
        dbg!("No GPUs specified. Use -g 0 if there is only one GPU.");
        std::process::exit(1);
    }

    let prover = Arc::new(Prover {
        terminator: Default::default(), //Arc::new(AtomicBool::new(false)),
        total_proofs: Default::default(),
        cuda: cli.cuda.unwrap(),
        jobs: cli.jobs.unwrap_or(1),
    });

    let thread_pools = prover.get_thread_pools();
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

impl Prover {
    fn get_thread_pools(&self) -> Vec<Arc<ThreadPool>> {
        #[cfg(feature = "cuda")]
        return self.get_thread_pools_gpu();
    }

    fn get_thread_pools_gpu(&self) -> Vec<Arc<ThreadPool>> {
        let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

        let total_jobs = self.cuda.len() as u8 * self.jobs;
        for index in 0..total_jobs {
            let pool = ThreadPoolBuilder::new()
                .stack_size(8 * 1024 * 1024)
                .num_threads(2)
                .thread_name(move |idx| format!("ap-cuda-{}-{}", index, idx))
                .build()
                .unwrap();
            thread_pools.push(Arc::new(pool));
        }
        println!("Pools  CUDA_NUMS={:?}, CUDA_JOBS={}", self.cuda, self.jobs);
        thread_pools
    }

    async fn mine_with_terminator(&self, thread_pools: Vec<Arc<ThreadPool>>) {
        let mut joins = Vec::new();
        let block_template = posw::get_genesis_template();
        let cuda_num = self.cuda.len() as i16;
        let cuda_jobs = self.jobs as i16;
        let mut total_jobs = cuda_num * cuda_jobs;
        for tp in thread_pools.iter() {
            let total_proofs = self.total_proofs.clone();
            let terminator = self.terminator.clone();
            let tp = tp.clone();
            let block_template = block_template.clone();
            total_jobs -= 1;
            joins.push(task::spawn_blocking(move || {
                // joins.push(task::spawn(async move {
                while !terminator.load(Ordering::SeqCst) {
                    let total_proofs = total_proofs.clone();
                    // let terminator = terminator.clone();
                    let block_template = block_template.clone();
                    tp.install(|| {
                        time_spend("", || {
                            posw::get_proof_gpu(
                                block_template,
                                rand::random::<u64>(),
                                total_jobs % cuda_num,
                            );
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
