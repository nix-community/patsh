use anyhow::{Context as _, Result};
use clap::Parser as ClapParser;
use tree_sitter::Parser;

use std::{
    ffi::OsString,
    fs::{self, File},
    path::PathBuf,
};

use patsh::{patch, Context};

/// A command-line tool for patching shell scripts
/// https://github.com/nix-community/patsh
#[derive(ClapParser)]
#[command(version, verbatim_doc_comment)]
struct Opts {
    /// the file to be patched
    input: PathBuf,

    /// output path of the patched file,
    /// defaults to the input path,
    /// however, --force is required to patch in place
    output: Option<PathBuf>,

    /// bash command used to list the built-in commands
    #[arg(short, long, default_value = "bash", value_name = "COMMAND")]
    bash: OsString,

    /// remove existing output file if needed
    #[arg(short, long)]
    force: bool,

    /// use something other than the PATH variable for path resolution
    #[arg(short, long)]
    path: Option<OsString>,

    /// path to the nix store, e.g. `builtins.storeDir`
    #[arg(short, long, default_value = "/nix/store", value_name = "PATH")]
    store_dir: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_bash::language())?;

    let src = fs::read(&opts.input)?;
    let tree = parser
        .parse(&src, None)
        .with_context(|| format!("failed to parse {}", opts.input.display()))?;

    let mut ctx = Context::load(opts.bash, opts.path, src, opts.store_dir)?;

    patch(
        &mut ctx,
        tree,
        &mut File::options()
            .write(true)
            .create(true)
            .create_new(!opts.force)
            .open(opts.output.unwrap_or(opts.input))?,
    )?;

    Ok(())
}
