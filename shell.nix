{ pkgs ? import <nixpkgs> {} }: pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.clippy
  ];
  shellHook =  ''
    echo "cargo build"
  '';
}
