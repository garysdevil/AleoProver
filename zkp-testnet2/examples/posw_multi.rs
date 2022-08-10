use std::time::Instant;

mod utils;
mod zkp;

fn main() {
    utils::time_spend("posw_multi.rs", || -> () {
        for _ in 0..100 {
            mine();
        }
    });
}

fn mine() {
    let block_template = zkp::get_genesis_template();
    let mut joins = Vec::new();
    for i in 0..10 {
        let block_template = block_template.clone();
        joins.push(std::thread::spawn(move || {
            let start = Instant::now();
            zkp::get_proof(block_template, rand::random::<u64>());
            let duration = start.elapsed();
            println!(
                "{}. Time elapsed in generating a valid proof() is: {:?}",
                i, duration
            );
        }));
    }
    for thread in joins {
        thread.join().unwrap();
    }
}
