use std::sync::atomic::AtomicBool;
use std::time::Instant;

use snarkvm::dpc::{testnet2::Testnet2, BlockTemplate, Network, PoSWScheme};

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

fn main() {
    let rng = &mut ChaChaRng::seed_from_u64(1234567);

    // Construct the block template.
    let block = Testnet2::genesis_block();
    let block_template = BlockTemplate::new(
        block.previous_block_hash(),
        block.height(),
        block.timestamp(),
        block.difficulty_target(),
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

    for i in 0..10 {
        let start = Instant::now();
        Testnet2::posw()
            .mine(&block_template, &AtomicBool::new(false), rng)
            .unwrap();
        let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
    }

    // let _is_valid = Testnet2::posw().verify_from_block_header(Testnet2::genesis_block().header());
}




// println!("生成一个符合难度的证明");
// println!("验证一个证明");