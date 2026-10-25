//! Modified Rust translation of the pinned V1 parser and renderer; Apache-2.0.
use serde::{Deserialize, Serialize};
pub const MAX_INPUT: usize = 1_048_576;
pub const MAX_OUTPUT: usize = 8_388_608;
pub const MAX_NODES: usize = 8192;
pub const MAX_DEPTH: usize = 256;
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub charset: String,
    pub trailing_dir_slash: bool,
    pub full_path: bool,
    pub root_dot: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            charset: "utf-8".into(),
            trailing_dir_slash: false,
            full_path: false,
            root_dot: true,
        }
    }
}
impl Options {
    pub fn form_defaults() -> Self {
        Self {
            trailing_dir_slash: true,
            root_dot: false,
            ..Self::default()
        }
    }
}
fn js_space(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}
struct Node<'a> {
    name: &'a str,
    indent: usize,
    parent: usize,
    children: Vec<usize>,
}
fn append(out: &mut String, text: &str) -> Result<(), String> {
    if out.len().saturating_add(text.len()) > MAX_OUTPUT {
        return Err("Output resource limit exceeded (8 MiB)".into());
    }
    out.push_str(text);
    Ok(())
}
/// Names are literal text, never filesystem paths. Duplicates and input order survive.
pub fn generate(input: &str, options: &Options) -> Result<String, String> {
    if input.len() > MAX_INPUT {
        return Err("Input resource limit exceeded (1 MiB)".into());
    }
    let (child, last, directory) = match options.charset.as_str() {
        "ascii" => ("|-- ", "`-- ", "|   "),
        "utf-8" => ("├── ", "└── ", "│   "),
        _ => return Err("Invalid charset".into()),
    };
    let mut nodes = vec![Node {
        name: ".",
        indent: 0,
        parent: 0,
        children: vec![],
    }];
    let mut path = vec![0];
    for line in input
        .split(['\r', '\n'])
        .filter(|s| !s.chars().all(js_space))
    {
        if nodes.len() > MAX_NODES {
            return Err("Node resource limit exceeded (8192)".into());
        }
        let whitespace = line.len() - line.trim_start_matches(js_space).len();
        let indent = line[..whitespace].encode_utf16().count();
        let mut name = &line[whitespace..];
        if let Some(tail) = name.strip_prefix('-') {
            if let Some(c) = tail.chars().next().filter(|c| js_space(*c)) {
                name = &tail[c.len_utf8()..];
            }
        }
        while path.len() > 1 && nodes[*path.last().expect("root")].indent >= indent {
            path.pop();
        }
        if path.len() > MAX_DEPTH {
            return Err("Depth resource limit exceeded (256)".into());
        }
        let parent = *path.last().expect("root");
        let index = nodes.len();
        nodes[parent].children.push(index);
        nodes.push(Node {
            name,
            indent,
            parent,
            children: vec![],
        });
        path.push(index);
    }
    let is_last = |i: usize| nodes[nodes[i].parent].children.last() == Some(&i);
    let mut output = String::new();
    if options.root_dot {
        append(&mut output, ".")?;
    }
    for i in 1..nodes.len() {
        if options.root_dot || i > 1 {
            append(&mut output, "\n")?;
        }
        let mut ancestors = Vec::new();
        let mut p = nodes[i].parent;
        while p != 0 {
            ancestors.push(p);
            p = nodes[p].parent;
        }
        // V1 removes the first four UTF-16 characters when rootDot is off.
        for &a in ancestors.iter().rev().skip(usize::from(!options.root_dot)) {
            append(&mut output, if is_last(a) { "    " } else { directory })?;
        }
        if options.root_dot || !ancestors.is_empty() {
            append(&mut output, if is_last(i) { last } else { child })?;
        }
        if options.full_path {
            append(&mut output, "./")?;
            for &a in ancestors.iter().rev() {
                append_name(&mut output, nodes[a].name, true)?;
            }
        }
        append_name(
            &mut output,
            nodes[i].name,
            options.trailing_dir_slash && !nodes[i].children.is_empty(),
        )?;
    }
    Ok(output)
}
fn append_name(out: &mut String, name: &str, slash: bool) -> Result<(), String> {
    append(out, name)?;
    if slash && !name.trim_end_matches(js_space).ends_with('/') {
        append(out, "/")?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn literal_duplicates_and_bullets() {
        assert_eq!(
            generate("- a\n  - b\na", &Options::form_defaults()).unwrap(),
            "a/\n└── b\na"
        );
    }
    #[test]
    fn library_and_ui_defaults_differ() {
        assert_eq!(generate("", &Options::default()).unwrap(), ".");
        assert_eq!(generate("", &Options::form_defaults()).unwrap(), "");
    }
    #[test]
    fn output_limit_is_an_error() {
        let mut s = String::new();
        for n in 0..180 {
            s.push_str(&" ".repeat(n));
            s.push_str(&"x".repeat(3000));
            s.push('\n');
        }
        assert!(generate(
            &s,
            &Options {
                full_path: true,
                ..Options::default()
            }
        )
        .unwrap_err()
        .contains("Output resource limit"));
    }
    #[test]
    fn explicit_limits() {
        assert!(generate(&"x".repeat(MAX_INPUT + 1), &Options::default()).is_err());
        assert!(generate(&"x\n".repeat(MAX_NODES + 1), &Options::default()).is_err());
        let mut s = String::new();
        for n in 0..258 {
            s.push_str(&" ".repeat(n));
            s.push_str("x\n");
        }
        assert!(generate(&s, &Options::default()).is_err());
    }
}
