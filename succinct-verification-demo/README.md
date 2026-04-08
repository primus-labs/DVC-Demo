# SP1 zkVM 验证流程

## 环境准备

### 1. 安装 SP1 工具链

```bash
curl -L https://sp1.succinct.xyz | bash
source ~/.bashrc  # 或 source ~/.zshrc
sp1up
cargo prove --version
```

### 2. 安装 Docker（链上验证需要）

```bash
brew install --cask docker  # macOS
docker --version
```

## 编译 zkVM 程序

```bash
cd dvc-succinct-program
cargo prove build
```

ELF 输出：`target/elf-compilation/riscv32im-succinct-zkvm-elf/release/dvc-succinct-program`

## 链下验证

### 生成 Proof

```bash
cd succinct-off-chain-verification-demo
cargo run --release -- generate
```

输出保存到 `output/proof.json`

### 验证 Proof

```bash
cargo run --release -- verify
```

## 链上验证

### 生成 Groth16 Proof

```bash
export SP1_PROVER=cpu
cargo run --release --bin prove_groth16
```

输出：
- `output/groth16_proof.bin` (~500 bytes)
- `output/vk.bin`
- `output/public_values.bin`

### 部署验证合约

```bash
forge install succinctlabs/sp1-contracts
forge build
forge create src/MyVerifier.sol:MyVerifier \
    --rpc-url $RPC_URL \
    --private-key $PRIVATE_KEY \
    --constructor-args $VERIFIER_ADDRESS $VK_HASH
```

### 调用合约验证

```javascript
const verifier = new ethers.Contract(VERIFIER_ADDRESS, ABI, wallet);
const tx = await verifier.verifyProof(proofHex, publicValuesHex);
```

## Proof 模式

| 模式 | Proof 大小 | 验证方式 | 需要 Docker |
|------|-----------|---------|-------------|
| cpu | ~9 MB | 本地 | 否 |
| groth16 | ~500 bytes | 链上 | 是 |

## 命令速查

```bash
# 环境安装
curl -L https://sp1.succinct.xyz | bash && sp1up

# 编译程序
cd dvc-succinct-program && cargo prove build

# 链下验证
cd succinct-off-chain-verification-demo
cargo run --release -- generate    # 生成
cargo run --release -- verify      # 验证

# 链上验证
export SP1_PROVER=cpu
cargo run --release --bin prove_groth16
```