{ targets_configs, nixpkgs, crane, fenix, ... }:
let
  # eachSystem [system] (system: ...)
  #
  # Returns an attrset with a key for every system in the given array, with
  # the key's value being the result of calling the callback with that key.
  eachSystem = supportedSystems: callback: builtins.foldl'
    (overall: system: overall // { ${system} = callback system; })
    {}
    supportedSystems;

  # eachCrossSystem [system] (buildSystem: targetSystem: ...)
  #
  # Returns an attrset with a key "$buildSystem.cross-$targetSystem" for
  # every combination of the elements of the array of system strings. The
  # value of the attrs will be the result of calling the callback with each
  # combination.
  #
  # There will also be keys "$system.default", which are aliases of
  # "$system.cross-$system" for every system.
  #
  eachCrossSystem = supportedSystems: callback:
    eachSystem supportedSystems (buildSystem: builtins.foldl'
      (inner: targetSystem: inner // {
        "cross-${targetSystem}" = callback buildSystem targetSystem;
      })
      { default = callback buildSystem buildSystem; }
      supportedSystems
    );

  mkPkgs = buildSystem: targetSystem: import nixpkgs ({
    system = buildSystem;
  } // (if targetSystem == null then {} else {
      # The nixpkgs cache doesn't have any packages where cross-compiling has
      # been enabled, even if the target platform is actually the same as the
      # build platform (and therefore it's not really cross-compiling). So we
      # only set up the cross-compiling config if the target platform is
      # different.
      crossSystem.config = targets_configs.${targetSystem}.crossSystemConfig;
    }));

  mkConfig = buildSystem: { 
    pkgs = mkPkgs buildSystem null; 
    rustTarget = targets_configs.${buildSystem}.rustTarget;
  };

  mkCrossConfig = buildSystem: targetSystem: rec {
    pkgs = mkPkgs buildSystem null;
    pkgsCross = mkPkgs buildSystem targetSystem;
    rustTarget = targets_configs.${targetSystem}.rustTarget;

    fenixPkgs = fenix.packages.${buildSystem};
    mkToolchain = fenixPkgs: fenixPkgs.stable;

    toolchain = fenixPkgs.combine [
      (mkToolchain fenixPkgs).rustc
      (mkToolchain fenixPkgs).cargo
      (mkToolchain fenixPkgs.targets.${rustTarget}).rust-std
    ];

    buildPackageAttrs = if builtins.hasAttr "makeBuildPackageAttrs" targets_configs.${targetSystem} then
      targets_configs.${targetSystem}.makeBuildPackageAttrs pkgsCross
    else
      {};

    craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

    slang_filter = path: _type: builtins.match ".*slang$" path != null;
    slang_bin_filter = path: _type: builtins.match "bin$" path != null;
    glsl_filter = path: _type: builtins.match ".*glsl$" path != null;
    file_filter = path: type: (craneLib.filterCargoSources path type)
      || (slang_filter path type) 
      || (slang_bin_filter path type) 
      || (glsl_filter path type);
  };

in {
  inherit eachSystem;
  inherit eachCrossSystem;
  inherit mkConfig;  
  inherit mkCrossConfig;
}
