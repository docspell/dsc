{ callPackage, fetchFromGitHub, makeRustPlatform }:
let
  moz_overlay = builtins.fetchTarball
    https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz;

  mozilla = callPackage "${moz_overlay}/package-set.nix" {};

  rust_stable = mozilla.latest.rustChannels.stable.rust;
in makeRustPlatform {
  cargo = rust_stable;
  rustc = rust_stable;
}
