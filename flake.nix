{
  description = "A command-line tool for patching shell scripts";

  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.flake-utils.follows = "flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-compat.follows = "crane";
      inputs.rust-overlay.follows = "crane";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "fenix";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, crane, fenix, flake-utils, nixpkgs }:
    {
      herculesCI.ciSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
    } // flake-utils.lib.eachDefaultSystem (system:
      let
        inherit (crane.lib.${system}.overrideToolchain fenix.packages.${system}.default.toolchain)
          buildDepsOnly buildPackage cargoClippy cargoFmt cargoNextest;
        inherit (nixpkgs.legacyPackages.${system}) coreutils libiconv nixpkgs-fmt runCommand stdenv;
        inherit (nixpkgs.lib) optional sourceByRegex;

        custom = runCommand "custom" { } ''
          mkdir -p $out/bin
          touch $out/bin/{'foo$','foo"`'}
          chmod +x $out/bin/{'foo$','foo"`'}
        '';

        args = {
          src = sourceByRegex self [
            "(src|tests)(/.*)?"
            "Cargo\\.(toml|lock)"
            "rustfmt.toml"
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
          build = self.packages.${system}.default;
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
      });
}
