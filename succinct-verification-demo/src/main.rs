//! SP1 zkVM 链下验证示例
//!
//! 演示如何使用 SP1 SDK 进行链下 proof 验证
//!
//! # 运行方式
//!
//! ```bash
//! # 1. 生成 proof（需要先编译 ELF）
//! cargo run --release -- generate
//!
//! # 2. 验证 proof
//! cargo run --release -- verify
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sp1_sdk::{HashableKey, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use std::fs;
use std::path::Path;

// ============================================================================
// 数据结构
// ============================================================================

/// Proof 存储格式
#[derive(Debug, Serialize, Deserialize)]
struct ProofFixture {
    /// 验证密钥（序列化）
    vk: String,
    /// Proof 数据（序列化）
    proof: String,
    /// 公开输出
    public_values: String,
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> Result<()> {
    // 初始化日志
    sp1_sdk::utils::setup_logger();

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "generate" => generate_proof()?,
        "verify" => verify_proof()?,
        "help" => print_help(),
        _ => {
            println!("未知命令: {}", command);
            print_help();
        }
    }

    Ok(())
}

fn print_help() {
    println!("SP1 zkVM 链下验证示例");
    println!();
    println!("用法:");
    println!("  cargo run --release -- generate   生成 proof");
    println!("  cargo run --release -- verify     验证 proof");
    println!();
    println!("注意: generate 需要先有 ELF 文件和输入数据");
}

// ============================================================================
// 生成 Proof
// ============================================================================

fn generate_proof() -> Result<()> {
    println!("==========================================");
    println!("🔧 SP1 Proof 生成");
    println!("==========================================");
    println!();

    // 检查 ELF 文件
    let elf_path = Path::new("../dvc-succinct-program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/dvc-succinct-program");
    let input_path = Path::new("../testdata/binance-attestation.json");
    let output_dir = Path::new("./output");

    if !elf_path.exists() {
        println!("❌ ELF 文件不存在: {:?}", elf_path);
        println!();
        println!("请先编译程序:");
        println!("  cd ../dvc-succinct-program");
        println!("  cargo prove build");
        return Ok(());
    }

    if !input_path.exists() {
        println!("❌ 输入文件不存在: {:?}", input_path);
        return Ok(());
    }

    // 创建输出目录
    fs::create_dir_all(output_dir)?;

    // 加载 ELF
    println!("📦 加载 ELF 文件...");
    let elf = fs::read(elf_path)?;
    println!("   ELF 大小: {} bytes", elf.len());

    // 加载输入
    println!("📦 加载输入数据...");
    let input = fs::read_to_string(input_path)?;
    println!("   输入大小: {} bytes", input.len());

    // 创建 stdin
    let mut stdin = SP1Stdin::new();
    stdin.write(&input);

    // 创建 prover client
    println!();
    println!("🔄 生成 Proof（本地模式）...");
    let client = ProverClient::from_env();

    // Setup
    let (pk, vk) = client.setup(&elf);

    // 生成 proof（使用 mock 模式快速生成）
    let proof = client.prove(&pk, &stdin).run()
        .context("生成 proof 失败")?;

    println!("   ✅ Proof 生成成功");

    // 验证 proof
    println!();
    println!("🔐 验证 Proof...");
    client.verify(&proof, &vk)
        .context("验证 proof 失败")?;
    println!("   ✅ 验证通过");

    // 保存 proof
    println!();
    println!("💾 保存 Proof...");

    let vk_bytes = bincode::serialize(&vk)?;
    let proof_bytes = bincode::serialize(&proof)?;

    let fixture = ProofFixture {
        vk: format!("0x{}", hex::encode(&vk_bytes)),
        proof: format!("0x{}", hex::encode(&proof_bytes)),
        public_values: proof.public_values.raw(),
    };

    let json = serde_json::to_string_pretty(&fixture)?;
    let output_path = output_dir.join("proof.json");
    fs::write(&output_path, json)?;

    println!("   ✅ 已保存: {:?}", output_path);

    // 输出摘要
    println!();
    println!("==========================================");
    println!("📊 摘要");
    println!("==========================================");
    println!("   VK 大小: {} bytes", vk_bytes.len());
    println!("   Proof 大小: {} bytes", proof_bytes.len());
    println!("   Public Values: {} bytes", (fixture.public_values.len() - 2) / 2);
    println!();

    Ok(())
}

// ============================================================================
// 验证 Proof
// ============================================================================

fn verify_proof() -> Result<()> {
    println!("==========================================");
    println!("🔐 SP1 Proof 链下验证");
    println!("==========================================");
    println!();

    let proof_path = Path::new("./output/proof.json");

    if !proof_path.exists() {
        println!("❌ Proof 文件不存在: {:?}", proof_path);
        println!();
        println!("请先运行: cargo run --release -- generate");
        return Ok(());
    }

    // 加载 proof
    println!("📂 加载 Proof...");
    let json = fs::read_to_string(proof_path)?;
    let fixture: ProofFixture = serde_json::from_str(&json)?;

    println!("   VK: {}...", &fixture.vk[..34]);
    println!("   Proof: {}...", &fixture.proof[..34]);
    println!("   Public Values: {} bytes", (fixture.public_values.len() - 2) / 2);
    println!();

    // 解码数据
    println!("🔧 解码数据...");
    let vk_hex = fixture.vk.strip_prefix("0x").unwrap_or(&fixture.vk);
    let proof_hex = fixture.proof.strip_prefix("0x").unwrap_or(&fixture.proof);

    let vk_bytes = hex::decode(vk_hex)?;
    let proof_bytes = hex::decode(proof_hex)?;

    println!("   VK bytes: {}", vk_bytes.len());
    println!("   Proof bytes: {}", proof_bytes.len());
    println!();

    // 反序列化
    println!("🔧 反序列化...");
    let vk: SP1VerifyingKey = bincode::deserialize(&vk_bytes)
        .context("VK 反序列化失败")?;
    println!("   ✅ VK OK");

    let proof: SP1ProofWithPublicValues = bincode::deserialize(&proof_bytes)
        .context("Proof 反序列化失败")?;
    println!("   ✅ Proof OK");
    println!();

    // 验证
    println!("🔐 开始验证...");
    let client = ProverClient::from_env();

    match client.verify(&proof, &vk) {
        Ok(_) => {
            println!();
            println!("==========================================");
            println!("✅✅✅ 验证成功！✅✅✅");
            println!("==========================================");
            println!();
            println!("这是一个有效的 SP1 zkVM Proof");
            println!();

            // 解析 public values
            parse_public_values(&fixture.public_values)?;
        }
        Err(e) => {
            println!();
            println!("==========================================");
            println!("❌ 验证失败");
            println!("==========================================");
            println!();
            println!("错误: {}", e);
        }
    }

    Ok(())
}

// ============================================================================
// 解析 Public Values
// ============================================================================

fn parse_public_values(hex_str: &str) -> Result<()> {
    println!("📋 Public Values 解析:");
    println!("----------------------------------------");

    let hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(hex)?;

    // 尝试提取可读文本
    let mut readable = Vec::new();
    let mut current = String::new();

    for &b in &bytes {
        if b >= 32 && b <= 126 {
            current.push(b as char);
        } else if !current.is_empty() {
            if current.len() > 3 {
                readable.push(current.clone());
            }
            current.clear();
        }
    }
    if current.len() > 3 {
        readable.push(current);
    }

    for (i, s) in readable.iter().enumerate() {
        println!("   {}: {}", i + 1, s);
    }

    println!();
    Ok(())
}