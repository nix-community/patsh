{
  description = "A command-line tool for patching shell scripts";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs.lib) genAttrs importTOML;
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
            inherit (nixpkgs.legacyPackages.${system}) rustPlatform;
          in
          {
            default = rustPlatform.buildRustPackage {
              pname = "patsh";
              inherit ((importTOML (self + "/Cargo.toml")).package) version;
              src = self;
              cargoLock.lockFile = self + "/Cargo.lock";
            };
          });
    };
}
