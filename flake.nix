{
  description = "A command-line tool for patching shell scripts";

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
    in
    {
      devShells = forEachSystem (system:
        let
          inherit (nixpkgs.legacyPackages.${system}) mkShell;
        in
        {
          default = mkShell {
            LD_LIBRARY_PATH = self.packages.${system}.tree-sitter-bash;
            TREE_SITTER_BASH = self.packages.${system}.tree-sitter-bash;
          };
        });

      packages = forEachSystem (system:
        let
          inherit (nixpkgs.legacyPackages.${system}) runCommand rustPlatform tree-sitter;
        in
        {
          default = rustPlatform.buildRustPackage {
            pname = "patsh";
            inherit ((importTOML (self + "/Cargo.toml")).package) version;
            src = self;
            cargoLock.lockFile = self + "/Cargo.lock";
            TREE_SITTER_BASH = self.packages.${system}.tree-sitter-bash;
          };

          tree-sitter-bash = runCommand "tree-sitter-bash" { } ''
            install -Dm444 ${tree-sitter.builtGrammars.tree-sitter-bash}/parser $out/libtree-sitter-bash.a
          '';
        });
    };
}
