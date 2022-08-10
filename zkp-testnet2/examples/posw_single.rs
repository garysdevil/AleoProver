use std::time::Instant;

mod utils;
mod zkp;

fn main() {
    utils::time_spend("posw_single.rs", || -> () {
        for i in 0..1000 {
            mine(i);
        }
    });
}

fn mine(i: u32) {
    let block_template = zkp::get_genesis_template();

    let start = Instant::now();
    zkp::get_proof(block_template, rand::random::<u64>());
    let duration = start.elapsed();
    println!(
        "{}. Time elapsed in generating a valid proof() is: {:?}",
        i, duration
    );
}
