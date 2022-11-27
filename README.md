# patsh

A command-line tool for patching shell scripts inspired by [resholve](https://github.com/abathur/resholve)

[![version](https://img.shields.io/crates/v/patsh?logo=rust&style=flat-square)][https://crates.io/crates/patsh]
[![deps](https://deps.rs/repo/github/nix-community/patsh/status.svg?style=flat-square&compact=true)](https://deps.rs/repo/github/nix-community/patsh)
[![license](https://img.shields.io/badge/license-MPL--2.0-blue?style=flat-square)](https://www.mozilla.org/en-US/MPL/2.0)
[![ci](https://img.shields.io/github/workflow/status/nix-community/patsh/ci?label=ci&logo=github-actions&style=flat-square)](https://github.com/nix-community/patsh/actions?query=workflow:ci)

```sh
nix run github:nix-community/patsh -- -f script.sh
```

## Usage

```
Usage: patsh [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   the file to be patched
  [OUTPUT]  output path of the patched file, defaults to the input path, however, --force is required to patch in place

Options:
  -f, --force    remove existing output file if needed
  -h, --help     Print help information
  -V, --version  Print version information
```

## TODO

- sudo/doas support
- escaping strings
- resolving variables
- diagnostics for unresolved commands

## Changelog

See [CHANGELOG.md](CHANGELOG.md)
