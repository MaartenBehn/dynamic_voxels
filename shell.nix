with builtins;

{
  pkgs ? (
    import <nixpkgs> {
      config.allowUnfree = true;
    }
  ),
    #pkgs-fix ? (
  #import (getFlake "/home/stroby/dev/nixpkgs") {
  #    config.allowUnfree = true;
#    config.cudaSupport = true;
#        }
#    ),
  pkgs-mcwitt ? (
    import (getFlake "github:mcwitt/nixpkgs/fix/nsight_systems") {
      config.allowUnfree = true;
      config.cudaSupport = true;
    }
  ),

  ...
}:
let
  my-nsight_compute = pkgs.cudaPackages.nsight_compute.overrideAttrs(
    oldAttrs: {
        buildInputs = oldAttrs.buildInputs ++ [
            pkgs.rdma-core
            pkgs.ucx
            pkgs.e2fsprogs
            pkgs.kdePackages.qtwayland
            ];
        }
    );
  rustc_version = "stable";
in pkgs.mkShell {

  name = "dynamic_voxels";
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$${rustc_version}-x86_64-unknown-linux-gnu/bin/
  '';

  RUSTC_VERSION = rustc_version;
  RUSTUP_TOOLCHAIN="${rustc_version}-x86_64-unknown-linux-gnu";
    #CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS = "[\"-C\", \"link-arg=--ld-path=${pkgs.mold}/bin/mold\"]";

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
    watchexec
    renderdoc
    python3
    graphviz

    pkgs-mcwitt.cudaPackages.nsight_systems
      #pkgs-mcwitt.cudaPackages.nsight_compute
    my-nsight_compute 
  ];

  # Use fast liker 
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.clang}/bin/clang";
  
  RUSTFLAGS = 
      # Use faster linker     
      [''-C link-arg=-fuse-ld=${pkgs.mold-wrapped}/bin/mold''] ++
      # Add precompiled library to rustc search path
      (builtins.map (a: '' -L ${a}/lib'') [
      # add libraries here (e.g. pkgs.libvmi)
      pkgs.vulkan-headers
      pkgs.vulkan-loader
      pkgs.vulkan-validation-layers
    ]);

  LD_LIBRARY_PATH =with pkgs; lib.makeLibraryPath [
      # load external libraries that you need in your rust project
      libxkbcommon
      wayland-scanner.out
      libGL
      wayland
      vulkan-headers 
      vulkan-loader
      vulkan-validation-layers
  ];

  VULKAN_SDK = "${pkgs.vulkan-headers}";
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
  SHADERC_LIB_DIR = pkgs.lib.makeLibraryPath [ "${pkgs.shaderc.lib}" ];
}
