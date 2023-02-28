{ modulesPath, pkgs, lib, config, ... }:
let
  full-text-search = {
    enabled = true;
    solr.url = "http://localhost:${toString config.services.solr.port}/solr/docspell";
    postgresql = {
      pg-config = {
        "german" = "my-germam";
      };
    };
  };
  watchDir = "/docspell-watch";
  integrationHeaderValue = "test123";
in
{
  # Common development config
  imports = [ (modulesPath + "/virtualisation/qemu-vm.nix") ];
  services.openssh = {
    enable = true;
    settings.PermitRootLogin = "yes";
  };
  i18n = {
    defaultLocale = "en_US.UTF-8";
  };
  console.keyMap = "us";

  services.xserver = {
    enable = false;
  };

  networking = {
    hostName = "docspelltest";
    firewall.allowedTCPPorts = [ 7880 ];
  };
  users.users.root.password = "root";

  # Otherwise oomkiller kills docspell
  virtualisation.memorySize = 4096;

  virtualisation.forwardPorts = [
    # SSH
    { from = "host"; host.port = 64022; guest.port = 22; }
    # Docspell
    { from = "host"; host.port = 64080; guest.port = 7880; }
  ];
  system.stateVersion = "22.11";
  # This slows down the build of a vm
  documentation.enable = false;

  # Add dsc to the environment
  environment.systemPackages = [ pkgs.dsc ];
  # configure dsc-watch
  systemd.tmpfiles.rules =
    [
      "d ${watchDir} 1777 root root 10d" # directory to watch
    ];

  services.dsc-watch = {
    enable = true;
    docspell-url = "http://localhost:7880";
    exclude-filter = null;
    watchDirs =
      [
        watchDir # Note, dsc expects files to be in a subdirectory corresponding to a collective. There is no way to declaratively create a collective as of the time of writing
      ];
    integration-endpoint =
      {
        enabled = true;
        header = "Docspell-Integration:${integrationHeaderValue}";
      };
  };

  # Docspell service configuration and its requirements
  services.docspell-joex = {
    enable = true;
    waitForTarget = "solr-init.target";
    bind.address = "0.0.0.0";
    base-url = "http://localhost:7878";
    jvmArgs = [ "-J-Xmx1536M" ];
    inherit full-text-search;
  };
  services.docspell-restserver = {
    enable = true;
    bind.address = "0.0.0.0";
    openid = lib.mkForce [ ];
    backend = {
      addons.enabled = true;
      signup = {
        mode = "open";
      };
    };
    integration-endpoint = {
      enabled = true;
      http-header = {
        enabled = true;
        header-value = integrationHeaderValue;
      };
    };
    inherit full-text-search;
    extraConfig = {
      files = {
        default-store = "database";
        stores = {
          minio = {
            enabled = true;
          };
        };
      };
    };
  };
  nixpkgs.config = {
    permittedInsecurePackages = [
      "solr-8.6.3"
    ];
  };

  services.solr = {
    enable = true;
  };
  # This is needed to run solr script as user solr
  users.users.solr.useDefaultShell = true;

  systemd.services.solr-init =
    let
      solrPort = toString config.services.solr.port;
      initSolr = ''
        if [ ! -f ${config.services.solr.stateDir}/docspell_core ]; then
          while ! echo "" | ${pkgs.inetutils}/bin/telnet localhost ${solrPort}
          do
             echo "Waiting for SOLR become ready..."
             sleep 1.5
          done
          ${pkgs.su}/bin/su -s ${pkgs.bash}/bin/sh solr -c "${pkgs.solr}/bin/solr create_core -c docspell -p ${solrPort}";
          touch ${config.services.solr.stateDir}/docspell_core
        fi
      '';
    in
    {
      script = initSolr;
      after = [ "solr.target" ];
      wantedBy = [ "multi-user.target" ];
      requires = [ "solr.target" ];
      description = "Create a core at solr";
    };


}
