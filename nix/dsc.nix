{ lib
, stdenv
, pkg-config
, fetchgit
, openssl
, installShellFiles
, git
, binutils-unwrapped
, rustPlatform
, pname
, version
, description
}:
rustPlatform.buildRustPackage rec {
  inherit pname version;

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

  cargoLock.lockFile = ../Cargo.lock;

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
    inherit description;
    homepage = "https://github.com/docspell/dsc";
    license = with licenses; [ gpl3 ];
    maintainers = with maintainers; [ eikek ];
  };
}
