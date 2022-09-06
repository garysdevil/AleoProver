pub mod store;
use snarkvm::prelude::MapRead;
use store::rocksdb::{self, DataMap, Database};
use store::DataID;

pub mod logger;

// use crate::store::{
//     rocksdb::{self, DataMap, Database},
//     DataID,
//     TransactionDB,
//     TransitionDB,
// };

#[macro_use]
extern crate tracing;

use snarkvm::console::network::{Network, Testnet3};
// use snarkvm_console::{
//     account::{Address, PrivateKey, ViewKey},
//     network::Testnet3,
// };


pub fn main() {
    logger::initialize_logger(2);

    main1();
    
}

// fn main2(){
    
//     // type Testnet3BlockHash = <Testnet3 as Network>::BlockHash;
//     // let id_map: DataMap<u32, Testnet3BlockHash>;
//     let id_map = rocksdb::RocksDB::open_map(Testnet3::ID, DataID::BlockIDMap).unwrap();
//     let mut key_iter = id_map.keys();
//     while let Some(element) = &key_iter.next() {
//         // println!("{}", element);
//         let value = id_map.get(element).unwrap().unwrap();
//         println!("{:?}={:?}", element, value);
//     }
// }

fn main1(){
    type Testnet3BlockHash = <Testnet3 as Network>::BlockHash;
    
    let id_map: DataMap<u32, Testnet3BlockHash>;
    id_map = rocksdb::RocksDB::open_map(Testnet3::ID, None, DataID::BlockIDMap).unwrap();
    let key_iter = id_map.keys();
    let height_max = key_iter.max_by_key(|x| **x).unwrap();
    let height_max = *height_max;
    let hash = id_map.get(&height_max).unwrap().unwrap();
    println!("{}={}", height_max, hash);

    let mut value_iter = id_map.values();
    while let Some(element) = &value_iter.next() {
        println!("{}", element);
        // let value = id_map.get(element).unwrap().unwrap();
        // println!("{:?}={:?}", element, value);
    }
}