{ pkgs ? (import <nixpkgs> {
    config.allowUnfree = true;
}), ... }:

let my-nsight_compute = pkgs.cudaPackages.nsight_compute.overrideAttrs(
    oldAttrs: {
        buildInputs = oldAttrs.buildInputs ++ [
            pkgs.rdma-core
            pkgs.ucx
            pkgs.e2fsprogs
            pkgs.kdePackages.qtwayland
            ];
        }
    );
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

    my-nsight_compute
  ];

  buildInputs = with pkgs; [
    boost
  ];

  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
   # load external libraries that you need in your rust project here
   libxkbcommon
   wayland-scanner.out
  ];

  # Add precompiled library to rustc search path
  RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
    # add libraries here (e.g. pkgs.libvmi)
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-validation-layers

  ]);

  VULKAN_SDK = "${pkgs.vulkan-headers}";
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
}
