#!/bin/bash

set -euo pipefail

export CARGO_TARGET_DIR=$PWD/target_ci
export RUSTFLAGS=-Dwarnings
export DEFMT_LOG=trace

find . -name '*.rs' -not -path '*target*' | xargs rustfmt --check --edition 2024

cargo build --release --manifest-path rnfc/Cargo.toml --features ''
cargo build --release --manifest-path rnfc/Cargo.toml --features 'defmt'
cargo build --release --manifest-path rnfc/Cargo.toml --features 'log'
RUST_LOG=trace cargo test --release --manifest-path rnfc/Cargo.toml --features 'log'

cargo build --release --manifest-path rnfc-fm175xx/Cargo.toml --features ''
cargo build --release --manifest-path rnfc-fm175xx/Cargo.toml --features 'defmt'
cargo build --release --manifest-path rnfc-fm175xx/Cargo.toml --features 'log'

cargo build --release --manifest-path rnfc-st25r39/Cargo.toml --features ''
cargo build --release --manifest-path rnfc-st25r39/Cargo.toml --features 'defmt'
cargo build --release --manifest-path rnfc-st25r39/Cargo.toml --features 'log'

cargo build --release --manifest-path rnfc-acr122u/Cargo.toml --features ''

cargo build --release --manifest-path examples/st25r39-disco/Cargo.toml --target thumbv7em-none-eabi
cargo build --release --manifest-path examples/fm175xx/Cargo.toml --target thumbv7em-none-eabi
