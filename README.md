## 测试
```bash
cd zkp-testnet2
cargo bench --bench testnet2_marlin

cargo run --release --example zkp_rayon_tokio
cargo run --release --example posw_single
```

```
cd zkp-testnet3
cargo bench --bench testnet3_marlin
cargo bench testnet2_marlin --features cuda

cargo run --release --example zkp_rayon_tokio
cargo run --release --example zkp_rayon_tokio --features cuda
```