use anyhow::{bail, Result};
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

use crate::context::Context;

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
        patch_node(ctx, cur.node())?;
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

fn patch_node(ctx: &mut Context, node: Node) -> Result<()> {
    let (range, name) = if let Some(x) = parse_literal(&ctx.src, &node) {
        x
    } else {
        return Ok(());
    };

    if name == "exec" {
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

    let path = PathBuf::from(name);
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

fn parse_literal(src: &[u8], node: &Node) -> Option<(Range<usize>, String)> {
    let range = node.byte_range();
    match node.kind() {
        "raw_string" => {
            let content = &src[range.start + 1 .. range.end - 1];
            Some((range, String::from_utf8(content.into()).ok()?))
        }

        "string" => {
            let content = str::from_utf8(&src[range.start + 1 .. range.end - 1]).ok()?;
            eprintln!("{}", content);
            let mut chars = content.chars();
            let mut result = String::with_capacity(content.len());

            while let Some(c) = chars.next() {
                match c {
                    '\\' => {
                        let c = chars.next()?;
                        if !matches!(c, '$' | '`' | '"' | '\\') {
                            result.push('\\');
                        }
                        result.push(c);
                    }
                    '$' => result.push(chars.next().is_none().then_some('$')?),
                    _ => result.push(c),
                }
            }
            eprintln!("{}", result);

            Some((range, result))
        }

        "word" => {
            let content = str::from_utf8(&src[range.clone()]).ok()?;
            let mut chars = content.chars();
            let mut result = String::with_capacity(content.len());

            while let Some(c) = chars.next() {
                result.push(match c {
                    '\\' => chars.next()?,
                    '$' => chars.next().is_none().then_some('$')?,
                    c => c,
                });
            }

            Some((range, result))
        }

        _ => None,
    }
}
