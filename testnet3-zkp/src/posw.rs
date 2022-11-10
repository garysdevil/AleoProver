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


// fn coinbase_puzzle_prove(c: &mut Criterion) {
//     let rng = &mut thread_rng();

//     let max_degree = 1 << 15;
//     let max_config = PuzzleConfig { degree: max_degree };
//     let universal_srs = CoinbasePuzzle::<Testnet3>::setup(max_config).unwrap();

//     for degree in [(1 << 13) - 1] {
//         let config = PuzzleConfig { degree };
//         let puzzle = CoinbasePuzzleInst::trim(&universal_srs, config).unwrap();

//         c.bench_function(&format!("CoinbasePuzzle::Prove 2^{}", ((degree + 1) as f64).log2()), |b| {
//             let (epoch_challenge, address, nonce) = sample_inputs(degree, rng);
//             b.iter(|| puzzle.prove(&epoch_challenge, address, nonce).unwrap())
//         });
//     }
// }

// #[cfg(feature = "setup")]
pub fn get_proof() {
    let rng = &mut thread_rng();
    let max_degree = 1 << 15;
    let max_config = PuzzleConfig { degree: max_degree };
    let universal_srs = CoinbasePuzzle::<Testnet3>::setup(max_config).unwrap();

    let degree = (1 << 13) - 1;
    let config = PuzzleConfig { degree };
    let puzzle = CoinbasePuzzleInst::trim(&universal_srs, config).unwrap();

    let (epoch_challenge, address, nonce) = sample_inputs(degree, rng);
    puzzle.prove(&epoch_challenge, address, nonce).unwrap();
}

fn main() {
    #[cfg(feature = "setup")]
    get_proof();
}
