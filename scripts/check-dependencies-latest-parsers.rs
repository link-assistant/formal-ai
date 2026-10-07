//! Manifest and lockfile parsers for `check-dependencies-latest.rs`.
//!
//! Included by the script with `#[path]` (and so also by the unit crate that
//! `#[path]`-includes the script). It lives in its own file because a
//! rust-script is a single file by construction and `scripts/check-file-size.rs`
//! caps a Rust file at 1000 lines; the text parsers carry no registry or
//! rewrite decisions, so they are the honest part to lift out.

use super::*;

/// Parse the manifest's dependency tables: `name = "req"`,
/// `name = { version = "req", .. }`; path/git/workspace entries are skipped.
pub fn parse_cargo_manifest(text: &str) -> Vec<(String, String, usize, Option<String>)> {
    let mut out = Vec::new();
    let mut section = Section::Outside;
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim_start();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            let header = line.trim_start_matches('[').trim_end_matches(']');
            section = if matches!(
                header,
                "dependencies" | "dev-dependencies" | "build-dependencies"
            ) {
                Section::Table
            } else if header.starts_with("dependencies.")
                || header.starts_with("dev-dependencies.")
                || header == "workspace.dependencies"
            {
                // The dependency's name is in the header; member lines are
                // its options, not dependencies themselves.
                Section::SubTable
            } else {
                Section::Outside
            };
            continue;
        }
        if section != Section::Table {
            continue;
        }
        let Some((name, value)) = split_once(line, '=') else {
            continue;
        };
        let name = name.trim().trim_matches('"').to_string();
        let value = value.trim();
        if value.contains("path =") || value.contains("git =") || value.contains("workspace = true")
        {
            continue;
        }
        let requirement = if let Some(version) = version_key_of_inline_table(value) {
            version
        } else if let Some(quoted) = quoted_value(value) {
            quoted
        } else {
            continue;
        };
        out.push((name, requirement, index + 1, blocked_annotation(raw)));
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Outside,
    Table,
    SubTable,
}

/// The `version = "…"` member of `{ version = "…", features = […] }`.
pub fn version_key_of_inline_table(value: &str) -> Option<String> {
    if !value.starts_with('{') {
        return None;
    }
    for part in split_top_level_commas(&value[1..value.len().saturating_sub(1)]) {
        if let Some((key, item)) = split_once(&part, '=')
            && key.trim() == "version"
        {
            return quoted_value(item.trim());
        }
    }
    None
}

pub fn split_top_level_commas(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth: usize = 0;
    let mut current = String::new();
    for character in text.chars() {
        match character {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(character);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            ',' if depth == 0 => {
                parts.push(current.clone());
                current.clear();
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

/// `[[package]] name/version` pairs from a Cargo.lock.
pub fn parse_cargo_lock(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut name = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("name = ") {
            name = quoted_value(value).filter(|_| !line.starts_with('#'));
        } else if let Some(value) = line.strip_prefix("version = ")
            && let (Some(name), Some(version)) = (name.take(), quoted_value(value))
        {
            out.insert(name, version);
        }
    }
    out
}

/// Dependency-object members of a package.json, with `"name//"` blocked notes.
pub fn parse_package_json(text: &str) -> Vec<(String, String, usize, Option<String>)> {
    let mut blocked: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let Some((key, value)) = json_member(line) else {
            continue;
        };
        if let Some(name) = key.strip_suffix("//")
            && let Some(url) = json_string(value)
        {
            blocked.insert(
                name.to_string(),
                url.trim_start_matches("blocked:").trim().to_string(),
            );
        }
    }
    let mut out = Vec::new();
    let mut in_dependencies = false;
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        // A section header opens an object on the same line: `"key": {`.
        if trimmed.starts_with('"') && trimmed.ends_with('{') {
            if let Some((key, _)) = json_member(trimmed.trim_end_matches('{').trim_end()) {
                in_dependencies = matches!(
                    key.as_str(),
                    "dependencies" | "devDependencies" | "optionalDependencies"
                );
            }
            continue;
        }
        if !in_dependencies {
            continue;
        }
        let Some((key, value)) = json_member(line) else {
            continue;
        };
        if key.ends_with("//") {
            continue;
        }
        let Some(range) = json_string(value) else {
            continue;
        };
        out.push((key.clone(), range, index + 1, blocked.get(&key).cloned()));
    }
    out
}

/// A `"key": value` member on one line, when present.
pub fn json_member(line: &str) -> Option<(String, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('"')?;
    let end = rest.find('"')?;
    let value = &rest[end + 1..];
    let colon = value.find(':')?;
    Some((rest[..end].to_string(), value[colon + 1..].trim()))
}

pub fn json_string(value: &str) -> Option<String> {
    quoted_value(value.trim_end_matches(',').trim())
}

/// The hoisted resolutions of a package-lock v3 `packages` map.
pub fn parse_package_lock(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(version) = trimmed.strip_prefix("\"version\": ") {
            if let (Some(name), Some(version)) =
                (current.take(), quoted_value(version.trim_end_matches(',')))
            {
                out.insert(name, version);
            }
        } else if trimmed.starts_with("\"node_modules/") && trimmed.ends_with("\": {") {
            let path = trimmed
                .trim_start_matches("\"node_modules/")
                .trim_end_matches("\": {");
            // Hoisted entries only: nested duplicates belong to their parent.
            if !path.contains("/node_modules/") {
                current = Some(path.to_string());
            }
        }
    }
    out
}

