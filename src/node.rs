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
use tokio_stream::StreamExt;

use futures::SinkExt;


use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::dpc::Address;
use snarkos::{Data, Message};
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
            let framed_option = match socket {
                Err(e) => {
                    error!("Failed to connect to operator: {}", e);
                    // sleep(Duration::from_secs(5)).await;
                     None
                }
                Ok(socket) => {
                    info!("Connected to {}", node.operator);
                    let mut framed = Framed::new(socket, Message::<Testnet2, Prover<Testnet2>>::PeerRequest); // 创建一个帧实例(数据流, 帧消息)
                    let challenge = Message::ChallengeRequest( // 构建一个帧消息，回应operator节点发送过来的ChallengeRequest
                        12,
                        Testnet2::ALEO_MAXIMUM_FORK_DEPTH,
                        NodeType::Prover,
                        State::Ready,
                        4132,
                        thread_rng().gen(), // 生成一个随机数
                        0,
                    );
                    if let Err(e) = framed.send(challenge).await { // 发送帧
                        error!("Error sending challenge request: {}", e);
                        None
                    } else {
                        info!("Sent challenge request");
                        Some(framed)
                    }
                }
            };
            // let receiver = &mut *receiver.lock().await;
            if let None = framed_option {  // 如果 var_name为非None，则会把9被赋值给temp变量，执行块内的代码
                panic!("framed_option = None");
            }
            let mut framed = framed_option.unwrap();

            loop {
                // 处理对等节点发送过来的帧
                match framed.next().await{
                    Some(Err(e)) => {
                        warn!("Failed to read the message: {:?}", e);
                    }
                    None => {
                        error!("Disconnected from operator");
                        sleep(Duration::from_secs(5)).await;
                        break;
                    }
                    Some(Ok(message)) => {
                        info!("Received {} from operator", message.name());
                        match message {
                            Message::ChallengeRequest(..) => {
                                let resp = Message::ChallengeResponse(Data::Object(Testnet2::genesis_block().header().clone()));
                                if let Err(e) = framed.send(resp).await {
                                    error!("Error sending challenge response: {:?}", e);
                                } else {
                                    debug!("Sent challenge response");
                                }
                            }
                            Message::ChallengeResponse(..) => {
                                let ping = Message::<Testnet2, Prover<Testnet2>>::Ping(
                                    12,
                                    Testnet2::ALEO_MAXIMUM_FORK_DEPTH,
                                    NodeType::Prover,
                                    State::Ready,
                                    Testnet2::genesis_block().hash(),
                                    Data::Object(Testnet2::genesis_block().header().clone()),
                                );
                                if let Err(e) = framed.send(ping).await {
                                    error!("Error sending ping: {:?}", e);
                                } else {
                                    debug!("Sent ping");
                                }
                            }
                            Message::Pong(..) => {
                                let register = Message::<Testnet2, Prover<Testnet2>>::PoolRegister(node.address);
                                if let Err(e) = framed.send(register).await {
                                    error!("Error sending pool register: {:?}", e);
                                } else {
                                    debug!("Sent pool register");
                                }
                            }
                            _ => {
                                debug!("Unhandled message: {}", message.name());
                            }
                        }
                    }
                }  
            }
        });
    
    }
}

