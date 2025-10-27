#!/bin/sh

cd ./scripts || cd ../scripts || true
cd ..

cargo build --features $RUST_FEATURES --profile release
perf record --call-graph dwarf ./target/x86_64-unknown-linux-gnu/release/dynamic_voxels

hotspot ./perf.data
