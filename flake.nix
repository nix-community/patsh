{
  inputs = {
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
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      imports = [ inputs.treefmt-nix.flakeModule ];

      perSystem =
        {
          config,
          inputs',
          lib,
          pkgs,
          ...
        }:
        let
          inherit (pkgs)
            callPackage
            ;
        in
        {
          packages = {
            default = callPackage ./package.nix { };
          };

          checks =
            let
              packages = lib.mapAttrs' (n: lib.nameValuePair "package-${n}") config.packages;
              checks = {
                clippy = config.packages.default.overrideAttrs (old: {
                  pname = "patsh-clippy";

                  nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.clippy ];

                  doCheck = false;

                  buildPhase = ''
                    runHook preBuild
                    cargo clippy --target ${pkgs.stdenv.targetPlatform.rust.rustcTarget} \
                      --offline --no-default-features -- -D warnings
                    runHook postBuild
                  '';

                  installPhase = ''
                    touch $out
                  '';
                });
              };
            in
            packages // checks;

          treefmt = {
            programs = {
              actionlint.enable = true;
              deadnix.enable = true;
              nixfmt.enable = true;
              oxfmt.enable = true;
              rustfmt = {
                enable = true;
                package = inputs'.fenix.packages.latest.rustfmt;
              };
              statix.enable = true;
              taplo.enable = true;
              zizmor.enable = true;
            };
          };
        };
    };
}
