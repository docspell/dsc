{ pkgs ? import <nixpkgs> {} }:

pkgs.callPackage (import nix/release.nix) {}
