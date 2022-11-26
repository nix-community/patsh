use anyhow::{bail, Context, Result};
use clap::Parser as ClapParser;
use tree_sitter::{Language, Node, Parser, TreeCursor};
use which::which;

use std::{
    ffi::OsStr,
    fs::{self, read_link, File},
    io::Write,
    ops::Range,
    os::unix::ffi::OsStrExt,
    path::{Component, PathBuf},
};

#[link(name = "tree-sitter-bash", kind = "dylib")]
extern "C" {
    fn tree_sitter_bash() -> Language;
}

type Patches = Vec<(Range<usize>, PathBuf)>;

#[derive(ClapParser)]
struct Opts {
    input: PathBuf,
    output: PathBuf,
    #[arg(short, long)]
    force: bool,
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    let mut parser = Parser::new();
    parser.set_language(unsafe { tree_sitter_bash() })?;

    let src = fs::read(&opts.input)?;
    let tree = parser
        .parse(&src, None)
        .with_context(|| format!("failed to parse {}", opts.input.display()))?;

    let mut patches = Vec::new();
    walk(&mut patches, &src, &mut tree.walk())?;

    let mut last = 0;
    let mut file = File::options()
        .write(true)
        .create(true)
        .create_new(!opts.force)
        .open(opts.output)?;

    for (range, path) in patches {
        file.write_all(&src[last .. range.start])?;
        file.write_all(path.as_os_str().as_bytes())?;
        last = range.end;
    }

    Ok(())
}

fn walk(patches: &mut Patches, src: &[u8], cur: &mut TreeCursor) -> Result<()> {
    if cur.node().kind() == "command_name" && cur.goto_first_child() {
        patch_node(patches, src, cur.node())?;
        cur.goto_parent();
    }

    if cur.goto_first_child() {
        walk(patches, src, cur)?;
        while cur.goto_next_sibling() {
            walk(patches, src, cur)?;
            while cur.goto_next_sibling() {
                walk(patches, src, cur)?;
            }
            if !cur.goto_parent() {
                return Ok(());
            };
        }
    }

    Ok(())
}

fn patch_node(patches: &mut Patches, src: &[u8], node: Node) -> Result<()> {
    if node.child_count() != 0 {
        return Ok(());
    };

    let range = node.byte_range();
    let path = PathBuf::from(OsStr::from_bytes(&src[range.clone()]));
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
        (Some(Component::Normal(name)), None, ..) => name,
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

    if !path.starts_with("/nix/store") {
        return Ok(());
    }

    let mut idx = patches.len();
    let mut replace = false;

    for (i, (other, _)) in patches.iter().enumerate() {
        if range.start < other.start {
            if range.end <= other.end {
                idx = i;
            } else {
                bail!("{range:?} and {other:?} overlaps");
            }
        } else if range.start < other.end {
            if range.end <= other.end {
                replace = true;
            } else {
                bail!("{range:?} and {other:?} overlaps");
            }
        } else {
            break;
        }
    }

    if replace {
        patches[idx] = (range, path);
    } else {
        patches.insert(idx, (range, path));
    }

    Ok(())
}
