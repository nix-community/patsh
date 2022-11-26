{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs.lib) genAttrs importTOML;

      forEachSystem = genAttrs [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      tree-sitter-bash = { runCommand, tree-sitter }: runCommand "tree-sitter-bash" { } ''
        mkdir $out
        ln -s ${tree-sitter.builtGrammars.tree-sitter-bash}/parser $out/libtree-sitter-bash.a
      '';
    in
    {
      devShells = forEachSystem (system:
        let
          inherit (nixpkgs.legacyPackages.${system}) callPackage mkShell;
        in
        {
          default = mkShell {
            LD_LIBRARY_PATH = callPackage tree-sitter-bash { };
            TREE_SITTER_BASH = callPackage tree-sitter-bash { };
          };
        });

      packages = forEachSystem (system:
        let
          inherit (nixpkgs.legacyPackages.${system}) callPackage rustPlatform;
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "patsh";
            inherit ((importTOML (self + "/Cargo.toml")).package) version;
            src = self;
            cargoLock.lockFile = self + "/Cargo.lock";
            TREE_SITTER_BASH = callPackage tree-sitter-bash { };
          };
        });
    };
}
