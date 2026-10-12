//! Shared lexical candidate extraction for issue #1182 audits.
//! This is a conservative brace/token scan, not a replacement for the planned
//! meta-language syntax census. Named Rust and JavaScript functions are covered;
//! closures, methods without a keyword, regex literals and template interpolations
//! require the future grammar-backed scan.

fn identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Keep offsets and newline counts while removing comments, and separately mask
/// quoted literals so their braces and keyword-like text cannot become code.
fn lexical(source: &str) -> (Vec<char>, Vec<char>) {
    let mut clean: Vec<char> = source.chars().collect();
    let mut mask = clean.clone();
    let mut i = 0;
    while i < clean.len() {
        if clean[i] == '/' && clean.get(i + 1) == Some(&'/') {
            let start = i;
            while i < clean.len() && clean[i] != '\n' {
                i += 1;
            }
            for j in start..i {
                clean[j] = ' ';
                mask[j] = ' ';
            }
        } else if clean[i] == '/' && clean.get(i + 1) == Some(&'*') {
            let start = i;
            i += 2;
            let mut depth = 1;
            while i < clean.len() && depth > 0 {
                if clean[i] == '/' && clean.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if clean[i] == '*' && clean.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            for j in start..i {
                if clean[j] != '\n' {
                    clean[j] = ' ';
                    mask[j] = ' ';
                }
            }
        } else {
            let start = i;
            let mut quote_at = i;
            let mut hashes = 0;
            if clean[i] == 'r' {
                quote_at += 1;
                while clean.get(quote_at) == Some(&'#') {
                    hashes += 1;
                    quote_at += 1;
                }
            }
            let raw = quote_at > i && clean.get(quote_at) == Some(&'"');
            let quote = if raw { '"' } else { clean[i] };
            let character_literal = quote == '\''
                && (clean.get(i + 2) == Some(&'\'')
                    || (clean.get(i + 1) == Some(&'\\')
                        && (i + 3..(i + 12).min(clean.len())).any(|j| clean[j] == '\'')));
            if raw || quote == '"' || quote == '`' || character_literal {
                i = if raw { quote_at + 1 } else { i + 1 };
                while i < clean.len() {
                    if !raw && clean[i] == '\\' {
                        i = (i + 2).min(clean.len());
                        continue;
                    }
                    if clean[i] == quote
                        && (!raw || (1..=hashes).all(|n| clean.get(i + n) == Some(&'#')))
                    {
                        i = (i + 1 + if raw { hashes } else { 0 }).min(clean.len());
                        break;
                    }
                    i += 1;
                }
                for j in start..i {
                    if mask[j] != '\n' {
                        mask[j] = '#';
                    }
                }
            } else {
                i += 1;
            }
        }
    }
    (clean, mask)
}

pub fn strip_comments(source: &str) -> String {
    lexical(source).0.into_iter().collect()
}

/// Preserve string whitespace while dropping insignificant code whitespace.
pub fn normalize(source: &str) -> String {
    let (clean, mask) = lexical(source);
    clean
        .iter()
        .zip(mask)
        .filter_map(|(&c, masked)| (!c.is_whitespace() || masked == '#').then_some(c))
        .collect()
}

pub fn functions(source: &str) -> Vec<(String, String, usize, usize)> {
    let (clean, mask) = lexical(source);
    let mut result = Vec::new();
    let mut i = 0;
    while i < mask.len() {
        let keyword = ["fn", "function"].into_iter().find(|word| {
            let end = i + word.len();
            end < mask.len()
                && (i == 0 || !identifier(mask[i - 1]))
                && mask[i..end].iter().copied().eq(word.chars())
                && mask[end].is_whitespace()
        });
        let Some(keyword) = keyword else {
            i += 1;
            continue;
        };
        let start = i;
        i += keyword.len();
        while i < mask.len() && mask[i].is_whitespace() {
            i += 1;
        }
        let name_start = i;
        while i < mask.len() && identifier(mask[i]) {
            i += 1;
        }
        if name_start == i {
            continue;
        }
        let name: String = clean[name_start..i].iter().collect();
        let mut scan = i;
        while scan < mask.len() && mask[scan] != '{' && mask[scan] != ';' {
            scan += 1;
        }
        if scan == mask.len() || mask[scan] == ';' {
            continue;
        }
        let open = scan;
        let mut depth = 1;
        scan += 1;
        while scan < mask.len() && depth > 0 {
            match mask[scan] {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
            scan += 1;
        }
        if depth != 0 {
            continue;
        }
        let body: String = clean[open + 1..scan - 1].iter().collect();
        let normalized = normalize(&body);
        let lines = body.matches('\n').count() + 1;
        let start_line = clean[..start].iter().filter(|&&c| c == '\n').count() + 1;
        result.push((name, normalized, lines, start_line));
        i = scan;
    }
    result
}
