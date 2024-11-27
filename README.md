


### Run
```shell
cargo clean -p dynamic_voxels && cargo run
```

### Hot reload
```shell
watchexec -e rs,glsl,comp "cargo clean -p dynamic_voxels && cargo build --lib"
```
