let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-25.05";
  pkgs = import nixpkgs {
    config = {
      unfree = false;
    };
    overlays = [
      (import (
        builtins.fetchTarball {
          url = "https://github.com/oxalica/rust-overlay/tarball/master";
        }
      ))
    ];
  };
in

pkgs.mkShellNoCC {
  packages = with pkgs; [
    # Pull in the latest nightly Rust (with rust-src, rustfmt, and clippy)
    (rust-bin.selectLatestNightlyWith (
      toolchain:
      toolchain.default.override {
        extensions = [
          "rust-src"
          "rustfmt"
          "clippy"
        ];
      }
    ))

    # Keep the rest of your tools as before
    rust-analyzer
    cargo-bump
    sqlx-cli

    lolcat
    cowsay
    docker
    docker-buildx
    docker-compose
  ];

  GREETING = "Hikari Development Environment Activated!";

  shellHook = ''
    echo $GREETING | cowsay | lolcat
    cargo check
    cargo clippy
  '';
}
