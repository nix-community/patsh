{
  lib,
  runCommand,
  rustPlatform,
  stdenv,
  coreutils,
}:

let
  test-support = runCommand "patsh-test-support" { } ''
    mkdir -p $out/bin
    touch $out/bin/{'foo$','foo"`'}
    chmod +x $out/bin/{'foo$','foo"`'}
  '';
in

rustPlatform.buildRustPackage {
  pname = "patsh";
  inherit ((lib.importTOML ./Cargo.toml).package) version;
  __structuredAttrs = true;

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.lock
      ./Cargo.toml
      ./src
      ./tests
    ];
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeCheckInputs = [ test-support ];

  postPatch = ''
    for file in tests/fixtures/*-expected.sh; do
      substituteInPlace $file \
        --subst-var-by cc ${stdenv.cc} \
        --subst-var-by coreutils ${coreutils} \
        --subst-var-by test_support ${test-support}
    done
  '';
}
