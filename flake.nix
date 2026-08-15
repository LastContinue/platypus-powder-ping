{
  description = "Basic Rust Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "llvm-tools-preview" ];
        };

        buildInputs = with pkgs; [
          rustToolchain
          cargo-llvm-cov
          cargo-machete
          clippy
          lldb
          rustfmt
          rust-analyzer
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          # DEVSHELL_NO_MOTD = 1;
        };
        formatter = pkgs.nixfmt;
      }
    );
}
