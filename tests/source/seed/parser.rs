//! Tiny Links Notation parser shared by every `seed` loader.
//!
//! Links Notation files in this repo are shallow trees of `name "value"`
//! lines with two-space indentation. The parser intentionally stays small:
//! no comments, no arbitrary escapes — only `\"`, `\n`, and `\\`.

#[derive(Debug, Default, Clone)]
pub struct LinoNode {
    pub name: String,
    pub id: String,
    pub children: Vec<Self>,
}

impl LinoNode {
    pub fn find_child_value(&self, name: &str) -> &str {
        for child in &self.children {
            if child.name == name {
                return &child.id;
            }
        }
        ""
    }
}

pub fn parse_lino(text: &str) -> LinoNode {
    let mut root = LinoNode::default();
    let mut stack: Vec<(Option<usize>, Vec<usize>)> = vec![(None, Vec::new())];
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.chars().take_while(|c| *c == ' ').count();
        let content = &line[indent..];
        let node = parse_lino_line(content);
        while stack.len() > 1
            && stack
                .last()
                .and_then(|s| s.0)
                .is_some_and(|top| top >= indent)
        {
            stack.pop();
        }
        let parent_path = stack.last().map(|s| s.1.clone()).unwrap_or_default();
        let parent = navigate_mut(&mut root, &parent_path);
        parent.children.push(node);
        let new_index = parent.children.len() - 1;
        let mut new_path = parent_path;
        new_path.push(new_index);
        stack.push((Some(indent), new_path));
    }
    root
}

fn navigate_mut<'a>(root: &'a mut LinoNode, path: &[usize]) -> &'a mut LinoNode {
    let mut current = root;
    for &index in path {
        current = &mut current.children[index];
    }
    current
}

fn parse_lino_line(content: &str) -> LinoNode {
    let mut node = LinoNode::default();
    if let Some(quote_start) = content.find(" \"") {
        node.name = content[..quote_start].trim().to_string();
        let rest = &content[quote_start + 2..];
        if let Some(close) = find_closing_quote(rest) {
            node.id = unescape_value(&rest[..close]);
        }
    } else {
        node.name = content.trim().to_string();
    }
    node
}

pub fn find_closing_quote(rest: &str) -> Option<usize> {
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub fn unescape_value(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut iter = raw.chars();
    while let Some(c) = iter.next() {
        if c == '\\' {
            match iter.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn escape_value(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn parse_codepoint(value: &str) -> u32 {
    let trimmed = value.trim();
    trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .map_or_else(
            || trimmed.parse::<u32>().unwrap_or(0),
            |stripped| u32::from_str_radix(stripped, 16).unwrap_or(0),
        )
}

pub fn split_pipe_list(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
