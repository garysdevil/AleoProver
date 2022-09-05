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

type Testnet3BlockHash = <Testnet3 as Network>::BlockHash;

pub fn main() {
    logger::initialize_logger(2);

    // let path = "/root/vscode/myrust/mylocal/local_path_for_rocksdb_storage";
    // let data_id = DataID::BlockIDMap;

    // let database = rocksdb::RocksDB::open(Testnet3::ID).unwrap();
    // let storage = rocksdb::RocksDB::open_map(N::ID, DataID::BlockIDMap).unwrap();

    let id_map: DataMap<u32, Testnet3BlockHash>;
    id_map = rocksdb::RocksDB::open_map(Testnet3::ID, DataID::BlockIDMap).unwrap();

    let mut key_iter =
        <DataMap<u32, Testnet3BlockHash> as MapRead<u32, Testnet3BlockHash>>::keys(&id_map);
    // let mut value_iter =
    //     <DataMap<u32, Testnet3BlockHash> as MapRead<u32, Testnet3BlockHash>>::values(&id_map);


    // loop {
    //     match &key_iter.next() {
    //         None => break,
    //         Some(element) => {
    //             // println!("{:?}", element);
    //             let value = id_map.get(element).unwrap().unwrap();
    //             println!("{:?}={:?}", element, value);
    //         }
    //     }
    // }
    while let Some(element) = &key_iter.next() {
        // println!("{}", element);
        let value = id_map.get(element).unwrap().unwrap();
        println!("{:?}={:?}", element, value);
    }
    // loop {
    //     match &key_iter.next() {
    //         None => break,
    //         Some(element) => {
    //             println!("{:?}", element);
    //         }
    //     }
    // }
}
