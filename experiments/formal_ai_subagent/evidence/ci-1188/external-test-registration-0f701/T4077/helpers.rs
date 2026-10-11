#[derive(Debug)]
struct SourceToken<'a> {
    text: &'a str,
    start: usize,
    end: usize,
    depth: usize,
}

// Comments and literals are atomic: their braces and test-shaped text cannot
// establish an item boundary or a registration's structural depth.
fn source_tokens(source: &str) -> Option<Vec<SourceToken<'_>>> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut delimiters = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if source[index..].starts_with("//") {
            index = source[index..].find('\n').map_or(bytes.len(), |n| index + n + 1);
            continue;
        }
        if source[index..].starts_with("/*") {
            index += 2;
            let mut comments = 1;
            while comments > 0 {
                if index >= bytes.len() { return None; }
                if source[index..].starts_with("/*") { comments += 1; index += 2; }
                else if source[index..].starts_with("*/") { comments -= 1; index += 2; }
                else { index += source[index..].chars().next()?.len_utf8(); }
            }
            continue;
        }
        let start = index;
        let depth = delimiters.len();
        let raw_start = if source[index..].starts_with("br") { index + 2 }
            else if bytes[index] == b'r' { index + 1 } else { index };
        let mut raw_quote = raw_start;
        while raw_quote < bytes.len() && bytes[raw_quote] == b'#' { raw_quote += 1; }
        if raw_start > index && bytes.get(raw_quote) == Some(&b'"') {
            let closing = format!("\"{}", "#".repeat(raw_quote - raw_start));
            index = raw_quote + 1;
            index += source[index..].find(&closing)? + closing.len();
        } else if bytes[index] == b'"' {
            index += 1;
            loop {
                match bytes.get(index)? {
                    b'\\' => { index += 1; index += source[index..].chars().next()?.len_utf8(); }
                    b'"' => { index += 1; break; }
                    _ => index += source[index..].chars().next()?.len_utf8(),
                }
            }
        } else if bytes[index] == b'\'' {
            // A lifetime has no closing quote. A character (including escaped
            // punctuation) is consumed atomically rather than as a delimiter.
            let mut end = index + 1;
            if bytes.get(end) == Some(&b'\\') {
                end += 1;
                if bytes.get(end) == Some(&b'u') && bytes.get(end + 1) == Some(&b'{') {
                    end += source[end..].find('}')? + 1;
                } else if bytes.get(end) == Some(&b'x') { end += 3; }
                else { end += source[end..].chars().next()?.len_utf8(); }
            } else { end += source[end..].chars().next()?.len_utf8(); }
            index = if bytes.get(end) == Some(&b'\'') { end + 1 } else { index + 1 };
        } else {
            let first = source[index..].chars().next()?;
            index += first.len_utf8();
            if first == '_' || first.is_alphanumeric() {
                while index < bytes.len() {
                    let next = source[index..].chars().next()?;
                    if next != '_' && !next.is_alphanumeric() { break; }
                    index += next.len_utf8();
                }
            } else if matches!(first, '(' | '[' | '{') { delimiters.push(first); }
            else if matches!(first, ')' | ']' | '}') {
                let expected = match first { ')' => '(', ']' => '[', _ => '{' };
                if delimiters.pop()? != expected { return None; }
            }
        }
        tokens.push(SourceToken { text: &source[start..index], start, end: index, depth });
    }
    delimiters.is_empty().then_some(tokens)
}

fn external_test_file(crate_root: &Path, source_path: &Path, literal: &str) -> Option<(PathBuf, String)> {
    use sha2::{Digest, Sha256};
    use std::path::Component;

    let relative = literal.strip_prefix('"')?.strip_suffix('"')?;
    if relative.is_empty() || relative.contains('\\')
        || relative.split('/').any(|part| part.is_empty() || part == ".") { return None; }
    let crate_root = crate_root.canonicalize().ok()?;
    let tests_root = crate_root.join("tests");
    let mut target = source_path.parent()?.canonicalize().ok()?;
    if !target.starts_with(&crate_root) { return None; }
    let mut entered_path = false;
    for component in Path::new(relative).components() {
        match component {
            Component::ParentDir if !entered_path && target != crate_root => { target.pop(); }
            Component::Normal(part) => {
                entered_path = true;
                target.push(part);
                if fs::symlink_metadata(&target).ok()?.file_type().is_symlink() { return None; }
            }
            _ => return None,
        }
    }
    if !target.starts_with(&tests_root) || target.extension()?.to_str()? != "rs"
        || !fs::symlink_metadata(&tests_root).ok()?.is_dir()
        || fs::symlink_metadata(&tests_root).ok()?.file_type().is_symlink()
        || !fs::symlink_metadata(&target).ok()?.is_file()
        || target.canonicalize().ok()? != target {
        return None;
    }
    let content = fs::read(&target).ok()?;
    let digest = format!("{:x}", Sha256::digest(&content));
    if format!("{:x}", Sha256::digest(fs::read(&target).ok()?)) != digest { return None; }
    Some((target, digest))
}

fn external_test_registrations(crate_root: &Path, source_path: &Path, source: &str) -> Vec<(usize, usize, PathBuf, String)> {
    let Some(tokens) = source_tokens(source) else { return Vec::new(); };
    let mut registrations = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.depth != 0 || (index > 0 && !matches!(tokens[index - 1].text, ";" | "}")) { continue; }
        let Some(item) = tokens.get(index..index + 16) else { continue; };
        let fixed = [(0,"#"),(1,"["),(2,"cfg"),(3,"("),(4,"test"),(5,")"),(6,"]"),
            (7,"#"),(8,"["),(9,"path"),(10,"="),(12,"]"),(13,"mod"),(15,";")];
        if fixed.iter().any(|(offset, expected)| item[*offset].text != *expected) { continue; }
        let name = item[14].text;
        if name.is_empty() || !name.chars().next().is_some_and(|c| c == '_' || c.is_alphabetic())
            || !name.chars().all(|c| c == '_' || c.is_alphanumeric()) { continue; }
        let Some((path, digest)) = external_test_file(crate_root, source_path, item[11].text) else { continue; };
        registrations.push((token.start, item[15].end, path, digest));
    }
    registrations
}
