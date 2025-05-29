{ mkEnv, lib, ... } :{
  mkCrate = cross@{ craneLib, buildPackageAttrs, file_filter, pkgsCross, ... }:
    let 
      env = mkEnv cross;
    in craneLib.buildPackage (buildPackageAttrs // env // rec {
      src = lib.cleanSourceWith {
        src = ./..; # The original, unfiltered source
        filter = file_filter;
        name = "source"; # Be reproducible, regardless of the directory name
      };
      strictDeps = true;
      doCheck = false;
      cargoExtraArgs = "--features ${env.RUST_FEATURES} --debug"; 

      # Required because ring crate is special. This also seems to have
      # fixed some issues with the x86_64-windows cross-compile :shrug:
      TARGET_CC = "${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc";

      CARGO_BUILD_RUSTFLAGS = [
        # Static link cann ot be used with vulkan 
        #"-C" "target-feature=+crt-static"
        #"-C" "link-arg=-static"

        # -latomic is required to build openssl-sys for armv6l-linux, but
        # it doesn't seem to hurt any other builds.
        #"-C" "link-arg=-latomic"

        # https://github.com/rust-lang/cargo/issues/4133
        "-C" "linker=${TARGET_CC}"

      ] ++ (builtins.map (a: '' -L native=${a}/lib'') [
          # Needed in the final link step
          pkgsCross.vulkan-loader
        ]);
    });
}
