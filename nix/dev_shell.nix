{ mkEnv, ... }: {
  mkShell = cross@{ pkgs, ... }:  
    pkgs.mkShell ((mkEnv cross) // {
  });
}
