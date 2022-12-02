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
    nix-filter.url = "github:numtide/nix-filter";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, crane, fenix, flake-utils, nix-filter, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        inherit (crane.lib.${system}.overrideToolchain fenix.packages.${system}.default.toolchain)
          buildDepsOnly buildPackage cargoClippy cargoFmt cargoNextest;
        inherit (nixpkgs.legacyPackages.${system}) coreutils libiconv nixpkgs-fmt stdenv;
        inherit (nixpkgs.lib) optional;

        args' = {
          src = nix-filter.lib {
            root = self;
            include = [
              "src"
              "tests"
              "Cargo.lock"
              "Cargo.toml"
              "rustfmt.toml"
            ];
          };

          buildInputs = optional stdenv.isDarwin libiconv;

          postPatch = ''
            substituteInPlace tests/fixtures/*-expected.sh \
              --subst-var-by coreutils ${coreutils}
          '';
        };

        args = args' // {
          cargoArtifacts = buildDepsOnly args';
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
