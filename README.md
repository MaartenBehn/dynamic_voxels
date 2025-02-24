

### Run
```shell
cargo clean -p dynamic_voxels && cargo run
```

### Force shader rebuild
```shell
watchexec -e glsl,comp "rm -rf target/debug/.fingerprint/dynamic_voxels*"
```

### Hot reload
```shell
watchexec -e rs,glsl,comp "cargo clean -p dynamic_voxels && cargo build --lib"
```

```shell 
nix shell --impure --expr 'with builtins; with import (getFlake github:mcwitt/nixpkgs/fix/nsight_systems) { config = { allowUnfree = true; cudaSupport = true; }; }; cudaPackages.nsight_systems' --command 'nsys-ui'
nix shell --impure --expr 'with builtins; with import (getFlake github:mcwitt/nixpkgs/fix/nsight_systems) { config = { allowUnfree = true; cudaSupport = true; }; }; cudaPackages_11.nsight_compute' --command 'ncu-ui'
```

## Problems
Pipeline barriers and Wait events with no Workload between
