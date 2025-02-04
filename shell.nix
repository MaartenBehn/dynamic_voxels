with builtins;

{
  pkgs ? (
    import <nixpkgs> {
      config.allowUnfree = true;
    }
  ),
  pkgs-fix ? (
    import (getFlake "github:mcwitt/nixpkgs/fix/nsight_systems") {
      config.allowUnfree = true;
      config.cudaSupport = true;
    }
  ),
  ...
}:

let
  my-nsight_compute = pkgs-fix.cudaPackages.nsight_compute.overrideAttrs (oldAttrs: {
    buildInputs = oldAttrs.buildInputs ++ [
      pkgs-fix.rdma-core
      pkgs-fix.ucx
      pkgs-fix.e2fsprogs
      pkgs-fix.gst_all_1.gst-plugins-base
      pkgs-fix.qt6.qtwayland
    ];
  });
in
pkgs.mkShell rec {

  name = "dynamic_voxels";
  RUSTC_VERSION = "stable";
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    export RUSTUP_TOOLCHAIN=$RUSTC_VERSION-x86_64-unknown-linux-gnu
  '';

  packages = with pkgs; [

    rustup
    clang
    pkg-config
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    glslang
    linuxPackages_latest.perf
    hotspot
    cmake
    fontconfig
    vulkan-tools
    graphviz.out
    watchexec
    renderdoc
    python3

    pkgs-fix.cudaPackages.nsight_systems
  ];

  LD_LIBRARY_PATH =
    with pkgs;
    lib.makeLibraryPath [
      # load external libraries that you need in your rust project here
      libxkbcommon
      wayland-scanner.out
    ];

  # Add precompiled library to rustc search path
  RUSTFLAGS = (
    builtins.map (a: ''-L ${a}/lib'') [
      # add libraries here (e.g. pkgs.libvmi)
      pkgs.vulkan-headers
      pkgs.vulkan-loader
      pkgs.vulkan-validation-layers

    ]
  );

  VULKAN_SDK = "${pkgs.vulkan-headers}";
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
}
