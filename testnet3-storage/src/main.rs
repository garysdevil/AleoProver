use snarkvm::{
    console::network::Testnet3,
    prelude::{TransitionMemory, TransitionStorage},
    synthesizer::store::helpers::MapRead,
};

pub mod logger;

#[macro_use]
extern crate tracing;

pub fn main() {
    logger::initialize_logger(2);
    _transition_memory_map();
}

fn _transition_memory_map() {
    let transition_memory: TransitionMemory<Testnet3>;
    transition_memory = TransitionMemory::open(None).unwrap();
    let kv = transition_memory.locator_map();
    let mut kv_iter = kv.values();
    while let Some(element) = &kv_iter.next() {
        println!("{:?}", element);
    }
}
