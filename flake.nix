# https://mediocregopher.com/posts/x-compiling-rust-with-nix.gmi
# https://crane.dev/

{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    crane.url = "github:ipetkov/crane";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, crane, fenix }@c:
    let  
      config = builtins.foldl' 
        (c: a: c // (import a c) )         # extend config with configs from paths
        (c // { lib = nixpkgs.lib; }) # inital config 
        [
          ./nix/targets_config.nix
          ./nix/cross_compile_functions.nix
          ./nix/env.nix
          ./nix/cross_crate.nix
          ./nix/dev_shell.nix
        ];
      names = builtins.attrNames config.targets_configs;
      mkCrate = a: b: config.mkCrate (config.mkCrossConfig a b);
      mkShell = a: config.mkShell (config.mkConfig a);
    in {
      packages = config.eachCrossSystem names mkCrate;
      devShell = config.eachSystem names mkShell;    
    };
}
