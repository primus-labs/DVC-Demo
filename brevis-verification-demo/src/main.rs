//! Brevis Pico zkVM 验证示例
//!
//! 演示如何使用 Pico SDK 进行 proof 生成和验证
//!
//! # 运行方式
//!
//! ```bash
//! # 1. 生成 proof 并验证（需要先编译 ELF）
//! cargo run --release -- generate
//!
//! # 2. 查看 proof 信息
//! cargo run --release -- info
//!
//! # 3. 重新加载并验证 proof
//! cargo run --release -- verify
//! ```
//!
//! # 前置条件
//!
//! 1. 安装 Pico 工具链: https://pico-docs.brevis.network/getting-started/installation
//! 2. 编译 zkVM 程序: cd ../dvc-brevis-program && cargo pico build

use anyhow::{Context, Result};
use pico_sdk::{client::KoalaBearProverClient, init_logger};
use pico_vm::{
    machine::proof::MetaProof,
    configs::stark_config::KoalaBearPoseidon2,
    instances::configs::embed_kb_bn254_poseidon2::KoalaBearBn254Poseidon2,
};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use std::fs;
use std::path::Path;

// ============================================================================
// 数据结构
// ============================================================================

/// Proof 存储格式
#[derive(Debug, Serialize, Deserialize)]
struct ProofFixture {
    /// RISC-V Proof 数据
    riscv_proof: String,
    /// Embed Proof 数据
    embed_proof: String,
    /// 输入数据（用于验证）
    input_data: String,
}

// ============================================================================
// 命令参数
// ============================================================================

#[derive(Debug, StructOpt)]
#[structopt(name = "brevis-verification-demo", about = "Brevis Pico zkVM 验证示例")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// 生成 Proof 并验证
    Generate {
        /// ELF 文件路径
        #[structopt(short, long, default_value = "../dvc-brevis-program/elf/riscv32im-pico-zkvm-elf")]
        elf: String,

        /// 输入数据路径
        #[structopt(short, long, default_value = "../testdata/binance-attestation.json")]
        input: String,

        /// 输出目录
        #[structopt(short, long, default_value = "./output")]
        output_dir: String,
    },

    /// 查看 Proof 信息
    Info {
        /// Proof 文件路径
        #[structopt(short, long, default_value = "./output/proof_fixture.json")]
        proof: String,
    },

    /// 重新加载并验证 Proof
    Verify {
        /// ELF 文件路径
        #[structopt(short, long, default_value = "../dvc-brevis-program/elf/riscv32im-pico-zkvm-elf")]
        elf: String,

        /// Proof 文件路径
        #[structopt(short, long, default_value = "./output/proof_fixture.json")]
        proof: String,
    },
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> Result<()> {
    init_logger();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Generate { elf, input, output_dir } => {
            generate_and_verify_proof(&elf, &input, &output_dir)?;
        }
        Command::Info { proof } => {
            show_proof_info(&proof)?;
        }
        Command::Verify { elf, proof } => {
            load_and_verify_proof(&elf, &proof)?;
        }
    }

    Ok(())
}

// ============================================================================
// 生成并验证 Proof
// ============================================================================

