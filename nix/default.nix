{ pkgs ? import <nixpkgs> {} }:

{
  dsc = pkgs.callPackage (import ./release.nix) {};
}
