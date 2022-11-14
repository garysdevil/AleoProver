#![allow(clippy::single_element_loop)]

use snarkvm::console::{account::*, network::Testnet3};
use snarkvm::synthesizer::{CoinbasePuzzle, EpochChallenge, PuzzleConfig};

use rand::{self, thread_rng, CryptoRng, RngCore};

type CoinbasePuzzleInst = CoinbasePuzzle<Testnet3>;

fn sample_inputs(
    degree: u32,
    rng: &mut (impl CryptoRng + RngCore),
) -> (EpochChallenge<Testnet3>, Address<Testnet3>, u64) {
    let epoch_challenge = sample_epoch_challenge(degree, rng);
    let (address, nonce) = sample_address_and_nonce(rng);
    (epoch_challenge, address, nonce)
}

fn sample_epoch_challenge(degree: u32, rng: &mut (impl CryptoRng + RngCore)) -> EpochChallenge<Testnet3> {
    EpochChallenge::new(rng.next_u32(), Default::default(), degree).unwrap()
}

fn sample_address_and_nonce(rng: &mut (impl CryptoRng + RngCore)) -> (Address<Testnet3>, u64) {
    let private_key = PrivateKey::new(rng).unwrap();
    let address = Address::try_from(private_key).unwrap();
    let nonce = rng.next_u64();
    (address, nonce)
}

pub fn get_sample_inputs() -> ( CoinbasePuzzle<Testnet3>, EpochChallenge<Testnet3>, Address<Testnet3>, u64){
    let rng = &mut thread_rng();
    let max_degree = 1 << 15;
    let max_config = PuzzleConfig { degree: max_degree };
    let universal_srs = CoinbasePuzzle::<Testnet3>::setup(max_config).unwrap();

    let degree = (1 << 13) - 1;
    let config = PuzzleConfig { degree };
    let puzzle = CoinbasePuzzleInst::trim(&universal_srs, config).unwrap();

    let (epoch_challenge, address, nonce) = sample_inputs(degree, rng);

    (puzzle, epoch_challenge, address, nonce)
}

pub fn get_proof(puzzle: CoinbasePuzzle<Testnet3>, epoch_challenge: EpochChallenge<Testnet3>, address: Address<Testnet3>, nonce: u64, mininum_proof_target: Option<u64>) {
    puzzle.prove(&epoch_challenge, address, nonce, mininum_proof_target).unwrap();
}

pub fn main() {
    let start = std::time::Instant::now();

    let (puzzle, epoch_challenge, address, nonce) = get_sample_inputs();

    let duration = start.elapsed();
    println!("{}. Total time elapsed  {:?}", "1. ", duration);

    let mininum_proof_target: Option<u64> = Option::from(0);
    get_proof(puzzle, epoch_challenge, address, nonce, mininum_proof_target);

    let duration = start.elapsed();
    println!("{}. Total time elapsed  {:?}", "2. ", duration);
}
