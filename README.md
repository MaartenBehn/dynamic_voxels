


### Build
```shell
cargo clean -p dynamic_voxels && cargo build
```

### Hot reload
```shell
watchexec -e rs,glsl,comp "cargo clean -p dynamic_voxels && cargo build --lib"
```
