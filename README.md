## 测试
```bash
cd zkp-testnet2
cargo bench --bench testnet2_marlin

cd zkp-testnet3
cargo bench --bench testnet3_marlin
cargo bench testnet2_marlin --features cuda

cargo run --release --example posw_rayon_tokio
cargo run --release --example posw_rayon_tokio --features cuda
```