## ZKP
```bash
# 零知识证明计算测试 CPU模式
cargo run --release --example posw_single

cargo run --release --features cuda --example posw_single

unbuffer cargo run --release --example posw | tee -a local.log
```
