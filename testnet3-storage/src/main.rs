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

    show_block_head();
}

fn show_block_head(){
    use snarkvm::prelude::Header;
    let header_map: DataMap<<Testnet3 as Network>::BlockHash, Header<Testnet3>>;

    header_map = rocksdb::RocksDB::open_map(Testnet3::ID, None, DataID::BlockHeaderMap).unwrap();

    let mut values_iter = header_map.values();
    while let Some(element) = &values_iter.next() {
        println!("{}", &element);
    }
}

fn _show_transaction(){
    
    type Testnet3TransactionID = <Testnet3 as Network>::TransactionID;
    let id_map: DataMap<u32, Testnet3TransactionID>;
    id_map = rocksdb::RocksDB::open_map(Testnet3::ID, None, DataID::TransactionIDMap).unwrap();

    let mut keys_iter = id_map.keys();
    while let Some(element) = &keys_iter.next() {
        println!("{}", element);
        let value = id_map.get(element).unwrap();
        if let Some(trx) = value{
            println!("{:?}={:?}", element, trx);
        }
    }
}

fn _show_block_hash(){
    type Testnet3BlockHash = <Testnet3 as Network>::BlockHash;
    
    let id_map: DataMap<u32, Testnet3BlockHash>;
    id_map = rocksdb::RocksDB::open_map(Testnet3::ID, None, DataID::BlockIDMap).unwrap();

    // 输出所有块高的哈希
    // let mut value_iter = id_map.values();
    // while let Some(element) = &value_iter.next() {
    //     println!("{}", element);
    //     // let value = id_map.get(element).unwrap().unwrap();
    //     // println!("{:?}={:?}", element, value);
    // }

    // 输出本地存储里的高的一个块高的哈希
    let key_iter = id_map.keys();
    let height_max = key_iter.max_by_key(|x| **x).unwrap();
    let height_max = *height_max;
    let hash = id_map.get(&height_max).unwrap().unwrap();
    println!("{}={}", height_max, hash);


}