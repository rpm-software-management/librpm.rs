#!/bin/bash

export RUST_BACKTRACE=1
export RUSTFLAGS="-D warnings"

cargo fmt --all -- --check

cargo build --release
cargo build --tests
cargo doc --no-deps

