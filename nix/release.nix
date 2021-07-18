let
  fromCargo = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
  fixed = "0.1.0";
in

{ lib
, stdenv
, rustPlatform
, pkg-config
, fetchgit
, openssl
, installShellFiles
, version ? fromCargo
}:

rustPlatform.buildRustPackage rec {

  pname = "dsc";
  inherit version;

  src = fetchgit {
    url = https://github.com/docspell/dsc.git;
    rev =
      if lib.hasSuffix "-pre" version then
        "master"
      else
        "v${version}";
    leaveDotGit = true;
    sha256 = "0z0bwgrh6xq2avmbzzl0sp5c35isssbcn0xn3iky50nyf53dn6wh";
  };

  cargoSha256 = "09c9nx4qc9zv0lj0v906nl3551iw5nap0aqm93j2p8y8kqvs0vsz";

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

  meta = with lib; {
    description = "A command line interface to Docspell";
    homepage = "https://github.com/docspell/dsc";
    license = with licenses; [ gpl3 ];
    maintainers = with maintainers; [ eikek ];
  };

}
