{
  description = "Sentinel - semantic predicates for software delivery";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, fenix }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            pkgs = nixpkgs.legacyPackages.${system};
            fenixPkgs = fenix.packages.${system};
          }
        );
    in
    {
      packages = forAllSystems ({ pkgs, ... }: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "sentinel";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      });

      devShells = forAllSystems ({ pkgs, fenixPkgs }: rec {
        rustToolchain = with fenixPkgs; combine [
          stable.rustc
          stable.cargo
          stable.rustfmt
          stable.clippy

          # Bundled into the sysroot (lib/rustlib/src/rust/library), so
          # rust-analyzer finds the standard library without RUST_SRC_PATH.
          stable.rust-src
        ];

        default = pkgs.mkShell {
          packages = [
            rustToolchain
            fenixPkgs.rust-analyzer
          ];
        };
      });
    };
}
