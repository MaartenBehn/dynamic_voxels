{ lib, ... } :{
  mkEnv = { pkgs, rustTarget, ...}: { 
    # For things need in things like build.rs 
    nativeBuildInputs = [
      pkgs.shader-slang
    ];

    # Needed for linking when compiling a crate 
    buildInputs = [
      pkgs.vulkan-loader
    ];

    SHADERC_LIB_DIR = lib.makeLibraryPath [ "${pkgs.shaderc.lib}" ];

    #fixes issues related to openssl
    OPENSSL_DIR = "${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include/";

    CARGO_BUILD_TARGET = rustTarget;
    RUST_FEATURES = "profile_dag"; 
  };
}
