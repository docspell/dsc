# Adopted from https://github.com/srid/rust-nix-template/blob/master/flake.nix

{
  description = "A command line interface to Docspell";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-21.11";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay, ... }:
    let
      # If you change the name here, you must also do it in Cargo.toml
      name = "dsc";
      rustChannel = "stable";

      commit = self.shortRev or "dirty";
      date = self.lastModifiedDate or self.lastModified or "19700101";
      baseVersion = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
      version = "${baseVersion}+${builtins.substring 0 8 date}-${commit}";

    in
    utils.lib.eachDefaultSystem
      (system:
        let
          # Imports
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlay
              (self: super:
                let
                  rustLatest = self.rust-bin.${rustChannel}.latest.default;
                in {
                  rustc = rustLatest;
                  cargo = rustLatest;
                  rustPlatform = pkgs.makeRustPlatform {
                    rustc = rustLatest;
                    cargo = rustLatest;
                  };
              })
            ];
          };
        in
        rec {
          packages.${name} = pkgs.callPackage ./nix/dsc.nix {
            inherit version;
            description = self.description;
            pname = name;
          };

          # `nix build`
          defaultPackage = packages.${name};

          # `nix run`
          apps.${name} = utils.lib.mkApp {
            inherit name;
            drv = packages.${name};
          };
          defaultApp = apps.${name};

          # `nix develop`
          devShell = pkgs.mkShell
            {
              inputsFrom = builtins.attrValues self.packages.${system};
              buildInputs = with pkgs;
                # Tools you need for development go here.
                [
                  nixpkgs-fmt
                  cargo-watch
                  pkgs.rust-bin.${rustChannel}.latest.rust-analysis
                  pkgs.rust-bin.${rustChannel}.latest.rls
                ];

              RUST_SRC_PATH = "${pkgs.rust-bin.${rustChannel}.latest.rust-src}/lib/rustlib/src/rust/library";
            };
        }
      );
}
