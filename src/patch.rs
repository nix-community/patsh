use anyhow::Result;
use is_executable::IsExecutable;
use tree_sitter::{Node, Tree, TreeCursor};

use std::{
    fs::read_link,
    io::Write,
    ops::Range,
    os::unix::prelude::OsStrExt,
    path::{Component, PathBuf},
    str,
};

use crate::{parse_command, Context};

pub fn patch(ctx: &mut Context, tree: Tree, out: &mut impl Write) -> Result<()> {
    walk(ctx, &mut tree.walk())?;

    let mut last = 0;
    for (range, path) in &ctx.patches {
        out.write_all(&ctx.src[last .. range.start])?;
        let path = path.as_os_str().as_bytes();
        if let Ok(path) = str::from_utf8(path) {
            write!(out, "{}", shell_escape::escape(path.into()))?;
        } else {
            out.write_all(b"'")?;
            out.write_all(path)?;
            out.write_all(b"'")?;
        }
        last = range.end;
    }
    out.write_all(&ctx.src[last ..])?;

    Ok(())
}

fn walk(ctx: &mut Context, cur: &mut TreeCursor) -> Result<()> {
    if cur.node().kind() == "command_name" && cur.goto_first_child() {
        patch_node(ctx, cur.node());
        cur.goto_parent();
    }

    if cur.goto_first_child() {
        walk(ctx, cur)?;
        cur.goto_parent();
    }

    if cur.goto_next_sibling() {
        walk(ctx, cur)?;
    }

    Ok(())
}

fn patch_node(ctx: &mut Context, node: Node) {
    let (no_builtins, commands) = parse_command(ctx, &node);
    for (range, name) in commands {
        let path = PathBuf::from(name);
        if path.starts_with(&ctx.store_dir) {
            continue;
        }

        let mut c = path.components();
        let name = match c.next() {
            Some(Component::RootDir) => {
                if let Some(Component::Normal(name)) = c.last() {
                    name
                } else {
                    continue;
                }
            }
            Some(Component::Normal(name))
                if c.next().is_none() && (no_builtins || !ctx.builtins.contains(&name.into())) =>
            {
                name
            }
            _ => continue,
        };

        let Some(idx) = get_patch_index(&ctx.patches, &range) else { continue; };

        let Some(mut path) = ctx.paths.iter().find_map(|path| {
            let path = path.join(name);
            path.is_executable().then_some(path)
        }) else {
            continue;
        };

        while let Ok(resolved) = read_link(&path) {
            if resolved.file_name() == Some(name) {
                path = resolved;
            } else {
                break;
            }
        }

        if path.starts_with(&ctx.store_dir) {
            add_patch(&mut ctx.patches, idx, range, path);
        }
    }
}

pub(crate) fn get_patch_index(
    patches: &[(Range<usize>, PathBuf)],
    range: &Range<usize>,
) -> Option<(usize, bool)> {
    let mut idx = patches.len();
    let mut replace = false;

    for (i, (other, _)) in patches.iter().enumerate() {
        if range == other {
            return None;
        } else if range.start < other.start {
            if range.end <= other.end {
                idx = i;
            } else {
                panic!("{range:?} and {other:?} overlaps");
            }
        } else if range.start < other.end {
            if range.end <= other.end {
                idx = i;
                replace = true;
            } else {
                panic!("{range:?} and {other:?} overlaps");
            }
        } else {
            break;
        }
    }

    Some((idx, replace))
}

pub(crate) fn add_patch(
    patches: &mut Vec<(Range<usize>, PathBuf)>,
    (idx, replace): (usize, bool),
    range: Range<usize>,
    path: PathBuf,
) {
    if replace {
        patches[idx] = (range, path);
    } else {
        patches.insert(idx, (range, path));
    }
}
