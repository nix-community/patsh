use tree_sitter::Node;

use std::{ops::Range, os::unix::prelude::OsStrExt, path::PathBuf, str};

pub(crate) fn parse_command(src: &[u8], node: &Node) -> Option<(Range<usize>, String)> {
    let (range, name) = parse_literal(src, node)?;
    match name.as_str() {
        "command" => parse_exec(src, node, &[], &[]),
        "exec" => parse_exec(src, node, &['a'], &[]),
        "type" => parse_exec(src, node, &[], &[]),
        _ => match PathBuf::from(&name).file_name().map(OsStrExt::as_bytes) {
            Some(b"doas") => parse_exec(src, node, &['C', 'u'], &[]),
            Some(b"sudo") => parse_exec(
                src,
                node,
                &['C', 'D', 'g', 'h', 'p', 'R', 'U', 'T', 'u'],
                &[
                    "close-from",
                    "chdir",
                    "group",
                    "host",
                    "prompt",
                    "chroot",
                    "other-user",
                    "command-timeout",
                    "user",
                ],
            ),
            Some(_) => Some((range, name)),
            None => None,
        },
    }
}

fn parse_exec(
    src: &[u8],
    node: &Node,
    short: &[char],
    long: &[&str],
) -> Option<(Range<usize>, String)> {
    let cur = &mut node.walk();
    let mut args = node
        .parent()?
        .parent()?
        .children_by_field_name("argument", cur);

    while let Some(arg) = args.next() {
        let (range, arg) = parse_literal(src, &arg)?;
        let mut chars = arg.chars();

        match chars.next()? {
            '-' => match chars.next() {
                Some('-') => {
                    if chars.next().is_none() {
                        return parse_literal(src, &args.next()?);
                    } else {
                        if long.contains(&&arg[2 ..]) {
                            args.next()?;
                        }
                        continue;
                    }
                }

                Some(c) => {
                    if short.contains(&chars.last().unwrap_or(c)) {
                        args.next()?;
                    }
                    continue;
                }

                None => return Some((range, arg)),
            },

            _ => return Some((range, arg)),
        }
    }

    None
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
