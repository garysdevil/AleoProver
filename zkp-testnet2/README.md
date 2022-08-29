## ZKP
```bash

cp zkp-testnet2/Cargo.remote.toml zkp-testnet2/Cargo.toml
# 零知识证明计算测试 CPU模式

# 单核测试
cargo run --release --example posw_single
cargo run --release --features cuda --example posw_single

# 多核测试
cargo run --release --example posw_rayon_tokio
cargo run --release --features cuda --example posw_rayon_tokio

unbuffer cargo run --release --example posw_rayon_tokio | tee -a local.log
```
