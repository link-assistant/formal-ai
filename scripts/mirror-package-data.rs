#!/usr/bin/env rust-script
//! Mirror every repository file a `rust/src` `include_str!` names into the
//! package, so the published crate is self-contained (issue #1138, plan 16
//! L1 remediation).
//!
//! Usage:
//!   rust-script scripts/mirror-package-data.rs --write
//!   rust-script scripts/mirror-package-data.rs --check
//!   rust-script --test scripts/mirror-package-data.rs
//!
//! `cargo package` builds the extracted `.crate` as the package root, so an
//! `include_str!` that escapes `rust/` cannot resolve in the archive — and
//! this crate is published to crates.io, where the archive is all a consumer
//! gets. The fix is `rust/embedded/`, a committed byte mirror at the
//! repository-relative subpath of every demanded file. Sources point at the
//! mirror; this script keeps the mirror equal to the files it mirrors.
//!
//! `include_str!` in `rust/src` may resolve outside the package root.**
//! Tests and examples are repository self-audits and are not shipped, so the
//! scan covers `rust/src` only.

//! ```cargo
//! [package]
//! edition = "2024"
//! ```

#![cfg_attr(test, allow(dead_code, unused_imports))]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// One `include_str!` site as written in the source.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IncludeSite {
    /// The literal argument when the site is `include_str!("...")`.
    literal: Option<String>,
    /// True for the self-reading idiom `concat!(env!("CARGO_MANIFEST_DIR"),
    /// "/", file!())`, which names a file inside `src/` by construction.
    self_read: bool,
}

/// Classify one `include_str!(...)` argument span (the text between the
/// parentheses, whitespace-trimmed).
fn parse_include_site(argument: &str) -> Result<IncludeSite, String> {
    if argument.contains("file!()") {
        return Ok(IncludeSite {
            literal: None,
            self_read: true,
        });
    }
    let trimmed = argument.trim();
    if let Some(literal) = trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .and_then(|inner| inner.find('"').is_none().then(|| inner.to_string()))
    {
        return Ok(IncludeSite {
            literal: Some(literal),
            self_read: false,
        });
    }
    Err(format!(
        "an include_str! argument that is neither a plain literal nor the \
         file!() self-read idiom: `{trimmed}`; rewrite it to a plain relative \
         literal pointing into rust/embedded/"
    ))
}

/// Every `include_str!` site in one source file, in order.
fn include_sites(text: &str) -> Result<Vec<(usize, IncludeSite)>, String> {
    let needle = "include_str!";
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(start) = text[cursor..].find(needle) {
        let site = cursor + start;
        let line = text[..site].matches('\n').count() + 1;
        let after = &text[site + needle.len()..];
        // A call has `(` as the next non-whitespace byte; doc-comment
        // mentions like [`include_str!`] continue with prose instead.
        let Some(open) = after.find(|byte: char| !byte.is_whitespace()).filter(|&offset| {
            after[offset..].starts_with('(')
        }) else {
            cursor = site + needle.len();
            continue;
        };
        let mut depth = 0usize;
        let mut closed = None;
        for (offset, character) in after[open..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        closed = Some(open + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = closed else {
            return Err(format!("line {line}: unterminated include_str!"));
        };
        let parsed = parse_include_site(&after[open + 1..end])
            .map_err(|error| format!("line {line}: {error}"))?;
        out.push((line, parsed));
        cursor = site + needle.len() + end + 1;
    }
    Ok(out)
}

/// What one resolved `include_str!` target demands of the mirror.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Demand {
    /// Resolves inside `rust/` outside the mirror: already packaged.
    InPackage,
    /// Resolves inside `rust/embedded/<repo-relative>`: demands that the
    /// repository file at that repo-relative path be mirrored byte-equal.
    Mirror { repo_relative: PathBuf },
}

fn classify(target: &Path, package_root: &Path, repo_root: &Path) -> Result<Demand, String> {
    let normalized = normalize_absolute(target);
    let package = normalize_absolute(package_root);
    let repo = normalize_absolute(repo_root);
    if let Ok(rest) = normalized.strip_prefix(&package) {
        let mut components = rest.components();
        if components.next().is_some_and(|component| {
            component == std::path::Component::Normal("embedded".as_ref())
        }) {
            let tail = components.as_path();
            if tail.components().next().is_some() {
                return Ok(Demand::Mirror {
                    repo_relative: tail.to_path_buf(),
                });
            }
        }
        return Ok(Demand::InPackage);
    }
    let shown = normalized
        .strip_prefix(&repo)
        .map(Path::display)
        .map(|shown| shown.to_string())
        .unwrap_or_else(|_| normalized.display().to_string());
    Err(format!(
        "resolves outside the package root: `{shown}`; rust/src may only \
         read files inside rust/, through the rust/embedded/ mirror"
    ))
}

fn normalize_absolute(path: &Path) -> PathBuf {
    let mut out = PathBuf::from("/");
    for component in path.components() {
        match component {
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::Normal(part) => out.push(part),
        }
    }
    out
}

fn collect_demands(
    src_root: &Path,
    package_root: &Path,
    repo_root: &Path,
) -> Result<BTreeMap<PathBuf, Vec<String>>, String> {
    let mut demands: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
    let mut stack = vec![src_root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|error| format!("{} readable: {error}", dir.display()))?;
        for entry in entries {
            let entry =
                entry.map_err(|error| format!("{} walkable: {error}", dir.display()))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("{} readable: {error}", path.display()))?;
            let display = path
                .strip_prefix(repo_root)
                .unwrap_or(&path)
                .display()
                .to_string();
            for (line, site) in include_sites(&text)
                .map_err(|error| format!("{display}: {error}"))?
            {
                if site.self_read {
                    continue;
                }
                let Some(literal) = site.literal else {
                    continue;
                };
                let target = path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(&literal);
                match classify(&target, package_root, repo_root)? {
                    Demand::InPackage => {}
                    Demand::Mirror { repo_relative } => {
                        demands.entry(repo_relative).or_default().push(format!(
                            "{display}:{line}"
                        ));
                    }
                }
            }
        }
    }
    Ok(demands)
}

