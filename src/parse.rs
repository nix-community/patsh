use tree_sitter::Node;

use std::{ops::Range, os::unix::prelude::OsStrExt, path::PathBuf, str};

use crate::{
    patch::{add_patch, get_patch_index},
    Context,
};

enum MultipleCommands {
    Always,
    Never,
    Flags { short: &'static [char] },
}

pub(crate) fn parse_command(ctx: &mut Context, node: &Node) -> (bool, Vec<(Range<usize>, String)>) {
    let Some((range, name)) = parse_literal(&ctx.src, node) else {
        return Default::default();
    };

    match name.as_str() {
        "command" => (
            false,
            parse_exec(
                &ctx.src,
                node,
                &[],
                &[],
                MultipleCommands::Flags { short: &['v', 'V'] },
            ),
        ),

        "exec" => (
            true,
            parse_exec(&ctx.src, node, &['a'], &[], MultipleCommands::Never),
        ),

        "type" => (
            false,
            parse_exec(&ctx.src, node, &[], &[], MultipleCommands::Always),
        ),

        _ => match PathBuf::from(&name).file_name().map(OsStrExt::as_bytes) {
            Some(b"doas") => {
                if let Some(idx) = get_patch_index(&ctx.patches, &range) {
                    add_patch(&mut ctx.patches, idx, range, "doas".into());
                    (
                        true,
                        parse_exec(&ctx.src, node, &['C', 'u'], &[], MultipleCommands::Never),
                    )
                } else {
                    Default::default()
                }
            }

            Some(b"sudo") => {
                if let Some(idx) = get_patch_index(&ctx.patches, &range) {
                    add_patch(&mut ctx.patches, idx, range, "sudo".into());
                    (
                        true,
                        parse_exec(
                            &ctx.src,
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
                            MultipleCommands::Never,
                        ),
                    )
                } else {
                    Default::default()
                }
            }

            Some(_) => (false, vec![(range, name)]),

            None => Default::default(),
        },
    }
}

fn parse_exec(
    src: &[u8],
    node: &Node,
    short: &[char],
    long: &[&str],
    multi: MultipleCommands,
) -> Vec<(Range<usize>, String)> {
    let cur = &mut node.walk();
    let Some(node) = node.parent().and_then(|node| node.parent()) else { return Vec::new()};
    let mut args = node.children_by_field_name("argument", cur);

    let mut multiple = matches!(multi, MultipleCommands::Always);
    let arg = loop {
        let Some(arg) = args.next() else { return Vec::new()};
        let Some((range, arg)) = parse_literal(src, &arg) else { continue; };
        let mut chars = arg.chars();

        match chars.next() {
            Some('-') => match chars.next() {
                Some('-') => {
                    if chars.next().is_none() {
                        return args.filter_map(|arg| parse_literal(src, &arg)).collect();
                    } else {
                        if long.contains(&&arg[2 ..]) {
                            args.next();
                        }
                        continue;
                    }
                }

                Some(c) => {
                    let mut chars = Some(c).into_iter().chain(chars);
                    while let Some(c) = chars.next() {
                        if short.contains(&c) {
                            if chars.next().is_none() {
                                args.next();
                            }
                            break;
                        }
                        multiple = multiple
                            || matches!(multi, MultipleCommands::Flags { short } if short.contains(&c));
                    }
                    continue;
                }

                None if multiple => {
                    break (range, arg);
                }

                None => return vec![(range, arg)],
            },

            Some(_) if multiple => {
                break (range, arg);
            }

            Some(_) => return vec![(range, arg)],

            None => continue,
        }
    };

    Some(arg)
        .into_iter()
        .chain(args.filter_map(|arg| parse_literal(src, &arg)))
        .collect()
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
