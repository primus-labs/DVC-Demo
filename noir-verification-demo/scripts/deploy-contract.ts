/**
 * deploy-contract.ts
 *
 * Deploy BinanceVerifier contract to Aztec local network (sandbox)
 */

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function main() {
  console.log("=".repeat(70));
  console.log("Aztec Local Network - BinanceVerifier 部署");
  console.log("=".repeat(70));
  console.log();

  // Load contract artifact
  const artifactPath = path.join(__dirname, "../contracts/binance_verifier/target/binance_verifier-BinanceVerifier.json");

  if (!fs.existsSync(artifactPath)) {
    console.log(`❌ 合约文件不存在: ${artifactPath}`);
    console.log("请先编译合约: cd contracts/binance_verifier && nargo compile");
    process.exit(1);
  }

  console.log("📄 加载合约 artifact...");
  const artifact = JSON.parse(fs.readFileSync(artifactPath, "utf-8"));
  console.log(`   合约名称: ${artifact.name}`);
  console.log();

  // Connect to sandbox
  const PXE_URL = "http://localhost:8080";
  console.log(`🔗 连接 Sandbox: ${PXE_URL}`);

  // Use SDK Client for simpler setup
  const { Client } = await import("aztec-attestation-sdk");

  const client = new Client({ nodeUrl: PXE_URL });
  await client.initialize();
  console.log("   ✅ 连接成功");
  console.log();

  // Get test account
  console.log("👤 获取测试账户...");
  const accountManager = await client.getAccount(0);
  console.log(`   账户地址: ${accountManager.address.toString()}`);

  // Get the wallet
  const wallet = client.getWallet();
  console.log("   ✅ 钱包已就绪");
  console.log();

  // Point H for commitment verification (from Primus example)
  const H = {
    x: BigInt("19978178333943292355349418156359056918133515370613875064303296301489725624535"),
    y: BigInt("13201885744872984780649110422697192888453633882501354541258277493771319153464"),
    is_infinite: false,
  };

  const ALLOWED_URLS = ["https://api.binance.com"];
  const MAX_URL_LEN = 128;

  console.log("🚀 部署 BinanceVerifier 合约...");
  console.log("   这可能需要 1-2 分钟...");
  console.log();

  try {
    // Hash allowed URLs
    const hashedUrls = await client.hashUrls(ALLOWED_URLS, MAX_URL_LEN);

    const { ContractDeployer } = await import("@aztec/aztec.js/deployment");
    const { Fr } = await import("@aztec/aztec.js/fields");

    const deploymentArgs = [accountManager.address, hashedUrls, H];

    const deployer = new ContractDeployer(artifact, wallet);

    const result = await deployer.deploy(...deploymentArgs).send({
      contractAddressSalt: Fr.random(),
      from: accountManager.address,
    });

    const contract = result.contract;

    console.log("=".repeat(70));
    console.log("✅ 部署成功!");
    console.log("=".repeat(70));
    console.log();
    console.log(`合约地址: ${contract.address.toString()}`);
    console.log();

    // Save deployment info
    const deploymentInfo = {
      contractAddress: contract.address.toString(),
      admin: accountManager.address.toString(),
      allowedUrls: ALLOWED_URLS,
      deployedAt: new Date().toISOString(),
    };

    const deploymentPath = path.join(__dirname, "../deployment.json");
    fs.writeFileSync(deploymentPath, JSON.stringify(deploymentInfo, null, 2));
    console.log(`📄 部署信息已保存到: ${deploymentPath}`);

    // Cleanup
    await client.cleanup();

  } catch (error) {
    console.error("❌ 部署失败:", error);
    await client.cleanup().catch(() => {});
    process.exit(1);
  }
}

main().catch(console.error);