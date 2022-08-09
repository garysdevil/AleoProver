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
    - 并行计算获取10个结果，进行100次并行计算。或者进行1000次计算。

- ``cargo run --release --example posw_tokio``
    - Total time elapsed  911.133755737s
    - 9.11s/posw
    - 88.1%

- ``run --release --example posw_rayon_tokio``
    - num_threads(5)
    - Total time elapsed  581.062611586s
    - 5.81s/posw
    - 58.1%

- ``run --release --example posw_rayon_multi``
    - num_threads(5)
    - Total time elapsed  885.977078525s
    - 8.85s/posw
    - 88.1%

- ``cargo run --release --example posw_rayon_multi``
    - num_threads(5)
    - Total time elapsed  575.091173021s
    - 5.75s/posw
    - 57.9%

- ``cargo run --release --example posw_single``
    - Total time elapsed  2471.244337022s
    - 2.471s/posw
    - 30.2%

- ``cargo run --release --example posw_rayon``
    - num_threads(5)
    - Total time elapsed  4450.445543527s
    - 4.45s/posw
    - 6.25%

- ``cargo run --release --example posw_rayon``
    - num_threads(20)
    - Total time elapsed  2452.189937946s
    - 2.452s/posw
    - 17.9%