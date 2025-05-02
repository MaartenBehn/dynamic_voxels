
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

```
slangc ./slang_shaders/render.slang -profile glsl_450 -target spirv -o ./slang_shaders/bin/render.spv -entry compute_main -stage compute
```


## Problems
Pipeline barriers and Wait events with no Workload between
