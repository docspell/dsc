{
  description = "A command line interface to Docspell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk/master";
    docspell-flake = {
      url = "github:eikek/docspell?dir=nix";
    };
  };

  outputs = inputs@{ flake-parts, self, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];
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
          packages = rec
          {
            default = naersk-lib.buildPackage {
              root = ./.;
              meta = with pkgs.lib; {
                description = "A command line interface to Docspell";
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
            dsc = default;
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
          overlayAttrs = {
            inherit (config.packages) dsc;
          };
          formatter = pkgs.nixpkgs-fmt;
        };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.
        nixosConfigurations.dev-vm =
          let
            system = "x86_64-linux";
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays =
                [
                  self.overlays.default
                  inputs.docspell-flake.overlays.default
                ];
            };
          in
          inputs.nixpkgs.lib.nixosSystem {
            inherit pkgs system;
            modules =
              [
                inputs.docspell-flake.nixosModules.default
                ./nix/nixosConfigurations
              ];
          };
      };
    };
}
