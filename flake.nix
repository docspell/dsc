{
  description = "A command line interface to Docspell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk/master";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ ];
      systems =
        [
          "aarch64-linux"
          "aarch64-darwin"
          "x86_64-darwin"
          "x86_64-linux"
        ]; # List taken from flake-utils
      perSystem = { config, self', inputs', pkgs, system, ... }:
        let
          naersk-lib = pkgs.callPackage inputs.naersk { };
        in
        rec {
          packages.default = naersk-lib.buildPackage {
            root = ./.;
            meta = with pkgs.lib; {
              inherit description;
              homepage = "https://github.com/docspell/dsc";
              license = with licenses; [ gpl3 ];
              maintainers = with maintainers; [ eikek ];
            };
            nativeBuildInputs = with pkgs;
              [
                pkg-config
                openssl
                installShellFiles
              ];
            postInstall =
              ''
                for shell in fish zsh bash; do
                  $out/bin/dsc generate-completions --shell $shell > dsc.$shell
                  installShellCompletion --$shell dsc.$shell
                done
              '';
          };
          apps.default = {
            type = "app";
            program = "${packages.default}/bin/dsc";
          };
          devShells.default = with pkgs; mkShell {
            buildInputs = [
              cargo
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
          };
          formatter = pkgs.nixpkgs-fmt;
        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.
      };
    };
}
