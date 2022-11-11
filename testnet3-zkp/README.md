## ZKP
```bash
# 零知识证明计算测试 CPU模式

# 多核测试
cargo run --release --example posw_rayon_tokio

unbuffer cargo run --release --example posw_rayon_tokio | tee -a local.log
```
