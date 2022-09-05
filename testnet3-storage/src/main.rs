pub mod store;
use snarkvm::prelude::MapRead;
use store::rocksdb::{self,Database,DataMap};
use store::rocksdb::map::*;
use store::{
    DataID
};

// use crate::store::{
//     rocksdb::{self, DataMap, Database},
//     DataID,
//     TransactionDB,
//     TransitionDB,
// };

#[macro_use]
extern crate tracing;


use snarkvm::console::network::Testnet3;
use snarkvm::console::network::Network;
// use snarkvm_console::{
//     account::{Address, PrivateKey, ViewKey},
//     network::Testnet3,
// };

type Testnet3BlockHash = <Testnet3 as Network>::BlockHash;

pub fn main() {
    initialize_logger(2);

    // let path = "/root/vscode/myrust/mylocal/local_path_for_rocksdb_storage";

    let data_id = DataID::BlockIDMap;

    // let database = rocksdb::RocksDB::open(Testnet3::ID).unwrap();
    // let storage = rocksdb::RocksDB::open_map(N::ID, DataID::BlockIDMap).unwrap();

    let id_map: DataMap<u32, Testnet3BlockHash>;
    id_map = rocksdb::RocksDB::open_map(Testnet3::ID, DataID::BlockIDMap).unwrap();

    // let mut key_iter = <DataMap<u32, Testnet3BlockHash> as MapRead<u32, Testnet3BlockHash>>::keys(&id_map);
    let mut key_iter = <DataMap<u32, Testnet3BlockHash> as MapRead<u32, Testnet3BlockHash>>::values(&id_map);
    // for element in key_iter.next() {
    //     println!("{:?}",element);
    // }

    loop{
        match key_iter.next(){
            None => {break}
            Some(element) => {
                println!("{:?}",element);
            }
        }
    }

}




pub fn initialize_logger(verbosity: u8) {
    match verbosity {
        0 => std::env::set_var("RUST_LOG", "info"),
        1 => std::env::set_var("RUST_LOG", "debug"),
        2 | 3 => std::env::set_var("RUST_LOG", "trace"),
        _ => std::env::set_var("RUST_LOG", "info"),
    };

    // Filter out undesirable logs.
    let filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive("hyper::client=off".parse().unwrap())
        .add_directive("hyper::proto=off".parse().unwrap())
        .add_directive("jsonrpsee=off".parse().unwrap())
        .add_directive("mio=off".parse().unwrap())
        .add_directive("rusoto_core=off".parse().unwrap())
        .add_directive("tokio_util=off".parse().unwrap())
        .add_directive("want=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    // Initialize tracing.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(verbosity == 3)
        .try_init();
}
