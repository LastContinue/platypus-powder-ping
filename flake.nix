{
  description = "Basic Rust Flake for ARM Macs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "aarch64-darwin";
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
      devShells.${system}.default = pkgs.mkShell {
        inherit buildInputs;
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        # DEVSHELL_NO_MOTD = 1;
      };
      formatter.${system} = pkgs.nixfmt;
    };
}
