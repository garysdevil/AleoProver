
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::dpc::{Address, BlockTemplate};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool};
// use std::sync::mpsc::{Sender, Receiver};

use tokio::sync::RwLock;
use tokio::sync::{mpsc, Mutex};
use rayon::{ThreadPool, ThreadPoolBuilder};
use anyhow::Result;

use crate::Node;


#[derive(Debug)]
pub struct ProverWork {
    share_difficulty: u64,
    block_template: BlockTemplate<Testnet2>,
}
impl ProverWork {
    pub fn new(share_difficulty: u64, block_template: BlockTemplate<Testnet2>) -> Self {
        Self {
            share_difficulty,
            block_template,
        }
    }
}


pub struct Prover {
    address: Address<Testnet2>,
    thread_pool: Arc<ThreadPool>,
    router: Arc<mpsc::Sender<ProverWork>>,
    node: Arc<Node>,
    terminator: Arc<AtomicBool>,
    current_block: Arc<RwLock<u32>>,
    total_proofs: Arc<RwLock<u32>>,
}

impl Prover {
    pub async fn init(
        address: Address<Testnet2>,
        node: Arc<Node>,
    ) -> Result<Arc<Self>> {
        let pool = ThreadPoolBuilder::new()
                .stack_size(8 * 1024 * 1024)
                .num_threads(8 as usize)
                .build()?;

        let terminator = Arc::new(AtomicBool::new(false));
        let (router_tx, mut router_rx) = mpsc::channel(1024);

        let prover = Arc::new(Self {
            address,
            thread_pool: Arc::new(pool),
            router: Arc::new(router_tx),
            node,
            terminator,
            current_block: Default::default(),
            total_proofs: Default::default(),
        });

        Ok(prover)
    }
    pub fn router(&self) -> Arc<mpsc::Sender<ProverWork>> {
        self.router.clone()
    }
}