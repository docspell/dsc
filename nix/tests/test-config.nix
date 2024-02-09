{ config, pkgs, ... }:
let
  dsc = import ../release.nix;
  docspellsrc = builtins.fetchTarball
    "https://github.com/eikek/docspell/archive/master.tar.gz";
  docspell = import "${docspellsrc}/nix/release.nix";
in {
  imports = [ ../module.nix ] ++ docspell.modules;

  i18n = { defaultLocale = "en_US.UTF-8"; };
  console.keyMap = "de";

  users.users.root = { password = "root"; };

  nixpkgs = {
    config = {
      packageOverrides = pkgs:
        let
          callPackage = pkgs.lib.callPackageWith (custom // pkgs);
          custom = {
            dsc = callPackage dsc { };
            docspell = callPackage docspell.currentPkg { };
          };
        in custom;
    };
  };

  services.dsc-watch = {
    enable = true;
    verbose = false;
    delete-files = true;
    docspell-url = "http://localhost:7880";
    integration-endpoint = {
      enabled = true;
      header = "Docspell-Integration:test123";
    };
    watchDirs = [ "/tmp/docs" ];
  };

  services.docspell-restserver = {
    enable = true;
    bind.address = "0.0.0.0";
    integration-endpoint = {
      enabled = true;
      http-header = {
        enabled = true;
        header-value = "test123";
      };
    };
    full-text-search = { enabled = false; };
  };

  environment.systemPackages = [ pkgs.jq pkgs.telnet pkgs.htop pkgs.dsc ];

  services.xserver = { enable = false; };

  networking = {
    hostName = "dsctest";
    firewall.allowedTCPPorts = [ 7880 ];
  };

  system.activationScripts = {
    initUploadDir = ''
      mkdir -p ${
        builtins.concatStringsSep " " config.services.dsc-watch.watchDirs
      }

    '';
  };
  system.stateVersion = "21.05";
}
