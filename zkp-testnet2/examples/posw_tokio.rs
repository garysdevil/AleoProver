use std::sync::atomic::AtomicBool;
use std::time::Instant;

use snarkvm::dpc::{testnet2::Testnet2, BlockTemplate, Network, PoSWScheme};

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

use tokio::task;

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    for _ in 0..100 {
        mine().await;
    }
    let duration = start.elapsed();
    println!("{}. Total time elapsed  {:?}", "posw_tokio.rs", duration);
}

async fn mine() {
    let block_template = get_template();
    let mut joins = Vec::new();
    for i in 0..10 {
        let block_template = block_template.clone();
        joins.push(task::spawn_blocking(move || {
            let rng = &mut ChaChaRng::seed_from_u64(1234567);
            let start = Instant::now();
            Testnet2::posw()
                .mine(&block_template, &AtomicBool::new(false), rng)
                .unwrap();
            let duration = start.elapsed();
            println!(
                "{}. Time elapsed in generating a valid proof() is: {:?}",
                i, duration
            );
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
