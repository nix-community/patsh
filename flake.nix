{
  description = "A command-line tool for patching shell scripts";

  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-compat.follows = "crane";
      inputs.rust-overlay.follows = "crane";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, crane, nixpkgs }:
    let
      inherit (nixpkgs.lib) genAttrs optional;
    in
    {
      packages = genAttrs
        [
          "aarch64-darwin"
          "aarch64-linux"
          "x86_64-darwin"
          "x86_64-linux"
        ]
        (system:
          let
            inherit (nixpkgs.legacyPackages.${system}) libiconv stdenv;
            inherit (crane.lib.${system}) buildPackage cleanCargoSource;
          in
          {
            default = buildPackage {
              src = cleanCargoSource self;
              buildInputs = optional stdenv.isDarwin libiconv;
            };
          });
    };
}
