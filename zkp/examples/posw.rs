use std::sync::atomic::AtomicBool;
use std::time::Instant;

use snarkvm::dpc::{testnet2::Testnet2, BlockTemplate, Network, PoSWScheme};

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

fn main() {
    for _ in 0..1000 {
        mine();
    }
}

fn mine() {
    let rng = &mut ChaChaRng::seed_from_u64(1234567);
    let block_template = get_template();

    let start = Instant::now();
    Testnet2::posw()
        .mine(&block_template, &AtomicBool::new(false), rng)
        .unwrap();
    let duration = start.elapsed();
    println!(
        "{}. Time elapsed in generating a valid proof() is: {:?}",
        "=", duration
    );
}

fn get_template() -> BlockTemplate<Testnet2> {
    // let difficulty_target: u64 = 18446744073709551615; // block.difficulty_target()
    let difficulty_target: u64 = 18446744073709551615;

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
