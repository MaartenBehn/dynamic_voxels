#!/bin/sh

cd ./scripts || cd ../scripts || true
cd ..

cargo build --features $RUST_FEATURES --release
perf record --call-graph dwarf,65528 ./target/x86_64-unknown-linux-gnu/release/dynamic_voxels
#perf record ./target/x86_64-unknown-linux-gnu/release/dynamic_voxels

hotspot ./perf.data