fn generate_and_verify_proof(elf_path: &str, input_path: &str, output_dir: &str) -> Result<()> {
    println!("==========================================");
    println!("🔧 Brevis Pico Proof 生成");
    println!("==========================================");
    println!();

    let elf_path = Path::new(elf_path);
    let input_path = Path::new(input_path);
    let output_dir = Path::new(output_dir);

    // 检查文件
    if !elf_path.exists() {
        println!("❌ ELF 文件不存在: {:?}", elf_path);
        println!();
        println!("请先编译程序:");
        println!("  cd ../dvc-brevis-program");
        println!("  cargo pico build");
        println!();
        println!("ELF 输出路径: elf/riscv32im-pico-zkvm-elf");
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
    let input_data = fs::read_to_string(input_path)?;
    println!("   输入大小: {} bytes", input_data.len());
    println!();

    // 创建 prover client
    println!("🔄 初始化 Prover Client...");
    let client = KoalaBearProverClient::new(&elf);
    println!("   ✅ Client 创建成功");
    println!();

    // 创建 stdin
    println!("📝 准备输入数据...");
    let mut stdin_builder = client.new_stdin_builder();
    let bytes = bincode::serialize(&input_data)
        .context("序列化输入数据失败")?;
    stdin_builder.write_slice(&bytes);
    println!("   输入数据大小: {} bytes", bytes.len());
    println!();

    // 生成 proof
    println!("🔄 生成 Proof...");
    println!("   注意: 这可能需要几分钟时间...");

    let (riscv_proof, embed_proof) = client.prove(stdin_builder)
        .context("生成 proof 失败")?;

    println!("   ✅ Proof 生成成功");
    println!();

    // ========================================
    // 本地验证 (关键步骤)
    // ========================================
    println!("==========================================");
    println!("🔐 本地验证 Proof");
    println!("==========================================");
    println!();

    println!("🔄 验证 RISC-V Proof 和 Embed Proof...");

    // 使用 client.verify() 进行完整验证
    client.verify(&(riscv_proof.clone(), embed_proof.clone()))
        .context("Proof 验证失败")?;

    println!("   ✅ RISC-V Proof 验证通过");
    println!("   ✅ Embed Proof 验证通过");
    println!();

    println!("==========================================");
    println!("✅✅✅ 本地验证成功！✅✅✅");
    println!("==========================================");
    println!();

    // 保存 proof 到自定义 JSON 格式
    println!("💾 保存 Proof...");

    let riscv_proof_hex = hex::encode(bincode::serialize(&riscv_proof)?);
    let embed_proof_hex = hex::encode(bincode::serialize(&embed_proof)?);

    let fixture = ProofFixture {
        riscv_proof: format!("0x{}", riscv_proof_hex),
        embed_proof: format!("0x{}", embed_proof_hex),
        input_data: input_data.clone(),
    };

    // 保存自定义格式到单独文件（避免被 write_onchain_data 覆盖）
    let json = serde_json::to_string_pretty(&fixture)?;
    let fixture_path = output_dir.join("proof_fixture.json");
    fs::write(&fixture_path, json)?;
    println!("   ✅ 已保存: {:?}", fixture_path);

    // 使用官方格式保存（用于链上验证）
    println!();
    println!("💾 保存链上验证格式...");
    client.write_onchain_data(output_dir, &riscv_proof, &embed_proof)
        .context("保存链上数据失败")?;
    println!("   ✅ 已保存到: {:?}", output_dir);

    // 输出摘要
    println!();
    println!("==========================================");
    println!("📊 摘要");
    println!("==========================================");
    println!("   RISC-V Proof: {} bytes", riscv_proof_hex.len() / 2);
    println!("   Embed Proof: {} bytes", embed_proof_hex.len() / 2);
    println!("   输入数据: {} bytes", input_data.len());
    println!();

    // 解析输入数据中的关键信息
    parse_input_data(&input_data)?;

    Ok(())
}

// ============================================================================
// 查看 Proof 信息
// ============================================================================

fn show_proof_info(proof_path: &str) -> Result<()> {
    println!("==========================================");
    println!("📋 Brevis Pico Proof 信息");
    println!("==========================================");
    println!();

    let proof_path = Path::new(proof_path);

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

    println!("   RISC-V Proof: {}...", &fixture.riscv_proof[..34]);
    println!("   Embed Proof: {}...", &fixture.embed_proof[..34]);
    println!("   输入数据: {} bytes", fixture.input_data.len());
    println!();

    // 解码数据
    println!("🔧 解码数据...");
    let riscv_hex = fixture.riscv_proof.strip_prefix("0x").unwrap_or(&fixture.riscv_proof);
    let embed_hex = fixture.embed_proof.strip_prefix("0x").unwrap_or(&fixture.embed_proof);

    let riscv_bytes = hex::decode(riscv_hex)?;
    let embed_bytes = hex::decode(embed_hex)?;

    println!("   RISC-V bytes: {}", riscv_bytes.len());
    println!("   Embed bytes: {}", embed_bytes.len());
    println!();

    // 显示验证信息
    println!("==========================================");
    println!("📋 Proof 结构说明");
    println!("==========================================");
    println!();

    println!("Brevis zkVM 生成的 Proof 包含两部分:");
    println!();
    println!("  1. RISC-V Proof (STARK 证明)");
    println!("     - 证明程序在 zkVM 中正确执行");
    println!("     - 包含执行轨迹和状态转换证明");
    println!("     - 大小: ~7.3 MB");
    println!();
    println!("  2. Embed Proof (嵌入证明)");
    println!("     - 证明输入/输出数据的正确性");
    println!("     - 用于链上数据验证");
    println!("     - 大小: ~300 KB");
    println!();

    println!("==========================================");
    println!("📋 验证方式");
    println!("==========================================");
    println!();
    println!("  本地验证:");
    println!("    cargo run --release -- verify");
    println!();
    println!("  链上验证:");
    println!("    - 使用 output/ 目录下的文件");
    println!("    - 需要部署 Brevis 验证合约");
    println!("    - 参考: https://pico-docs.brevis.network");
    println!();

    // 解析输入数据中的关键信息
    parse_input_data(&fixture.input_data)?;

    Ok(())
}

// ============================================================================
// 重新加载并验证 Proof
// ============================================================================

fn load_and_verify_proof(elf_path: &str, proof_path: &str) -> Result<()> {
    println!("==========================================");
    println!("🔐 Brevis Pico Proof 重新验证");
    println!("==========================================");
    println!();

    let elf_path = Path::new(elf_path);
    let proof_path = Path::new(proof_path);

    // 检查文件
    if !elf_path.exists() {
        println!("❌ ELF 文件不存在: {:?}", elf_path);
        println!();
        println!("请先编译程序:");
        println!("  cd ../dvc-brevis-program");
        println!("  cargo pico build");
        return Ok(());
    }

    if !proof_path.exists() {
        println!("❌ Proof 文件不存在: {:?}", proof_path);
        println!();
        println!("请先运行: cargo run --release -- generate");
        return Ok(());
    }

    // 加载 ELF
    println!("📦 加载 ELF 文件...");
    let elf = fs::read(elf_path)?;
    println!("   ELF 大小: {} bytes", elf.len());
    println!();

    // 加载 proof
    println!("📂 加载 Proof...");
    let json = fs::read_to_string(proof_path)?;
    let fixture: ProofFixture = serde_json::from_str(&json)?;

    let riscv_hex = fixture.riscv_proof.strip_prefix("0x").unwrap_or(&fixture.riscv_proof);
    let embed_hex = fixture.embed_proof.strip_prefix("0x").unwrap_or(&fixture.embed_proof);

    let riscv_bytes = hex::decode(riscv_hex)?;
    let embed_bytes = hex::decode(embed_hex)?;

    println!("   RISC-V Proof: {} bytes", riscv_bytes.len());
    println!("   Embed Proof: {} bytes", embed_bytes.len());
    println!();

    // 创建 prover client
    println!("🔄 初始化 Prover Client...");
    let client = KoalaBearProverClient::new(&elf);
    println!("   ✅ Client 创建成功");
    println!();

    // 反序列化 proof
    println!("🔧 反序列化 Proof...");

    // MetaProof 需要正确的 StarkGenericConfig 类型
    // KoalaBearProverClient 使用:
    //   - KoalaBearPoseidon2 用于 RISC-V proof
    //   - KoalaBearBn254Poseidon2 用于 Embed proof

    let riscv_proof: MetaProof<KoalaBearPoseidon2> = bincode::deserialize(&riscv_bytes)
        .context("RISC-V Proof 反序列化失败")?;
    println!("   ✅ RISC-V Proof 反序列化成功");

    let embed_proof: MetaProof<KoalaBearBn254Poseidon2> = bincode::deserialize(&embed_bytes)
        .context("Embed Proof 反序列化失败")?;
    println!("   ✅ Embed Proof 反序列化成功");
    println!();

    // 验证
    println!("🔐 开始验证...");
    println!();

    // 使用 client.verify() 进行完整验证
    client.verify(&(riscv_proof, embed_proof))
        .context("Proof 验证失败")?;

    println!("==========================================");
    println!("✅✅✅ 验证成功！✅✅✅");
    println!("==========================================");
    println!();
    println!("这是一个有效的 Brevis zkVM Proof");
    println!();

    // 解析输入数据中的关键信息
    parse_input_data(&fixture.input_data)?;

    Ok(())
}

// ============================================================================
// 解析输入数据
// ============================================================================

fn parse_input_data(input: &str) -> Result<()> {
    println!("📋 输入数据解析:");
    println!("----------------------------------------");

    let v: serde_json::Value = serde_json::from_str(input)?;

    // 提取 attestor 地址
    if let Some(attestor) = v.get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("attestor"))
        .and_then(|a| a.as_str())
    {
        println!("   Attestor: {}", attestor);
    }

    // 提取 URL
    if let Some(url) = v.get("public_data")
        .and_then(|pd| pd.get(0))
        .and_then(|item| item.get("attestation"))
        .and_then(|att| att.get("request"))
        .and_then(|req| req.get(0))
        .and_then(|r| r.get("url"))
        .and_then(|u| u.as_str())
    {
        println!("   URL: {}", url);
    }

    println!();
    Ok(())
}