{
  description = "kindergarten puzzle";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
        rust = pkgs.rust-bin.stable.latest.default.override {
          targets = [
            "wasm32-unknown-unknown"
          ];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs;
            [
              wasm-pack
              just
              python3
              watchexec
              nushell
              alejandra
              deadnix
              nixd
              clang
              pkg-config
              taplo
              prettierd
              prettier
              shfmt
              pnpm
              # add mado and mdsf
            ]
            ++ [
              rust
            ];
        };
      }
    );
}
