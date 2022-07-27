use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use snarkvm::dpc::testnet2::Testnet2;
use snarkvm::dpc::Address;
use tracing::{debug, error, info};

mod node;
use crate::node::Node;

mod prover;
use crate::prover::Prover;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(
        default_value = "aleo1sx4cx8mwmdncs3lhqdxss3ummn6p2yevu6xljhhjjka6rdkps5ysntzad0",
        long
    )]
    address: Address<Testnet2>,
    #[clap(default_value = "117.148.141.247:4135", long)]
    pool: SocketAddr,
}

#[tokio::main]
pub async fn main() {
    println!("Hello, world!");

    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false)
        // 设置输出的日志等级
        .with_max_level(tracing::Level::INFO)
        // Build the subscriber
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = Cli::parse();

    println!("address: {:?}", cli.address);

    let node = Node::init(cli.address, cli.pool);

    let prover: Arc<Prover> = match Prover::init(cli.address, node.clone()).await {
        Ok(prover) => prover,
        Err(e) => {
            error!("Unable to initialize prover: {}", e);
            std::process::exit(1);
        }
    };

    Node::start(node);

    std::future::pending::<()>().await;
}
