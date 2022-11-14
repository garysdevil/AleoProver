use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ansi_term::Colour::Cyan;
use clap::Parser;
use rand::{thread_rng, RngCore};
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;

use snarkvm::console::{account::*, network::Testnet3};
use snarkvm::synthesizer::{CoinbasePuzzle, EpochChallenge};

use zkp_testnet3::posw;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Enable debug logging
    #[clap(short = 'd', long = "debug")]
    debug: bool,
    /// Output log to file
    #[clap(short = 'o', long = "logpath")]
    logpath: Option<String>,
    /// the number of thread pool
    #[clap(default_value = "0", long = "pool_count")]
    pub pool_count: u16,
    ///the number of thread in one thread pool
    #[clap(default_value = "0", long = "pool_thread_count")]
    pub pool_thread_count: u16,
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
    address: Address<Testnet3>,
    puzzle: CoinbasePuzzle<Testnet3>,
    epoch_challenge: EpochChallenge<Testnet3>,
    #[cfg(feature = "cuda")]
    cuda: Option<Vec<i16>>,
    #[cfg(feature = "cuda")]
    jobs: u8,
}

// pub struct ProverEvent {
//     // NewTarget(u64),
//     // NewWork(u32, EpochChallenge<Testnet3>, Address<Testnet3>),
//     // Result(bool, Option<String>),
//     epoch_challenge: Arc<EpochChallenge<Testnet3>>,
// }

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    #[cfg(feature = "cuda")]
    if let None = cli.cuda {
        dbg!("No GPUs specified. Use -g 0 if there is only one GPU.");
        std::process::exit(1);
    }
    config_log(cli.debug, cli.logpath);
    let pool_count = cli.pool_count;
    let pool_thread_count = cli.pool_thread_count;
    
    
    let (puzzle, epoch_challenge, address, _) = posw::get_sample_inputs();
    let prover = Arc::new(Prover {
        terminator: Default::default(), //Arc::new(AtomicBool::new(false)),
        total_proofs: Default::default(),
        address,
        puzzle,
        epoch_challenge: epoch_challenge,
        #[cfg(feature = "cuda")]
        cuda: cli.cuda,
        #[cfg(feature = "cuda")]
        jobs: cli.jobs.unwrap_or(1),
    });

    let thread_pools = prover.get_thread_pools(pool_count, pool_thread_count);
    prover.statistic().await;
    prover.new_work(thread_pools).await;
}

impl Prover {
    fn get_thread_pools(&self, pool_count: u16, pool_thread_count: u16) -> Vec<Arc<ThreadPool>> {
        self.get_thread_pools_bycpumode(pool_count, pool_thread_count)
    }

    fn get_thread_pools_bycpumode(&self, mut pool_count: u16, mut pool_thread_count: u16) -> Vec<Arc<ThreadPool>> {
        let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();

        if pool_count == 0 || pool_thread_count == 0 {
            let available_threads = num_cpus::get() as u16;

            if available_threads % 12 == 0 {
                pool_count = available_threads / 12;
                pool_thread_count = 12;
            } else if available_threads % 10 == 0 {
                pool_count = available_threads / 10;
                pool_thread_count = 10;
            } else if available_threads % 8 == 0 {
                pool_count = available_threads / 8;
                pool_thread_count = 8;
            } else {
                pool_count = available_threads / 6;
                pool_thread_count = 6;
            }
        };

        println!(
            "Pools  pool_count={}, pool_thread_count={}",
            pool_count, pool_thread_count
        );
        for index in 0..pool_count {
            let pool = ThreadPoolBuilder::new()
                .stack_size(8 * 1024 * 1024)
                .num_threads(pool_thread_count.into())
                .thread_name(move |idx| format!("ap-cpu-{}-{}", index, idx))
                .build()
                .unwrap();
            thread_pools.push(Arc::new(pool));
        }

        thread_pools
    }

    async fn new_work(&self, thread_pools: Vec<Arc<ThreadPool>>) {
        let mut joins = Vec::new();
        let epoch_challenge = self.epoch_challenge.clone();
        let address = self.address.clone();
        let puzzle = self.puzzle.clone();

        for tp in thread_pools.iter() {
            let total_proofs = self.total_proofs.clone();
            let terminator = self.terminator.clone();
            let tp = tp.clone();
            let epoch_challenge = epoch_challenge.clone();
            let address = address.clone();
            let puzzle = puzzle.clone();
            joins.push(task::spawn_blocking(move || {
                // joins.push(task::spawn(async move {
                while !terminator.load(Ordering::SeqCst) {
                    let total_proofs = total_proofs.clone();
                    let epoch_challenge = epoch_challenge.clone();
                    let address = address.clone();
                    let puzzle = puzzle.clone();
                    tp.install(|| {
                        debug_time_spend("", || {
                            let nonce = thread_rng().next_u64();
                            let mininum_proof_target: Option<u64> = Option::from(0);
                            posw::get_proof(puzzle, epoch_challenge, address, nonce, mininum_proof_target);
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
                info!(
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

fn debug_time_spend<F>(comment: &str, f: F)
where
    F: FnOnce(),
{
    let start = std::time::Instant::now();
    f();
    let duration = start.elapsed();
    debug!("{}. Total time elapsed  {:?}", comment, duration);
}

fn config_log(debug: bool, file_path: Option<String>) {
    let tracing_level = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing_level)
        .finish();

    if let Some(file_path) = file_path {
        let file = std::fs::File::create(file_path).unwrap();
        let file = tracing_subscriber::fmt::layer()
            .with_writer(file)
            .with_ansi(false);
        tracing::subscriber::set_global_default(subscriber.with(file))
            .expect("unable to set global default subscriber");
    } else {
        tracing::subscriber::set_global_default(subscriber)
            .expect("unable to set global default subscriber");
    }
}
