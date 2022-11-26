{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    devShells = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed
      (system:
        let
          inherit (nixpkgs.legacyPackages.${system}) mkShell runCommand tree-sitter;
          tree-sitter-bash = runCommand "tree-sitter-bash" { } ''
            mkdir $out
            ln -s ${tree-sitter.builtGrammars.tree-sitter-bash}/parser $out/libtree-sitter-bash.a
          '';
        in
        {
          default = mkShell
            {
              LD_LIBRARY_PATH = tree-sitter-bash;
              TREE_SITTER_BASH = tree-sitter-bash;
            };
        });
  };
}
