## ZKP
```bash
# 零知识证明计算测试 CPU模式
cargo run --release --example posw

unbuffer a | tee -a local.log
```

## 测试
### 20220808
- CPU
    - Intel(R) Xeon(R) CPU E5-2695 v4 @ 2.10GHz
    - 72C
- 测试方式
    - 并行计算获取10个结果，进行100次并行计算。

- ``cargo run --release --example posw_tokio``
    - Total time elapsed  911.133755737s
    - 88.1%

- ``run --release --example posw_rayon_tokio``
    - Total time elapsed  581.062611586s
    - 58.1%
