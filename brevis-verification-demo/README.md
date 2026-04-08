# Brevis Pico zkVM 验证流程

## 安装 Pico 工具链

```bash
rustup install nightly-2025-08-04
rustup component add rust-src --toolchain nightly-2025-08-04
cargo +nightly-2025-08-04 install --git https://github.com/brevis-network/pico pico-cli
```

## 编译 zkVM 程序

```bash
cd dvc-brevis-program
cargo pico build
```

ELF 输出：`elf/riscv32im-pico-zkvm-elf`

## 生成 Proof 并验证

```bash
cd brevis-verification-demo
cargo run --release -- generate
```

本地验证成功输出：
```
✅ RISC-V Proof 验证通过
✅ Embed Proof 验证通过
✅✅✅ 本地验证成功！✅✅✅
```

## 查看 Proof 信息

```bash
cargo run --release -- info
```

## 重新验证 Proof

```bash
cargo run --release -- verify
```

## 命令说明

| 命令 | 说明 |
|------|------|
| `generate` | 生成 Proof 并本地验证 |
| `info` | 查看 Proof 信息 |
| `verify` | 从文件加载并验证 |

## 输出文件

```
output/
├── proof_fixture.json    # 本地验证格式
├── proof.json            # 链上验证格式
├── pv_file               # 公开值
└── groth16_witness.json  # Groth16 见证
```

## 链上验证

Brevis 支持链上验证，需要 Docker 生成 Groth16 proof 并部署验证合约。

详细步骤参考: https://pico-docs.brevis.network

## 与 SP1 对比

| 项目 | Succinct SP1 | Brevis Pico |
|------|-------------|-------------|
| 工具链安装 | `sp1up` | `cargo +nightly install pico-cli` |
| 编译命令 | `cargo prove build` | `cargo pico build` |
| ELF 输出路径 | `target/elf-compilation/...` | `elf/` |
| SDK | `sp1-sdk` | `pico-sdk` |
| 入口宏 | `sp1_zkvm::entrypoint!(main)` | `pico_sdk::entrypoint!(main)` |

## 相关链接

- [Brevis 文档](https://pico-docs.brevis.network)
- [Pico GitHub](https://github.com/brevis-network/pico)