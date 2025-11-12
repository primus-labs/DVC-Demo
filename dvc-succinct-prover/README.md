# DVC-Succinct-Prover

## Prerequisite

1. Install [SP1](https://docs.succinct.xyz/docs/sp1/getting-started/install).


## Build

```sh
RUST_LOG=info cargo build --release
```

## Run

```sh
RUST_LOG=info cargo run --release -- --execute
```


## Usage

```sh
./target/release/dvc-succinct-prover --help
The arguments for the command

Usage: dvc-succinct-prover [OPTIONS]

Options:
      --execute                  
      --prove                    
      --elf <ELF>                [default: ../dvc-succinct-program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/binance-elf]
      --input <INPUT>            [default: ../testdata/binance-attestation.json]
      --output-dir <OUTPUT_DIR>  [default: ./proof_output]
  -h, --help                     Print help
  -V, --version                  Print version
```
