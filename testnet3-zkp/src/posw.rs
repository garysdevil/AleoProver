#![allow(dead_code)]

// use rand_chacha::ChaChaRng;
// use std::sync::atomic::AtomicBool;

// use snarkvm::dpc::{posw::PoSWCircuit, testnet2::Testnet2, BlockTemplate, Network, PoSWProof};
// use snarkvm::utilities::UniformRand;

// use snarkvm_algorithms::SNARK;

use rand::prelude::*;
use rand::thread_rng;
use snarkvm::console::{account::*, network::Testnet3};
use snarkvm::prelude::{CoinbaseProvingKey, CoinbasePuzzle, EpochChallenge, PuzzleConfig};

type CoinbasePuzzleInst = CoinbasePuzzle<Testnet3>;

fn sample_inputs(
    degree: u32,
    rng: &mut (impl CryptoRng + RngCore),
) -> (EpochChallenge<Testnet3>, Address<Testnet3>, u64) {
    let epoch_challenge = sample_epoch_challenge(degree, rng);
    let (address, nonce) = sample_address_and_nonce(rng);
    (epoch_challenge, address, nonce)
}
fn sample_epoch_challenge(
    degree: u32,
    rng: &mut (impl CryptoRng + RngCore),
) -> EpochChallenge<Testnet3> {
    EpochChallenge::new(rng.next_u32(), Default::default(), degree).unwrap()
}

fn sample_address_and_nonce(rng: &mut (impl CryptoRng + RngCore)) -> (Address<Testnet3>, u64) {
    let private_key = PrivateKey::new(rng).unwrap();
    let address = Address::try_from(private_key).unwrap();
    let nonce = rng.next_u64();
    (address, nonce)
}

pub fn getCoinbaseProvingKey() -> CoinbaseProvingKey<Testnet3> {
    let rng = &mut thread_rng();

    let max_degree = 1 << 15;
    let max_config = PuzzleConfig { degree: max_degree };
    let universal_srs = CoinbasePuzzle::<Testnet3>::setup(max_config, rng).unwrap();

    let degree = (1 << 13) - 1;
    let config = PuzzleConfig { degree };
    let (pk, _) = CoinbasePuzzleInst::trim(&universal_srs, config).unwrap();
    pk
}

pub fn get_proof(pk: CoinbaseProvingKey<Testnet3>) {
    let degree = (1 << 13) - 1;
    let rng = &mut thread_rng();
    let (epoch_challenge, address, nonce) = sample_inputs(degree, rng);
    CoinbasePuzzleInst::prove(&pk, &epoch_challenge, &address, nonce).unwrap();
}
// pub fn get_proof(
//     pk: CoinbaseProvingKey<Testnet3>,
//     epoch_challenge: EpochChallenge<Testnet3>,
//     address: Address<Testnet3>,
//     nonce: u64,
// ) {
//     CoinbasePuzzleInst::prove(&pk, &epoch_challenge, &address, nonce).unwrap();
// }

fn main() {
    // get_proof();
}
