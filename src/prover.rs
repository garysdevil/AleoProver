
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::dpc::{Address, BlockHeader, BlockTemplate};
use snarkos::{Data, Message};

use std::sync::Arc;
// use std::sync::mpsc::{Sender, Receiver};
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::RwLock;
use tokio::sync::{mpsc, Mutex};
use rayon::{ThreadPool, ThreadPoolBuilder};
use anyhow::Result;
use tracing::{debug, error, info};
use tokio::task;
use rand::thread_rng;
use futures::executor::block_on;

use crate::Node;
// use crate::node::SendMessage;


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
                .num_threads(32 as usize)
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

        let p = prover.clone();
        // 启动一个任务等待获取块高模版，进行挖矿
        let _ = task::spawn(async move {
            while let Some(work) = router_rx.recv().await {
                p.new_work(work).await;
            }
        });

        Ok(prover)
    }

    pub fn router(&self) -> Arc<mpsc::Sender<ProverWork>> {
        self.router.clone()
    }

    async fn new_work(&self, work: ProverWork) {
        let block_template = work.block_template;
        let block_height = block_template.block_height();
        *(self.current_block.write().await) = block_height;
        let share_difficulty = work.share_difficulty;
        info!(
            "Received new work: block {}, share weight {}, share_difficulty {}",
            block_template.block_height(),
            u64::MAX / share_difficulty,
            share_difficulty
        );

        let current_block = self.current_block.clone();
        let terminator = self.terminator.clone();
        let address = self.address;
        let node = self.node.clone();
        let thread_pool = self.thread_pool.clone();
        let total_proofs = self.total_proofs.clone();

        thread_pool.spawn( move || { // 并行挖矿
            info!("======");
            while !terminator.load(Ordering::SeqCst) {
                let result_mining = BlockHeader::mine_once_unchecked(
                    &block_template,
                    &terminator,
                    &mut thread_rng(),
                );
                if let Err(e) = result_mining{
                    error!("result_mining error: {}", e);
                   continue;
                };
                let block_header = result_mining.unwrap();

                let nonce = block_header.nonce();
                let proof = block_header.proof().clone();
                let proof_difficulty = proof.to_proof_difficulty().unwrap_or(u64::MAX);

                // 难度值不满足
                if proof_difficulty > share_difficulty {
                    debug!(
                        "Share difficulty target not met: {} > {}",
                        proof_difficulty, share_difficulty
                    );
                    // *(block_on(total_proofs.write())) += 1;
                    continue;
                }

                // Send a `PoolResponse` to the operator.
                let message = Message::PoolResponse(address, nonce, Data::Object(proof));
                if let Err(error) = block_on(node.router().send(crate::node::SendMessage { message })){
                    error!("Failed to send PoolResponse: {}", error);
                }else{
                    info!("Sent PoolResponse");
                }
            }

        });
    }
}