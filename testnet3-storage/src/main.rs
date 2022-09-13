pub mod store;
use snarkvm::prelude::{MapRead, BlockStorage, TransitionStorage, TransitionMemory, TransactionStorage, DeploymentStorage, TransitionStore, ProgramStorage};
use store::rocksdb::{self, DataMap, Database};
use store::DataID;
use store::{BlockDB, TransitionDB, TransactionDB, DeploymentDB, ProgramDB};

pub mod logger;

#[macro_use]
extern crate tracing;

use snarkvm::console::network::{Network, Testnet3};
// use snarkvm_console::{
//     account::{Address, PrivateKey, ViewKey},
//     network::Testnet3,
// };


pub fn main() {
    logger::initialize_logger(2);
    _transaction_db();
}

fn _transaction_db(){
    let transition_memory: TransitionMemory<Testnet3>;
    transition_memory = TransitionMemory::open(None).unwrap();

    // let transition_store: TransitionStore<Testnet3, TransitionMemory<Testnet3>>;
    let transition_store: TransitionStore<_, TransitionDB<_>>;
    transition_store = TransitionStore::open(None).unwrap();

    let transaction_db: TransactionDB<Testnet3>;
    transaction_db = TransactionDB::open(transition_store).unwrap();

    let kv = transaction_db.id_map();
    let mut kv_iter = kv.keys();
    while let Some(element) = &kv_iter.next() {
        println!("{:?}", element);
    }
}

fn _program_db_map(){
    let program_db: ProgramDB<Testnet3>;
    program_db = ProgramDB::open(None).unwrap();
    let kv = program_db.key_value_id_map();
    let mut kv_iter = kv.keys();
    while let Some(element) = &kv_iter.next() {
        println!("{:?}", element);
    }
}

fn _transition_db_map(){

    let transition_db: TransitionDB<Testnet3>;
    transition_db = TransitionDB::open(None).unwrap();
    let kv = transition_db.locator_map();
    let mut kv_iter = kv.keys();
    while let Some(element) = &kv_iter.next() {
        // println!("{:?}", element);
        println!("{:?}: {:?}", element, transition_db.get(element));
    }
}

fn _transition_memory_map(){

    let transition_memory: TransitionMemory<Testnet3>;
    transition_memory = TransitionMemory::open(None).unwrap();
    let kv = transition_memory.locator_map();
    let mut kv_iter = kv.values();
    while let Some(element) = &kv_iter.next() {
        println!("{:?}", element);
    }
}

fn _block_db__map(){
    let block_db = BlockDB::<Testnet3>::open(None).unwrap();
    // dbg!(block_db);
    // dbg!(block_db.id_map());
    // dbg!(block_db.reverse_id_map());
    // dbg!(block_db.header_map());
    // dbg!(block_db.transactions_map());
    // dbg!(block_db.reverse_transactions_map());
    // // dbg!(block_db.transaction_store());
    // dbg!(block_db.signature_map());
    let kv = block_db.signature_map();
    let mut kv_iter = kv.values();
    while let Some(element) = &kv_iter.next() {
        println!("{:?}", element);
    }
}



fn _show_block_hash_head(){
    use snarkvm::prelude::Header;
    let header_map: DataMap<<Testnet3 as Network>::BlockHash, Header<Testnet3>>;

    header_map = rocksdb::RocksDB::open_map(Testnet3::ID, None, DataID::BlockHeaderMap).unwrap();

    let mut values_iter = header_map.keys();
    while let Some(element) = &values_iter.next() {
        println!("{}", &element);
    }
}

fn _show_transaction_id(){
    
    type Testnet3TransactionID = <Testnet3 as Network>::TransactionID;
    let id_map: DataMap<u32, Testnet3TransactionID>;
    id_map = rocksdb::RocksDB::open_map(Testnet3::ID, None, DataID::TransactionIDMap).unwrap();

    let mut keys_iter = id_map.keys();
    while let Some(element) = &keys_iter.next() {
        let value = id_map.get(element).unwrap();
        if let Some(trx) = value{
            println!("{:?}={:?}", element, trx);
        }else{
            println!("{}", element);
        }
    }
}

fn _show_block_height_hash(){
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