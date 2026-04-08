# Noir zkTLS Verification Demo

基于 Aztec Noir 的 zkTLS Attestation 验证。

## 目录结构

```
noir-verification-demo/
├── att_verifier_lib/           # Noir 验证库
├── att_verifier_parsing/       # TypeScript 解析库
├── aztec-attestation-sdk/      # Aztec SDK
├── contracts/binance_verifier/ # 验证合约示例
└── scripts/                    # 部署脚本
```

## 快速开始

### 1. 安装 Noir 工具链

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.18
```

### 2. 启动 Aztec 本地网络

```bash
docker-compose up -d
curl http://localhost:8080/status
```

### 3. 安装依赖并构建

```bash
yarn install
yarn build
```

### 4. 编译合约

**重要：必须使用 `aztec compile`**

```bash
cd contracts/binance_verifier
aztec compile
```

输出：`target/binance_verifier-BinanceVerifier.json`

### 5. 部署合约

```bash
cd scripts
yarn tsx deploy-contract.ts
```

部署信息保存到 `deployment.json`。

## 验证类型

| 类型 | 函数 | 说明 |
|------|------|------|
| Hash-based | `verify_hash()` | SHA256 哈希验证 |
| Commitment-based | `verify_commitment()` | Grumpkin 承诺验证 |

## 版本要求

| 组件 | 版本 |
|------|------|
| Aztec Sandbox | 4.2.0-aztecnr-rc.2 |
| @aztec/aztec.js | 4.2.0-aztecnr-rc.2 |
| Noir | 1.0.0-beta.18+ |

## 常见问题

**"bytecode_length_in_fields != 0"**：使用 `aztec compile` 而非 `nargo compile`

**"Method not found"**：检查 sandbox 版本与 SDK 版本是否匹配

**"scopes" 错误**：在 `send()` 时添加 `from: account.address` 参数

## 参考

- [HashCloak Blog](https://hashcloak.com/blog/primus-noir-zktls-tutorial)
- [primus-labs/zktls-verification-noir](https://github.com/primus-labs/zktls-verification-noir)
- [Aztec Documentation](https://docs.aztec.network/)