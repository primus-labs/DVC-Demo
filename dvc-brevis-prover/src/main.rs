use pico_sdk::{client::KoalaBearProverClient, init_logger};
use std::fs;
use structopt::StructOpt;

#[allow(unused_imports)]
use hex;

#[derive(Debug, StructOpt)]
#[structopt(name = "dvc-brevis-prover", about = "DVC-Brevis-Prover")]
struct Opt {
    #[structopt(short, long, default_value = "../dvc-brevis-program/elf/riscv32im-pico-zkvm-elf")]
    elf: String,

    #[structopt(short, long, default_value = "../testdata/hash_comparsion.json")]
    input: String,

    #[structopt(short, long, default_value = "./proof_output")]
    output_dir: String,
}

fn main() {
    let opt = Opt::from_args();

    // Initialize logger
    init_logger();

    // Load the ELF file
    let elf = load_elf(&opt.elf);
    // println!("elf length: {}", elf.len());

    // Initialize a client.
    let client = KoalaBearProverClient::new(&elf);

    // Initialize new stdin
    let mut stdin_builder = client.new_stdin_builder();

    // Input
    let attestation_data = fs::read_to_string(opt.input).unwrap();
    let bytes = bincode::serialize(&attestation_data).expect("failed to serialize data");
    // println!("data len: {} 0x{}", bytes.len(), hex::encode(&bytes));
    stdin_builder.write_slice(&bytes);

    // Generate proof
    let (riscv_proof, embed_proof) = client.prove(stdin_builder).expect("Failed to generate proof");

    // Save the proof
    let _ = fs::create_dir_all(&opt.output_dir).unwrap();
    client
        .write_onchain_data(&opt.output_dir, &riscv_proof, &embed_proof)
        .unwrap();
}

/// Loads an ELF file from the specified path.
pub fn load_elf(path: &str) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|err| {
        panic!("Failed to load ELF file from {}: {}", path, err);
    })
}
