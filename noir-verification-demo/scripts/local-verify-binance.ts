/**
 * local-verify-binance.ts
 *
 * Test script to verify parsing and data extraction from Binance attestation.
 * This demonstrates the parsing flow without requiring a running Aztec network.
 */

import fs from "fs";
import path from "path";

// Direct import from parsing library
import { parseHashingData } from "../att_verifier_parsing/dist/index.js";

// Configuration
const MAX_RESPONSE_NUM = 1;
const MAX_URL_LEN = 128;
const ALLOWED_URLS = ["https://api.binance.com"];
const ATT_PATH = path.join(path.dirname(new URL(import.meta.url).pathname), "../testdata/binance-attestation.json");

// ETH balance threshold: 0.1 ETH = 100000 (in units of 0.000001 ETH)
const THRESHOLD = 100000;

/**
 * Parse ETH balance from Binance attestation content
 */
function parseEthBalance(content: string): number {
  try {
    const data = JSON.parse(content);
    const ethBalance = data.balances?.find((b: any) => b.asset === "ETH");
    if (ethBalance) {
      // Convert to units of 0.000001 ETH
      const free = parseFloat(ethBalance.free || "0");
      const locked = parseFloat(ethBalance.locked || "0");
      return Math.floor((free + locked) * 1000000);
    }
    return 0;
  } catch {
    return 0;
  }
}

async function main() {
  console.log("=".repeat(70));
  console.log("Noir zkTLS Verification Demo - Binance Balance Parsing Test");
  console.log("=".repeat(70));
  console.log();

  // Check if attestation file exists
  if (!fs.existsSync(ATT_PATH)) {
    console.log(`❌ Attestation file not found: ${ATT_PATH}`);
    console.log("Please copy testdata/binance-attestation.json from the root project.");
    process.exit(1);
  }

  // Load attestation data
  console.log("📄 Loading attestation data...");
  const rawData = JSON.parse(fs.readFileSync(ATT_PATH, "utf-8"));
  console.log(`   verification_type: ${rawData.verification_type}`);
  console.log(`   attestor: ${rawData.public_data[0].attestor}`);
  console.log();

  // Parse attestation
  console.log("🔧 Parsing attestation...");
  try {
    const parsed = parseHashingData(rawData, {
      maxResponseNum: MAX_RESPONSE_NUM,
      allowedUrls: ALLOWED_URLS,
      maxUrlLen: MAX_URL_LEN,
    });

    console.log("   ✅ Parsing successful!");
    console.log();
    console.log("📊 Parsed Data:");
    console.log(`   Public Key X: ${Buffer.from(parsed.publicKeyX).toString("hex").slice(0, 16)}...`);
    console.log(`   Public Key Y: ${Buffer.from(parsed.publicKeyY).toString("hex").slice(0, 16)}...`);
    console.log(`   Message Hash: ${Buffer.from(parsed.hash).toString("hex").slice(0, 16)}...`);
    console.log(`   Signature: ${Buffer.from(parsed.signature).toString("hex").slice(0, 16)}...`);
    console.log(`   Request URLs: ${parsed.requestUrls.length} URL(s)`);
    console.log(`   Allowed URLs: ${parsed.allowedUrls.length} URL(s)`);
    console.log(`   Data Hashes: ${parsed.dataHashes.length} hash(es)`);
    console.log(`   ID: ${parsed.id}`);
    console.log();

    // Parse ETH balance from the content
    const content = rawData.private_data?.plain_json_response?.[0]?.content || "";
    if (content) {
      const ethBalance = parseEthBalance(content);
      console.log("💰 Business Data:");
      console.log(`   ETH Balance: ${(ethBalance / 1000000).toFixed(6)} ETH`);
      console.log(`   Threshold: ${(THRESHOLD / 1000000).toFixed(6)} ETH`);
      console.log();

      if (ethBalance > THRESHOLD) {
        console.log("   ✅ Balance EXCEEDS threshold - verification would PASS");
      } else {
        console.log("   ❌ Balance BELOW threshold - verification would FAIL");
      }
    }

    console.log();
    console.log("=".repeat(70));
    console.log("✅ Parsing Test Complete!");
    console.log("=".repeat(70));
    console.log();
    console.log("Next steps for full verification:");
    console.log("1. Install Aztec toolchain: https://docs.aztec.network/");
    console.log("2. Compile Noir library: cd att_verifier_lib && nargo compile");
    console.log("3. Compile contract: cd contracts/binance_verifier && aztec compile");
    console.log("4. Start local network: aztec start --local-network");
    console.log("5. Deploy and verify using the generated contract bindings");

  } catch (error) {
    console.error("   ❌ Parsing failed:", error);
    process.exit(1);
  }
}

main().catch(console.error);