


### Run
```shell
cargo clean -p dynamic_voxels && cargo run
```

### Force shader rebuild
```shell
watchexec -e glsl,comp "rm -rf target/debug/.fingerprint/dynamic_voxels* && rm -rf target/release/.fingerprint/dynamic_voxels*"
```

### Hot reload
```shell
watchexec -e rs,glsl,comp "cargo clean -p dynamic_voxels && cargo build --lib"
```
