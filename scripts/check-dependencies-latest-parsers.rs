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
        // The note must open with `blocked:` and name its tracking reference;
        // a note without one is prose, not a hold-back decision (R1169-3).
        if let Some(name) = key.strip_suffix("//")
            && let Some(note) = json_string(value)
            && let Some(reason) = note.trim().strip_prefix("blocked:")
            && let Some(url) = tracking_reference(reason)
        {
            blocked.insert(name.to_string(), url);
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

/// `FROM image:tag` bases; `--platform=` flags are skipped, and `${VAR}`
/// stages, `scratch` and earlier multi-stage aliases (`FROM builder` after
/// `FROM rust:1 AS builder`) are excluded because no registry publishes them.
pub fn parse_dockerfile_from(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    let mut stages: Vec<String> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("from ") else {
            continue;
        };
        let words: Vec<&str> = rest
            .split_whitespace()
            .take_while(|w| !w.starts_with('#'))
            .collect();
        if let Some(alias) = words
            .iter()
            .position(|w| *w == "as")
            .and_then(|i| words.get(i + 1))
        {
            stages.push((*alias).to_string());
        }
        let first = words
            .iter()
            .find(|w| !w.starts_with("--"))
            .copied()
            .unwrap_or_default();
        if first.is_empty()
            || first.starts_with("${")
            || first == "scratch"
            || stages.iter().any(|stage| stage == first)
        {
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
    let marker = "# blocked:";
    let start = line.find(marker)?;
    tracking_reference(&line[start + marker.len()..])
}

/// The tracking reference that opens a hold-back reason: its first token,
/// with optional `<…>` brackets, when it is an `https://` URL naming an issue
/// (`…/issues/<n>`) or a published security advisory (`…/advisories/GHSA-…`,
/// the record that tracks an unpatched vulnerability until a fix ships).
/// Prose after the reference explains the hold-back; a reason without such a
/// reference is not a decision, only drift (R1169-3).
pub fn tracking_reference(reason: &str) -> Option<String> {
    let token = reason.split_whitespace().next()?;
    let url = token
        .trim_start_matches('<')
        .trim_end_matches(['>', ',', ';']);
    let tracked = url.starts_with("https://")
        && (url.contains("/issues/") || url.contains("/advisories/GHSA-"));
    tracked.then(|| url.to_string())
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

/// The `owner/repo` a `uses:` action is released from: a sub-path action
/// (`github/codeql-action/init`) is versioned by its repository.
pub fn action_repository(action: &str) -> String {
    action.split('/').take(2).collect::<Vec<_>>().join("/")
}

/// A `uses:` ref that names a moving branch or tool selector rather than a
/// release (`dtolnay/rust-toolchain@stable`, `taiki-e/install-action@nextest`):
/// it has no release to fall behind, so the gate does not compare it. A
/// commit SHA is a pin, not a moving ref.
pub fn is_moving_ref(reference: &str) -> bool {
    !is_commit_sha(reference)
        && !without_v(reference)
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
}

/// A full 40-hex commit SHA.
pub fn is_commit_sha(reference: &str) -> bool {
    reference.len() == 40 && reference.chars().all(|c| c.is_ascii_hexdigit())
}

/// Whether `resolved` already selects `latest`: equal, or a floating prefix
/// (`v7` floats to the newest `7.x.y`, so it is current when `latest` is
/// `v7.0.1`).
pub fn selects_latest(resolved: &str, latest: &str) -> bool {
    let resolved: Vec<&str> = without_v(resolved).split('.').collect();
    let latest: Vec<&str> = without_v(latest).split('.').collect();
    resolved.len() <= latest.len() && resolved.iter().zip(&latest).all(|(r, l)| r == l)
}

/// `latest` cut to the precision of `resolved`, keeping a `v` prefix when
/// `latest` carries one: a floating `@v6` moves to `@v7`, not to `@v7.0.1`.
pub fn same_precision(latest: &str, resolved: &str) -> String {
    let components = without_v(resolved).split('.').count();
    let bare: Vec<&str> = without_v(latest).split('.').take(components).collect();
    let prefix = if latest.starts_with('v') { "v" } else { "" };
    format!("{prefix}{}", bare.join("."))
}

/// The string value of `"key": "value"` anywhere in a JSON body, whatever
/// whitespace the publisher puts around the colon: crates.io and npm answer
/// compact JSON, api.github.com pretty-prints it (`"tag_name": "v7.0.1"`).
pub fn json_field(body: &str, key: &str) -> Option<String> {
    json_field_at(body, key).map(|(value, _)| value)
}

/// [`json_field`] plus the byte offset just past the value's closing quote,
/// so a caller can walk every occurrence of a key in order.
pub fn json_field_at(body: &str, key: &str) -> Option<(String, usize)> {
    let needle = format!("\"{key}\"");
    let mut offset = 0;
    while let Some(found) = body[offset..].find(&needle) {
        let key_end = offset + found + needle.len();
        let after = &body[key_end..];
        let after_trimmed = after.trim_start();
        if let Some(value) = after_trimmed.strip_prefix(':') {
            let value_trimmed = value.trim_start();
            if let Some(string) = value_trimmed.strip_prefix('"') {
                let end = string.find('"')?;
                let string_start = body.len() - string.len();
                return Some((string[..end].to_string(), string_start + end + 1));
            }
        }
        offset = key_end;
    }
    None
}

/// crates.io `/api/v1/crates/{name}`: `crate.max_stable_version`.
pub fn crates_io_latest(body: &str) -> Option<String> {
    json_field(body, "max_stable_version")
}

/// registry.npmjs.org `/{name}`: `dist-tags.latest`.
pub fn npm_latest(body: &str) -> Option<String> {
    let tags = body.find("\"dist-tags\"")?;
    json_field(&body[tags..], "latest")
}

/// api.github.com `/repos/{owner}/{repo}/releases/latest`: `tag_name`.
pub fn github_latest(body: &str) -> Option<String> {
    json_field(body, "tag_name")
}

/// Docker Hub `/v2/repositories/{image}/tags`: the highest dotted-numeric tag
/// among the listed results (`latest`, suffixed variants and a rebuilt old
/// tag sorted first by push time do not win).
pub fn docker_hub_latest(body: &str) -> Option<String> {
    let results = body.find("\"results\"")?;
    let mut rest = &body[results..];
    let mut best: Option<(Vec<u64>, String)> = None;
    while let Some((tag, end)) = json_field_at(rest, "name") {
        rest = &rest[end..];
        let numbers: Option<Vec<u64>> = without_v(&tag)
            .split('.')
            .map(|part| part.parse::<u64>().ok())
            .collect();
        if let Some(numbers) = numbers
            && best.as_ref().is_none_or(|(current, _)| numbers > *current)
        {
            best = Some((numbers, tag));
        }
    }
    best.map(|(_, tag)| tag)
}

/// static.rust-lang.org `channel-rust-stable.toml`: the `[pkg.rust]`
/// `version = "1.90.0 (1159e78c4 2025-09-14)"`, first word.
pub fn rust_channel_latest(body: &str) -> Option<String> {
    let mut section = "";
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            section = trimmed;
        } else if section == "[pkg.rust]"
            && let Some(rest) = trimmed.strip_prefix("version = ")
        {
            return quoted_value(rest)?
                .split_whitespace()
                .next()
                .map(str::to_string);
        }
    }
    None
}

/// Leading dotted-numeric components of a version: `v1.98-slim` is `[1, 98]`,
/// `26.17.0` is `[26, 17, 0]`, and a part with a suffix ends the prefix.
pub fn numeric_prefix(version: &str) -> Vec<u64> {
    let mut numbers = Vec::new();
    for part in without_v(version).split('.') {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        let Ok(number) = digits.parse::<u64>() else {
            break;
        };
        numbers.push(number);
        if digits.len() != part.len() {
            break;
        }
    }
    numbers
}

/// Whether `resolved` is behind `latest` at the precision both state: a pin
/// newer than the registry's `latest` tag (a dist-tag lagging a release) or a
/// variant of the same version (`1.99-slim` against `1.99.0`) is not drift.
pub fn is_behind(resolved: &str, latest: &str) -> bool {
    let resolved_numbers = numeric_prefix(resolved);
    let latest_numbers = numeric_prefix(latest);
    if resolved_numbers.is_empty() || latest_numbers.is_empty() {
        return without_v(resolved) != without_v(latest);
    }
    let shared = resolved_numbers.len().min(latest_numbers.len());
    latest_numbers[..shared] > resolved_numbers[..shared]
}

/// api.github.com `/repos/{repo}/tags`: the highest version-shaped tag, for a
/// repository whose "latest release" is not one (github/codeql-action's is a
/// `codeql-bundle-v2.x` CLI bundle, not the action's `v4.x` line).
pub fn github_tags_latest(body: &str) -> Option<String> {
    let mut rest = body;
    let mut best: Option<(Vec<u64>, String)> = None;
    while let Some((tag, end)) = json_field_at(rest, "name") {
        rest = &rest[end..];
        let numbers = numeric_prefix(&tag);
        if !numbers.is_empty()
            && without_v(&tag).starts_with(|c: char| c.is_ascii_digit())
            && best.as_ref().is_none_or(|(current, _)| numbers > *current)
        {
            best = Some((numbers, tag));
        }
    }
    best.map(|(_, tag)| tag)
}

/// The tag `resolved` moves to: `latest`'s numbers at the precision the pin
/// states, keeping its variant suffix (`1.98-slim` with `1.99.0` is
/// `1.99-slim`, not `1.99.0`, which would swap the slim base for the full one).
pub fn docker_tag_at_precision(resolved: &str, latest: &str) -> String {
    let pinned = numeric_prefix(resolved);
    let latest_numbers = numeric_prefix(latest);
    if pinned.is_empty() || latest_numbers.len() < pinned.len() {
        return without_v(latest).to_string();
    }
    let numeric_len = {
        let mut consumed = 0;
        for (index, part) in resolved.split('.').enumerate().take(pinned.len()) {
            let digits = part.chars().take_while(char::is_ascii_digit).count();
            consumed += digits + usize::from(index > 0);
        }
        consumed
    };
    // The publisher's own digit strings, so Ubuntu's `26.04` keeps its zero.
    let numbers: Vec<String> = without_v(latest)
        .split('.')
        .take(pinned.len())
        .map(|part| part.chars().take_while(char::is_ascii_digit).collect())
        .collect();
    format!("{}{}", numbers.join("."), &resolved[numeric_len..])
}
