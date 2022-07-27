use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rand::{thread_rng, Rng};
use snarkvm::prelude::Network;
use tracing::{debug, error, info, warn};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tokio::time::{sleep, timeout};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use futures::SinkExt;


use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::dpc::Address;
use snarkos::{Message};
use snarkos::environment::Prover;
use snarkos::helpers::{NodeType, State};

use crate::prover::ProverWork;

#[derive(Debug)]
pub struct SendMessage {
    pub message: Message<Testnet2, Prover<Testnet2>>,
}

// #[derive(Debug)]
pub struct Node {
    address: Address<Testnet2>,
    operator: SocketAddr,
    router: Arc<Sender<SendMessage>>,
    receiver: Arc<Mutex<Receiver<SendMessage>>>,
}


impl Node {
    pub fn init(address: Address<Testnet2>, operator: SocketAddr) -> Arc<Self> {
        let (router_tx, router_rx) = mpsc::channel(1024);
        Arc::new(Self {
            address,
            operator,
            router: Arc::new(router_tx),
            receiver: Arc::new(Mutex::new(router_rx)),
        })
    }

    pub fn router(&self) -> Arc<Sender<SendMessage>> {
        self.router.clone()
    }

    pub fn receiver(&self) -> Arc<Mutex<Receiver<SendMessage>>> {
        self.receiver.clone()
    }

    pub fn start(
        // prover_router: Arc<Sender<ProverWork>>,
        node: Arc<Node>,
        // receiver: Arc<Mutex<Receiver<SendMessage>>>,
    ) {
        task::spawn(async move {
            info!("Connecting to operator...");
            let socket = match timeout(Duration::from_secs(5), TcpStream::connect(&node.operator)).await {
                Err(_) => {
                    error!("Failed to connect to operator: Timed out");
                    panic!();
                    // sleep(Duration::from_secs(5)).await;
                },
                Ok(socket) => {
                    socket
                }
            };
            match socket {
                Err(e) => {
                    error!("Failed to connect to operator: {}", e);
                    // sleep(Duration::from_secs(5)).await;
                }
                Ok(socket) => {
                    info!("Connected to {}", node.operator);
                    let mut framed = Framed::new(socket, Message::<Testnet2, Prover<Testnet2>>::PeerRequest);
                    let challenge = Message::ChallengeRequest( // 构建一个帧，回应operator节点发送过来的ChallengeRequest
                        12,
                        Testnet2::ALEO_MAXIMUM_FORK_DEPTH,
                        NodeType::Prover,
                        State::Ready,
                        4132,
                        thread_rng().gen(), // 生成一个随机数
                        0,
                    );
                    // if let Err(e) = framed.send(challenge).await {
                    //     error!("Error sending challenge request: {}", e);
                    // } else {
                    //     debug!("Sent challenge request");
                    // }
                }
            }

            
        });
    
    }
}

