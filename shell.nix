# Shell expression for the Nix package manager
#
# This nix expression creates an environment with necessary packages installed:
#
#  * rust
#
# To use:
#
#  $ nix-shell
#

{ pkgs ? import <nixpkgs> {} }:

with builtins;
let
  inherit (pkgs) stdenv;
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rust_build = nixpkgs.latest.rustChannels.nightly.rust;
in
  with pkgs;
  stdenv.mkDerivation {
    name = "timex";
    buildInputs = [
      rust_build
      ];
     LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";
  }
