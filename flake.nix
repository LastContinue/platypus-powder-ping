{
  description = "Basic Rust Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # nativeBuildInputs = with pkgs; [ ];
        buildInputs = with pkgs; [
          cargo
          cargo-machete
          clippy
          lldb
          rustc
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
