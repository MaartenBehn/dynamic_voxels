_ :{
  targets_configs = {
    "x86_64-linux" = {
      crossSystemConfig = "x86_64-unknown-linux-gnu";
      rustTarget = "x86_64-unknown-linux-gnu";
    };

    "i686-linux" = {
      crossSystemConfig = "i686-unknown-linux-gnu";
      rustTarget = "i686-unknown-linux-gnu";
    };

    "aarch64-linux" = {
      crossSystemConfig = "aarch64-unknown-linux-gnu";
      rustTarget = "aarch64-unknown-linux-gnu";
    };

    # Old Raspberry Pi's
    "armv6l-linux" = {
      crossSystemConfig = "armv6l-unknown-linux-musleabihf";
      rustTarget = "arm-unknown-linux-musleabihf";
    };

    "x86_64-windows" = {
      crossSystemConfig = "x86_64-w64-mingw32";
      rustTarget = "x86_64-pc-windows-gnu";
      makeBuildPackageAttrs = pkgsCross: {
        depsBuildBuild = [
          pkgsCross.stdenv.cc
          pkgsCross.windows.pthreads
        ];
      };
    };
  };
}
