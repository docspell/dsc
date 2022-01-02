let
  fromCargo = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
in

{ lib
, stdenv
, pkg-config
, fetchgit
, openssl
, installShellFiles
, git
, binutils-unwrapped
, callPackage
, version ? fromCargo
}:
let
  rustPlatform = callPackage ./rust-platform.nix {};
in
rustPlatform.buildRustPackage rec {

  pname = "dsc";
  inherit version;

  # src = fetchgit {
  #   url = https://github.com/docspell/dsc.git;
  #   rev =
  #     if lib.hasSuffix "-pre" version then
  #       "master"
  #     else
  #       "v${version}";
  #   # leaveDotGit = true;
  #   sha256 = "0wnl72bcn3mpy1n4rbzrffsibjjm28smzs7bszsvyb97rdj93yzw";
  # };
  src =
    let
      cleanSrcFilter = name: type:
        let basename = baseNameOf (toString name); in
        type != "directory" || (basename != "target" && basename != "nix" && basename != "tests");
      cleanSrc = src: lib.cleanSourceWith {
        filter = cleanSrcFilter;
        inherit src;
      };
    in cleanSrc ../.;

  cargoSha256 = "19896dzh1xda0f4nqa1p91n6nrv5vg8imm7961kn4s8nxyjjp69p";

  # only unit tests can be run
  checkPhase = ''
    cargo test --release unit
  '';

#  cargoBuildFlags = "--no-default-features --features rustls";

  PKG_CONFIG_PATH = "${openssl.dev}/lib/pkgconfig";
  nativeBuildInputs = [  installShellFiles openssl pkg-config ];
  # buildInputs = lib.optional stdenv.isDarwin Security;

  preFixup = ''
    for shell in fish zsh bash; do
      $out/bin/dsc generate-completions --shell $shell > dsc.$shell
      installShellCompletion --$shell dsc.$shell
    done
  '';

  strip = true;

  ## the strip=true above seems not to strip the binary
  postInstall = ''
    echo "Stripping $out/bin/dsc …"
    ${binutils-unwrapped}/bin/strip $out/bin/dsc
  '';

  meta = with lib; {
    description = "A command line interface to Docspell";
    homepage = "https://github.com/docspell/dsc";
    license = with licenses; [ gpl3 ];
    maintainers = with maintainers; [ eikek ];
  };

}
