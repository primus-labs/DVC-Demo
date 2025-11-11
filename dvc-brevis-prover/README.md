# DVC-Brevis-Prover


## Prerequisite

1. Install [Pico toolchains](https://pico-docs.brevis.network/getting-started/installation).

## Build

```sh
RUST_LOG=info cargo build --release
```

## Run

```sh
RUST_LOG=info cargo run --release
```

## Usage

```sh
./target/release/dvc-brevis-prover --help
dvc-brevis-prover 0.1.0
DVC-Brevis-Prover

USAGE:
    dvc-brevis-prover [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --elf <elf>                   [default: ../dvc-brevis-program/elf/riscv32im-pico-zkvm-elf]
    -i, --input <input>               [default: ../testdata/hash_comparsion.json]
    -o, --output-dir <output-dir>     [default: ./proof_output]
```