/// The `packages` entries of a bun.lock: `"name": ["name@version", …`.
pub fn parse_bun_lock(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix('"') else {
            continue;
        };
        let Some(key_end) = rest.find("\": ") else {
            continue;
        };
        let name = &rest[..key_end];
        let after = rest[key_end + 3..].trim_start();
        let Some(array_first) = after.strip_prefix("[\"") else {
            continue;
        };
        let Some(spec_end) = array_first.find('"') else {
            continue;
        };
        let spec = &array_first[..spec_end];
        // The first array element repeats the name with its version attached.
        if let Some(version) = spec.strip_prefix(&format!("{name}@")) {
            out.insert(name.to_string(), version.to_string());
        }
    }
    out
}

/// `uses: owner/repo@ref` pins, local (`./…`) and docker refs excluded.
pub fn parse_uses_pins(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let step = line.trim_start();
        let step = step.strip_prefix("- ").map_or(step, str::trim_start);
        let Some(rest) = step.strip_prefix("uses:") else {
            continue;
        };
        let reference = rest.trim();
        if reference.starts_with("./") || reference.starts_with("docker://") {
            continue;
        }
        let Some((action, pin)) = split_once(reference, '@') else {
            continue;
        };
        if action.split('/').count() < 2 || blocked_annotation(line).is_some() {
            continue;
        }
        out.push((action.to_string(), pin.to_string(), index + 1));
    }
    out
}

/// `FROM image:tag` bases, `${VAR}` stages and `scratch` excluded.
pub fn parse_dockerfile_from(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("from ") else {
            continue;
        };
        let first = rest.split_whitespace().next().unwrap_or_default();
        if first.is_empty() || first.starts_with("${") || first == "scratch" {
            continue;
        }
        let (image, tag) = match split_once(first, ':') {
            Some((image, tag)) => (image.to_string(), tag.to_string()),
            None => (first.to_string(), "latest".to_string()),
        };
        if blocked_annotation(line).is_some() {
            continue;
        }
        out.push((image, tag, index + 1));
    }
    out
}

/// The issue URL of a `# blocked: <url>` annotation on this line.
pub fn blocked_annotation(line: &str) -> Option<String> {
    let marker = "# blocked: ";
    let start = line.find(marker)?;
    // The issue URL is the first token; prose after it explains the hold-back.
    let url = line[start + marker.len()..]
        .split_whitespace()
        .next()
        .unwrap_or_default();
    if url.starts_with("https://") && url.contains("/issues/") {
        Some(url.to_string())
    } else {
        None
    }
}

pub fn quoted_value(text: &str) -> Option<String> {
    let start = text.find('"')?;
    let rest = &text[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn split_once(text: &str, separator: char) -> Option<(&str, &str)> {
    let index = text.find(separator)?;
    Some((&text[..index], &text[index + separator.len_utf8()..]))
}

/// `^1.2.3` -> `1.2.3`, `>=2.0.0` -> `2.0.0`, `=0.3.0` -> `0.3.0`.
pub fn base_version(range: &str) -> &str {
    let mut base = range;
    for operator in ["^", "~", ">=", "<=", ">", "=", "=="] {
        if let Some(stripped) = base.strip_prefix(operator) {
            base = stripped;
            break;
        }
    }
    base.trim()
}

pub fn without_v(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

pub fn minor_of(version: &str) -> String {
    let parts: Vec<&str> = without_v(version).split('.').collect();
    parts.iter().take(2).copied().collect::<Vec<_>>().join(".")
}
