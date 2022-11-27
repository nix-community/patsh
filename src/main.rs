use anyhow::{bail, Context as _, Result};
use clap::Parser as ClapParser;
use tree_sitter::{Language, Node, Parser, TreeCursor};
use which::which;

use std::{
    ffi::{OsStr, OsString},
    fs::{self, read_link, File},
    io::{BufRead, Write},
    ops::Range,
    os::unix::ffi::OsStrExt,
    path::{Component, PathBuf},
    process::Command,
};

#[link(name = "tree-sitter-bash", kind = "dylib")]
extern "C" {
    fn tree_sitter_bash() -> Language;
}

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

    /// path to the nix store, e.g. `builtins.storeDir`
    #[arg(short, long, default_value = "/nix/store", value_name = "PATH")]
    store_dir: PathBuf,
}

struct Context {
    builtins: Vec<OsString>,
    src: Vec<u8>,
    store_dir: PathBuf,
    patches: Vec<(Range<usize>, PathBuf)>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let mut parser = Parser::new();
    parser.set_language(unsafe { tree_sitter_bash() })?;

    let output = Command::new(&opts.bash).arg("-c").arg("enable").output()?;
    if !output.status.success() {
        bail!(
            "command `{} -c enable` failed: {}\n\nstdout: {}\nstderr: {}",
            opts.bash.to_string_lossy(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    let builtins: Vec<_> = output
        .stdout
        .lines()
        .filter_map(|line| line.ok()?.strip_prefix("enable ").map(Into::into))
        .collect();

    let src = fs::read(&opts.input)?;
    let tree = parser
        .parse(&src, None)
        .with_context(|| format!("failed to parse {}", opts.input.display()))?;

    let mut ctx = Context {
        builtins,
        src,
        store_dir: opts.store_dir,
        patches: Vec::new(),
    };

    walk(&mut ctx, &mut tree.walk())?;

    let mut last = 0;
    let mut file = File::options()
        .write(true)
        .create(true)
        .create_new(!opts.force)
        .open(opts.output.unwrap_or(opts.input))?;

    for (range, path) in ctx.patches {
        file.write_all(&ctx.src[last .. range.start])?;
        file.write_all(path.as_os_str().as_bytes())?;
        last = range.end;
    }
    file.write_all(&ctx.src[last ..])?;

    Ok(())
}

fn walk(ctx: &mut Context, cur: &mut TreeCursor) -> Result<()> {
    if cur.node().kind() == "command_name" && cur.goto_first_child() {
        patch_node(ctx, cur.node())?;
        cur.goto_parent();
    }

    if cur.goto_first_child() {
        walk(ctx, cur)?;
        while cur.goto_next_sibling() {
            walk(ctx, cur)?;
            while cur.goto_next_sibling() {
                walk(ctx, cur)?;
            }
            if !cur.goto_parent() {
                return Ok(());
            };
        }
    }

    Ok(())
}

fn patch_node(ctx: &mut Context, node: Node) -> Result<()> {
    let range = match node.kind() {
        "word" => node.byte_range(),
        "string" | "raw_string" => node.start_byte() + 1 .. node.end_byte() - 1,
        _ => return Ok(()),
    };

    let name = &ctx.src[range.clone()];
    if name == b"exec" {
        return if let Some(node) = node
            .parent()
            .and_then(|node| node.parent())
            .and_then(|node| node.child_by_field_name(b"argument"))
        {
            patch_node(ctx, node)
        } else {
            Ok(())
        };
    }

    let path = PathBuf::from(OsStr::from_bytes(name));
    let mut c = path.components();
    let name = match (c.next(), c.next(), c.next(), c.next(), c.next()) {
        (
            Some(Component::RootDir),
            Some(Component::Normal(usr)),
            Some(Component::Normal(bin)),
            Some(Component::Normal(name)),
            None,
        ) if usr == "usr" && bin == "bin" => name,
        (
            Some(Component::RootDir),
            Some(Component::Normal(bin)),
            Some(Component::Normal(name)),
            None,
            _,
        ) if bin == "bin" => name,
        (Some(Component::Normal(name)), None, ..) if !ctx.builtins.contains(&name.into()) => name,
        _ => return Ok(()),
    };

    // let Ok(mut path) = which(name) else { return Ok(()) };
    let mut path = if let Ok(path) = which(name) {
        path
    } else {
        return Ok(());
    };

    while let Ok(resolved) = read_link(&path) {
        if resolved.file_name() == Some(name) {
            path = resolved;
        } else {
            break;
        }
    }

    if !path.starts_with(&ctx.store_dir) {
        return Ok(());
    }

    let mut idx = ctx.patches.len();
    let mut replace = false;

    for (i, (other, _)) in ctx.patches.iter().enumerate() {
        if range.start < other.start {
            if range.end <= other.end {
                idx = i;
            } else {
                bail!("{range:?} and {other:?} overlaps");
            }
        } else if range.start < other.end {
            if range.end <= other.end {
                idx = i;
                replace = true;
            } else {
                bail!("{range:?} and {other:?} overlaps");
            }
        } else {
            break;
        }
    }

    if replace {
        ctx.patches[idx] = (range, path);
    } else {
        ctx.patches.insert(idx, (range, path));
    }

    Ok(())
}
