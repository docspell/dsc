{ config, lib, pkgs, ... }:

with lib;
let
  cfg = config.services.dsc-watch;
in
{

  ## interface
  options = {
    services.dsc-watch = {
      enable = mkOption {
        default = false;
        description = "Whether to enable dsc watch directory service.";
      };

      package = mkOption {
        default = pkgs.dsc;
        description = "Package that provides the dsc binary.";
        type = types.package;
      };

      docspell-url = mkOption {
        type = types.str;
        example = "http://localhost:7880";
        description = "The base url to the docspell server.";
      };

      watchDirs = mkOption {
        type = types.listOf types.str;
        description = "The directories to watch for new files.";
      };

      recursive = mkOption {
        type = types.bool;
        default = true;
        description = "Whether to watch directories recursively.";
      };


      verbose = mkOption {
        type = types.bool;
        default = false;
        description = "Run in verbose mode";
      };

      delete-files = mkOption {
        type = types.bool;
        default = false;
        description = "Whether to delete successfully uploaded files.";
      };

      include-filter = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "A filter for files to include when watching";
      };

      exclude-filter = mkOption {
        type = types.nullOr types.str;
        default = ".*";
        description = "A filter for files to exclude when watching";
      };

      integration-endpoint = mkOption {
        type = types.submodule ({
          options = {
            enabled = mkOption {
              type = types.bool;
              default = false;
              description = "Whether to upload to the integration endpoint.";
            };
            header-file = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = "A file containing the `header:value` pair for the integration endpoint.";
            };
            basic-file = mkOption {
              type = types.nullOr types.path;
              default = null;
              description = "A file containing the `user:password` pair for the integration endpoint.";
            };
            header = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = ''
                The `header:value` string matching the configured header-name
                 and value for the integration endpoint.
              '';
            };
            basic = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = ''
                The `user:password` string matching the configured user and password
                for the integration endpoint. Since both are separated by a colon, the
                user name may not contain a colon (the password can).
              '';
            };
          };
        });
        default = {
          enabled = false;
          header = null;
          basic = null;
        };
        description = "Settings for using the integration endpoint.";
      };
      source-id = mkOption {
        type = types.nullOr types.str;
        default = null;
        example = "abced-12345-abcde-12345";
        description = ''
          A source id to use for uploading. This is used when the
          integration endpoint setting is disabled.
        '';
      };
    };
  };

  ## implementation
  config = mkIf config.services.dsc-watch.enable {

    systemd.user.services.dsc-watch =
      let
        args = (if cfg.recursive then [ "-r" ] else [ ]) ++
          (if cfg.delete-files then [ "--delete" ] else [ ]) ++
          (if cfg.integration-endpoint.enabled then [ "-i" ] else [ ]) ++
          (if cfg.integration-endpoint.header-file != null
           then [ "--header-file" "'${cfg.integration-endpoint.header-file}'" ]
           else if cfg.integration-endpoint.basic-file != null
           then [ "--basic-file" "'${cfg.integration-endpoint.basic-file}'" ]
           else if cfg.integration-endpoint.header != null
           then [ "--header" "'${cfg.integration-endpoint.header}'" ]
           else if cfg.integration-endpoint.basic != null
           then [ "--basic" "'${cfg.integration-endpoint.basic}'" ]
           else [ ]) ++
          (if cfg.include-filter != null then
            [ "--matches" "'${toString cfg.include-filter}'" ]
          else [ ]) ++
          (if cfg.exclude-filter != null then
            [ "--not-matches" "'${toString cfg.exclude-filter}'" ]
          else [ ]) ++
          (if cfg.source-id != null then [ "--source" "'${cfg.source-id}'" ] else [ ]);

        cmd = "${cfg.package}/bin/dsc " +
          "-d '${cfg.docspell-url}'" +
          (if cfg.verbose then " -vv " else "") +
          " watch " +
          (builtins.concatStringsSep " " args) + " " +
          (builtins.concatStringsSep " " cfg.watchDirs);
      in
      {
        description = "Docspell Watch Directory";
        after = [ "networking.target" ];
        wants = [ "networking.target" ];
        wantedBy = [ "default.target" ];
        serviceConfig = {
          Restart = "on-failure";
          RestartSec = 5;
        };
        path = [ ];

        script = ''echo "Running for user: $(whoami)" && ${cmd}'';
      };
  };
}
