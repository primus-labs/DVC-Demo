# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a DVC (Data Verification and Computation) Demo that uses zkTLS and zkVM for privacy-preserving data verification. The system proves facts about private data (e.g., proving a Binance ETH balance exceeds a threshold) without revealing the actual data.

**Architecture:**
- zkTLS (via Primus Network) generates attestation data from TLS responses
- zkVM (Succinct SP1 or Brevis) executes Rust programs that verify attestations and apply business logic
- TEE service coordinates the prover execution

## Build Commands

**dvc-client (Node.js SDK):**
```sh
cd dvc-client && npm install
```

**dvc-service (Python FastAPI):**
```sh
cd dvc-service && pip install -r requirements.txt
```

**dvc-succinct-program (Rust zkVM program for SP1):**
```sh
cd dvc-succinct-program && RUST_LOG=info cargo prove build
```
ELF output: `target/elf-compilation/riscv32im-succinct-zkvm-elf/release/`

**dvc-succinct-prover (Rust prover for SP1):**
```sh
cd dvc-succinct-prover && cargo build --release
```

**dvc-brevis-program (Rust zkVM program for Brevis):**
```sh
cd dvc-brevis-program && RUST_LOG=info cargo pico build
```
ELF output: `elf/`

**dvc-brevis-prover (Rust prover for Brevis):**
```sh
cd dvc-brevis-prover && cargo build --release
```

## Run Commands

**dvc-service (TEE service):**
```sh
cd dvc-service && uvicorn main:app --host 0.0.0.0 --port 38080
# With SSL: uvicorn main:app --host 0.0.0.0 --port 38080 --ssl-keyfile certs/server.key --ssl-certfile certs/server.crt
```
API docs available at `http://localhost:38080/docs`

**dvc-succinct-prover (local execution):**
```sh
cd dvc-succinct-prover && cargo run --release -- --execute
# Or prove mode: cargo run --release -- --prove
```

## Prerequisites

- **SP1 (Succinct):** Install from https://docs.succinct.xyz/docs/sp1/getting-started/install
- **Pico (Brevis):** Install from https://pico-docs.brevis.network/getting-started/installation

## Environment Setup

Each component has a `.env.example` file. Copy to `.env` and configure:

- **dvc-client:** Set `DVC_SERVICE_URL`, `BINANCE_API_KEY`, `BINANCE_API_SECRET`
- **dvc-service:** Set `SUCCINCT_PROVER_BIN` path, `MAX_CONCURRENCY`, `MAX_QUEUE_SIZE`

## Workflow

1. **Write program:** Create Rust program in `dvc-succinct-program/src/main.rs`
2. **Build ELF:** Run `cargo prove build`
3. **Upload program:** Use `dvc-client/demo/upload_program.js` → returns program ID
4. **Generate attestation:** Use `dvc-client/demo/demo_binance.js` with Primus Network ZKTLS
5. **Submit task:** Use `dvc-client/demo/submit_task.js` with program ID + attestation
6. **Get result:** Use `dvc-client/demo/get_result.js` with task ID

## Component Structure

```
dvc-client/        # Node.js SDK and demo scripts
  src/             # prover_client.js, zktls_client.js
  demo/            # Example scripts for each workflow step

dvc-service/       # Python FastAPI service (runs in TEE)
  main.py          # API endpoints: /uploadProgram, /submitTask, /getResult, /listTasks

dvc-succinct-program/  # Rust program for SP1 zkVM
  src/main.rs      # Business logic - verify attestation, check conditions

dvc-succinct-prover/   # Rust prover that executes SP1 programs (generates STARK proof)

dvc-brevis-program/    # Alternative zkVM (Brevis)

dvc-brevis-prover/     # Brevis prover

testdata/          # Sample attestation data (binance-attestation.json)
```

## zkVM Program Structure (Succinct)

The Rust program in `dvc-succinct-program/src/main.rs` follows this pattern:

```rust
#![no_main]
sp1_zkvm::entrypoint!(main);

fn verify_attestation() -> Result<...> {
    // 1. Read attestation data from input
    // 2. Create attestation config with attestor address and allowed URLs
    // 3. Verify signature, attestor, and data source
}

fn app_main() -> Result<()> {
    // 1. Call verify_attestation() - MANDATORY for all programs
    // 2. Extract business data from verified messages
    // 3. Apply business logic (e.g., check balance > threshold)
    // 4. Use sp1_zkvm::io::commit() to output public values
}
```

Key constraint: `BASE_URLS` constant must match the URLs used in ZKTLS attestation.