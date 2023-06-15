{
  description = "A command-line tool for patching shell scripts";

  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-compat.follows = "";
      inputs.rust-overlay.follows = "";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ crane, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      perSystem = { inputs', lib, pkgs, self', system, ... }:
        let
          inherit (lib)
            optional
            sourceByRegex
            ;
          inherit (crane.lib.${system}.overrideToolchain inputs'.fenix.packages.default.toolchain)
            buildDepsOnly
            buildPackage
            cargoClippy
            cargoFmt
            cargoNextest
            ;
          inherit (pkgs)
            coreutils
            libiconv
            nixpkgs-fmt
            runCommand
            stdenv
            ;

          custom = runCommand "custom" { } ''
            mkdir -p $out/bin
            touch $out/bin/{'foo$','foo"`'}
            chmod +x $out/bin/{'foo$','foo"`'}
          '';

          args = {
            src = sourceByRegex ./. [
              "(src|tests)(/.*)?"
              "Cargo\\.(toml|lock)"
              ''rustfmt\.toml''
            ];

            buildInputs = optional stdenv.isDarwin libiconv;

            checkInputs = [ custom ];

            cargoArtifacts = buildDepsOnly args;

            postPatch = ''
              for file in tests/fixtures/*-expected.sh; do
                substituteInPlace $file \
                  --subst-var-by cc ${stdenv.cc} \
                  --subst-var-by coreutils ${coreutils} \
                  --subst-var-by custom ${custom}
              done
            '';
          };
        in
        {
          checks = {
            build = self'.packages.default;
            clippy = cargoClippy (args // {
              cargoClippyExtraArgs = "-- -D warnings";
            });
            fmt = cargoFmt args;
            test = cargoNextest args;
          };

          formatter = nixpkgs-fmt;

          packages.default = buildPackage (args // {
            doCheck = false;
          });
        };
    };
}
