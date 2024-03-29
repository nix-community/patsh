# Changelog

## v0.2.1 - 2023-08-18

### Changes

- use the `tree-sitter-bash` crate instead of the git repository
- update dependencies

## v0.2.0 - 2022-12-04

### Features

- Better support for quoted strings
- Support for `sodo`, `doas`, `command`, and `type`

## v0.1.3 - 2022-11-27

### Features

- `--bash` to specify the bash command used to list the built-in commands
- `--store` to specify the path to the nix store, e.g. `builtins.storeDir`
- `--path` to use something other than the PATH variable for path resolution

## v0.1.2 - 2022-11-27

### Features

- Support for `exec`

## v0.1.1 - 2022-11-27

### Fixes

- Correctly output end of the file

## v0.1.0 - 2022-11-26

First release
