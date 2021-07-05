let
  pkgs_source = import (builtins.fetchTarball "channel:nixos-21.05");
  moz_overlay = import (builtins.fetchTarball
    https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);

  pkgs = pkgs_source { overlays = [ moz_overlay ]; };
  arm = pkgs_source {
    crossSystem = pkgs.lib.systems.examples.raspberryPi;
  };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs;
    [ pkg-config
      arm.pkg-config
    ];
  buildInputs = with pkgs;
    [ # rustfmt
      # clippy
      (pkgs.rustChannelOfTargets "stable" null
                          [ "x86_64-unknown-linux-gnu"
                            "arm-unknown-linux-gnueabihf" ])
      arm.stdenv.cc
    ];
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
#  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
