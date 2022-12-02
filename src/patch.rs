use anyhow::{bail, Result};
use is_executable::IsExecutable;
use tree_sitter::{Node, Tree, TreeCursor};

use std::{
    ffi::OsStr,
    fs::read_link,
    io::Write,
    os::unix::prelude::OsStrExt,
    path::{Component, PathBuf},
};

use crate::context::Context;

pub fn patch(ctx: &mut Context, tree: Tree, out: &mut impl Write) -> Result<()> {
    walk(ctx, &mut tree.walk())?;

    let mut last = 0;
    for (range, path) in &ctx.patches {
        out.write_all(&ctx.src[last .. range.start])?;
        out.write_all(path.as_os_str().as_bytes())?;
        last = range.end;
    }
    out.write_all(&ctx.src[last ..])?;

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
    if path.starts_with(&ctx.store_dir) {
        return Ok(());
    }

    let mut c = path.components();
    let name = match c.next() {
        Some(Component::RootDir) => {
            if let Some(Component::Normal(name)) = c.last() {
                name
            } else {
                return Ok(());
            }
        }
        Some(Component::Normal(name))
            if c.next().is_none() && !ctx.builtins.contains(&name.into()) =>
        {
            name
        }
        _ => return Ok(()),
    };

    let mut path = if let Some(path) = ctx.paths.iter().find_map(|path| {
        let path = path.join(name);
        path.is_executable().then_some(path)
    }) {
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
