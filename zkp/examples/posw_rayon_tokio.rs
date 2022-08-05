use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

use snarkvm::dpc::{testnet2::Testnet2, BlockTemplate, Network, PoSWScheme};

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::task;

mod utils;

#[tokio::main]
async fn main() {
    let thread_pools = get_thread_pools();
    for _ in 0..100 {
        let thread_pools = thread_pools.clone();
        mine(thread_pools).await;
    }
}

fn get_thread_pools() -> Vec<Arc<ThreadPool>> {
    let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();
    for index in 0..10 {
        let pool = ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(5)
            .thread_name(move |idx| format!("ap-cpu-{}-{}", index, idx))
            .build()
            .unwrap();
        thread_pools.push(Arc::new(pool));
    }
    thread_pools
}

async fn mine(thread_pools: Vec<Arc<ThreadPool>>) {
    let mut joins = Vec::new();
    let block_template = get_template();
    for tp in thread_pools.iter() {
        let tp = tp.clone();
        let block_template = block_template.clone();
        // joins.push(task::spawn(async move {
        joins.push(task::spawn_blocking(move || {
            tp.install(|| {
                let rng = &mut ChaChaRng::seed_from_u64(1234567);
                let start = Instant::now();
                Testnet2::posw()
                    .mine(&block_template, &AtomicBool::new(false), rng)
                    .unwrap();
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

fn get_template() -> BlockTemplate<Testnet2> {
    let difficulty_target: u64 = 18446744073709551615; // block.difficulty_target()

    println!("Difficulty_target is: {:?}", difficulty_target);
    // Construct the block template.
    let block = Testnet2::genesis_block();
    let block_template = BlockTemplate::new(
        block.previous_block_hash(),
        block.height(),
        block.timestamp(),
        difficulty_target,
        block.cumulative_weight(),
        block.previous_ledger_root(),
        block.transactions().clone(),
        block
            .to_coinbase_transaction()
            .unwrap()
            .to_records()
            .next()
            .unwrap(),
    );
    block_template
}
