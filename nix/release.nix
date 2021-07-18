let
  fromCargo = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).package.version;
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

  # src = fetchgit {
  #   url = https://github.com/docspell/dsc.git;
  #   rev =
  #     if lib.hasSuffix "-pre" version then
  #       "master"
  #     else
  #       "v${version}";
  #   leaveDotGit = true;
  #   sha256 = "0z0bwgrh6xq2avmbzzl0sp5c35isssbcn0xn3iky50nyf53dn6wh";
  # };
  src =
    let
      cleanSrcFilter = name: type:
        let basename = baseNameOf (toString name); in
        type != "directory" || basename != "target";
      cleanSrc = src: lib.cleanSourceWith {
        filter = cleanSrcFilter;
        inherit src;
      };
    in cleanSrc ../.;

  cargoSha256 = "0j5nprcszgk75rrzjiymwcrww705x7fsq2kwsp5iy4xi7k449r5p";

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
