/// Value of `name="..."` in a tag's attribute text, if present.
pub(super) fn tag_attr(inner_tag: &str, name: &str) -> Option<String> {
    let lower = inner_tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let at = lower.find(&needle)?;
    let rest = &inner_tag[at + needle.len()..];
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let end = rest[1..].find(quote)? + 1;
        Some(rest[1..end].to_owned())
    } else {
        Some(
            rest.split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned(),
        )
    }
}

/// The text of an HTML fragment with every tag removed, whitespace kept.
fn remove_tags(fragment: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for character in fragment.chars() {
        match character {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(character),
            _ => {}
        }
    }
    out
}

/// Strip every tag from an HTML fragment and collapse its whitespace.
pub(super) fn strip_tags(fragment: &str) -> String {
    remove_tags(fragment)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Strip tags from a code block and keep it code (issue #1165 R1165-1).
///
/// Unlike prose, a code block's whitespace is meaning: every line keeps its
/// indentation and inner spacing (a highlighter's `<span>` around leading
/// spaces included), and only trailing whitespace goes. The blank lines that
/// open and close the block are the page's markup, not the program, and so
/// is the indentation every line shares; both are removed.
pub(crate) fn strip_tags_keep_lines(fragment: &str) -> String {
    let text = remove_tags(fragment);
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    let (Some(first), Some(last)) = (
        lines.iter().position(|line| !line.is_empty()),
        lines.iter().rposition(|line| !line.is_empty()),
    ) else {
        return String::new();
    };
    let body = &lines[first..=last];
    let mut shared: Option<&str> = None;
    for line in body.iter().copied().filter(|line| !line.is_empty()) {
        let indent = &line[..line.len() - line.trim_start().len()];
        shared = Some(shared.map_or(indent, |previous| {
            let common = previous
                .char_indices()
                .zip(indent.chars())
                .find(|((_, left), right)| left != right)
                .map_or_else(|| previous.len().min(indent.len()), |((at, _), _)| at);
            &previous[..common]
        }));
    }
    let shared = shared.unwrap_or_default();
    body.iter()
        .copied()
        .map(|line| line.strip_prefix(shared).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Text of each `<td>`/`<th>` cell in a table row.
pub(super) fn cell_texts(row: &str) -> Vec<String> {
    let lower = row.to_ascii_lowercase();
    let mut cells = Vec::new();
    let mut cursor = 0usize;
    while let Some(rel) = lower[cursor..].find("<t") {
        let open = cursor + rel;
        let Some(tag_end) = lower[open..].find('>') else {
            break;
        };
        let tag_end = open + tag_end;
        let name = lower[open + 1..tag_end]
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_owned();
        if (name == "td" || name == "th")
            && let Some(close) = lower[tag_end + 1..].find("</t")
        {
            cells.push(strip_tags(&row[tag_end + 1..tag_end + 1 + close]));
        }
        cursor = tag_end + 1;
    }
    cells
}

/// Decode the handful of entities real pages use.
pub(super) fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}
