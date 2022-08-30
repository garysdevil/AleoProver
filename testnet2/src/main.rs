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
    #[clap(default_value = "127.0.0.1:4135", long)]
    pool: SocketAddr,
    /// Specify the verbosity of the node [options: 0, 1, 2, 3, 4]
    #[clap(default_value = "2", long, action)]
    pub verbosity: u8,
}

#[tokio::main]
pub async fn main() {
    println!("Hello, world!");
    let cli = Cli::parse();

    let subscriber_builder = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false);

    let subscriber_builder = match cli.verbosity {
        0 => subscriber_builder.with_max_level(tracing::Level::ERROR),
        1 => subscriber_builder.with_max_level(tracing::Level::WARN),
        2 => subscriber_builder.with_max_level(tracing::Level::INFO),
        3 => subscriber_builder.with_max_level(tracing::Level::DEBUG),
        4 => subscriber_builder.with_max_level(tracing::Level::TRACE),
        _ => subscriber_builder.with_max_level(tracing::Level::INFO),
    };
    let subscriber = subscriber_builder.finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    println!("address: {:?}", cli.address);

    let node = Node::init(cli.address, cli.pool);

    let prover: Arc<Prover> = match Prover::init(cli.address, node.clone()).await {
        Ok(prover) => prover,
        Err(e) => {
            error!("Unable to initialize prover: {}", e);
            std::process::exit(1);
        }
    };

    // node收到挖矿块高模版则将其发送给prover任务进行挖矿
    Node::start(prover.router(), node);

    std::future::pending::<()>().await;
}
