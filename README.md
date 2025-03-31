

### Run
```shell
cargo run
```

### Hot reload lib
```shell
watchexec -e rs,glsl,comp "cargo build --profile dev-fast --lib"
```

```shell 
nix shell --impure --expr 'with builtins; with import (getFlake github:mcwitt/nixpkgs/fix/nsight_systems) { config = { allowUnfree = true; cudaSupport = true; }; }; cudaPackages.nsight_systems' --command 'nsys-ui'
nix shell --impure --expr 'with builtins; with import (getFlake github:mcwitt/nixpkgs/fix/nsight_systems) { config = { allowUnfree = true; cudaSupport = true; }; }; cudaPackages_11.nsight_compute' --command 'ncu-ui'
```

## Problems
Pipeline barriers and Wait events with no Workload between
