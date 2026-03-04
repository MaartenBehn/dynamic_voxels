#!/bin/sh

# exit when one command fails
set -e

# compile
cargo build --features $RUST_FEATURES --profile dev-fast 

# rund in background
./target/x86_64-unknown-linux-gnu/dev-fast/dynamic_voxels &

# get pid of background process
PID=$!

# wait a bit and move it to other screen
sleep 2 && hyprctl dispatch -- movetoworkspacesilent 4

# set trap to kill all background processes when the script is killed
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

# wait for background process
wait $PID

