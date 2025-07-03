#!/bin/sh

cd ./scripts || cd ../scripts || true
cd ..

export CARGO_PROFILE_DEV_OPT_LEVEL=3
cargo build --features $RUST_FEATURES
perf record --call-graph dwarf ./target/x86_64-unknown-linux-gnu/debug/dynamic_voxels

hotspot ./perf.data