fn walk_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    let write = match args {
        [flag] if flag == "--write" => true,
        [flag] if flag == "--check" => false,
        _ => {
            return Err("usage: mirror-package-data.rs --write|--check".to_string());
        }
    };
    let repo_root = std::env::current_dir().map_err(|error| error.to_string())?;
    let package_root = repo_root.join("rust");
    let src_root = package_root.join("src");
    let mirror_root = package_root.join("embedded");

    let demands = collect_demands(&src_root, &package_root, &repo_root)?;
    let mut copied = 0usize;
    let mut bytes = 0usize;
    let mut problems: Vec<String> = Vec::new();

    for (repo_relative, sites) in &demands {
        let source = repo_root.join(repo_relative);
        let mirror = mirror_root.join(repo_relative);
        let content = match fs::read(&source) {
            Ok(content) => content,
            Err(error) => {
                problems.push(format!(
                    "{} is mirrored by {} but unreadable: {error}",
                    source.display(),
                    sites.join(", ")
                ));
                continue;
            }
        };
        let mirror_ok = fs::read(&mirror).is_ok_and(|mirrored| mirrored == content);
        if !mirror_ok {
            if write {
                if let Some(parent) = mirror.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| format!("{} creatable: {error}", parent.display()))?;
                }
                fs::write(&mirror, &content)
                    .map_err(|error| format!("{} writable: {error}", mirror.display()))?;
                copied += 1;
                bytes += content.len();
            } else {
                problems.push(format!(
                    "{} is stale (demanded by {}); run \
                     `rust-script scripts/mirror-package-data.rs --write`",
                    mirror.strip_prefix(&repo_root).unwrap_or(&mirror).display(),
                    sites.join(", ")
                ));
            }
        }
    }

    let mut mirrored_files = Vec::new();
    walk_files(&mirror_root, &mut mirrored_files);
    for file in &mirrored_files {
        let tail = file.strip_prefix(&mirror_root).unwrap_or(file);
        if !demands.contains_key(tail) {
            if write {
                fs::remove_file(file)
                    .map_err(|error| format!("{} removable: {error}", file.display()))?;
            } else {
                problems.push(format!(
                    "{} mirrors a file no rust/src include_str! names anymore; \
                     run `rust-script scripts/mirror-package-data.rs --write`",
                    file.strip_prefix(&repo_root).unwrap_or(file).display()
                ));
            }
        }
    }

    if !problems.is_empty() {
        return Err(problems.join("\n"));
    }
    Ok(format!(
        "package-data mirror: {} files demanded, {} refreshed ({} bytes), all byte-equal",
        demands.len(),
        copied,
        bytes
    ))
}

fn main() {
    match run(&std::env::args().skip(1).collect::<Vec<_>>()) {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("package-data mirror failed:\n{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_literals_and_the_self_read_idiom_parse() {
        assert_eq!(
            parse_include_site(r#""../../embedded/data/seed/x.lino""#).unwrap(),
            IncludeSite {
                literal: Some("../../embedded/data/seed/x.lino".to_string()),
                self_read: false,
            }
        );
        assert_eq!(
            parse_include_site(r#"concat!(env!("CARGO_MANIFEST_DIR"), "/", file!())"#)
                .unwrap()
                .self_read,
            true
        );
        assert!(parse_include_site("concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../data/x\")")
            .is_err());
    }

    #[test]
    fn include_sites_walk_every_call_in_order() {
        let text = concat!(
            "const A: &str = include_str!(\"../embedded/data/a.lino\");\n",
            "const B: &str = include_str!(\n  \"../embedded/js/b.js\"\n);\n",
        );
        let sites = include_sites(text).unwrap();
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].0, 1);
        assert_eq!(sites[1].1.literal.as_deref(), Some("../embedded/js/b.js"));
    }

    #[test]
    fn classification_splits_mirror_demand_package_inside_and_escape() {
        let package = Path::new("/repo/rust");
        let repo = Path::new("/repo");
        assert_eq!(
            classify(
                &Path::new("/repo/rust/src/x.rs"),
                package,
                repo
            )
            .unwrap(),
            Demand::InPackage
        );
        assert_eq!(
            classify(
                &Path::new("/repo/rust/embedded/data/seed/x.lino"),
                package,
                repo
            )
            .unwrap(),
            Demand::Mirror {
                repo_relative: PathBuf::from("data/seed/x.lino")
            }
        );
        assert!(classify(&Path::new("/repo/data/seed/x.lino"), package, repo).is_err());
        assert!(classify(&Path::new("/repo/rust/../data/x.lino"), package, repo).is_err());
    }
}
