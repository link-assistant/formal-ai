#!/usr/bin/env rust-script
//! Keep every union-merged list in its canonical order.
//!
//! Issue #991 review feedback: "find a way to reduce possibility of conflicts
//! in these files in the future ... the end result should be that probability
//! of conflicts in the future reduced to zero."
//!
//! `scripts/analyze-merge-conflicts.py` shows that the single largest class of
//! manual conflict resolutions in this repository is the *append-only list*:
//! `src/lib.rs`, `tests/unit/mod.rs` and their siblings are nothing but a
//! canonically ordered list of declarations, and two branches that each add one
//! unrelated entry collide on the same region.
//!
//! `.gitattributes` marks those files `merge=union`, so git keeps both sides
//! instead of reporting a conflict. A union is a *superset* of two correct
//! files: it can be out of order and it can repeat an entry, but it never loses
//! one. This script is the second half of that mechanism — it restores the
//! canonical order and removes the repeats, and in `--check` mode it fails CI
//! whenever a committed list is not already canonical.
//!
//! The registry lives in `data/meta/merge-conflict-policy.lino`, so adding a new
//! union-merged list is a data-only edit.
//!
//! Usage:
//!     rust-script scripts/normalize-ordered-lists.rs           # check
//!     rust-script scripts/normalize-ordered-lists.rs --write   # rewrite
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! syn = { version = "2", features = ["full"] }
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[path = "lib/registered-module-paths.rs"]
mod registered_module_paths;

#[cfg(not(test))]
const POLICY: &str = "data/meta/merge-conflict-policy.lino";

/// One registered list file and how it is kept canonical.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ListFile {
    path: String,
    list_kind: String,
    /// Directory whose modules this list must declare, when the list is an
    /// exact mirror of a directory's contents.
    declares_directory: Option<String>,
    /// Module stems present in `declares_directory` that the list deliberately
    /// does not declare yet: a plan drafted as tests before its leaf lands
    /// (issue #1138's draft-first discipline). The mirror stays exact for
    /// everything else, so registering a red test early can never be forced by
    /// this gate.
    inert: Vec<String>,
}

// ---------------------------------------------------------------------------
// Links Notation registry
// ---------------------------------------------------------------------------

/// Strip one level of surrounding quotes from a registry value.
fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Read the `artifact` entries of the policy file that declare a `list_kind`.
///
/// The registry is indentation-structured Links Notation; only the two nesting
/// levels this script cares about are interpreted, which keeps the parser small
/// enough to live inside the gate it powers.
fn parse_registry(source: &str) -> Vec<ListFile> {
    let mut files = Vec::new();
    let mut artifact_kind: Option<String> = None;
    let mut current: Option<ListFile> = None;

    for line in source.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = indent_of(line);
        let trimmed = line.trim();
        let (key, value) = match trimmed.split_once(' ') {
            Some((key, value)) => (key, value.trim()),
            None => (trimmed, ""),
        };

        // A new artifact ends the previous one's file list.
        if indent == 2 {
            if let Some(file) = current.take() {
                files.push(file);
            }
            artifact_kind = None;
        }
        match (indent, key) {
            (4, "list_kind") => artifact_kind = Some(unquote(value)),
            (4, "file") => {
                if let Some(file) = current.take() {
                    files.push(file);
                }
                if let Some(kind) = artifact_kind.clone() {
                    current = Some(ListFile {
                        path: String::new(),
                        list_kind: kind,
                        declares_directory: None,
                        inert: Vec::new(),
                    });
                }
            }
            (6, "path") => {
                if let Some(file) = current.as_mut() {
                    file.path = unquote(value);
                }
            }
            (6, "declares_directory") => {
                if let Some(file) = current.as_mut() {
                    file.declares_directory = Some(unquote(value));
                }
            }
            (6, "inert") => {
                if let Some(file) = current.as_mut() {
                    file.inert.push(unquote(value));
                }
            }
            _ => {}
        }
    }
    if let Some(file) = current.take() {
        files.push(file);
    }
    files.retain(|file| !file.path.is_empty());
    files
}

// ---------------------------------------------------------------------------
// Rust declaration lists
// ---------------------------------------------------------------------------

/// Split `mod name;` into its visibility and its name.
fn parse_mod_declaration(line: &str) -> Option<(String, String)> {
    let rest = line.strip_suffix(';')?;
    let (visibility, rest) = match rest.rsplit_once("mod ") {
        Some((visibility, name)) => (visibility.trim_end(), name),
        None => return None,
    };
    if !matches!(visibility, "" | "pub" | "pub(crate)" | "pub(super)") {
        return None;
    }
    let name = rest.trim();
    if name.is_empty()
        || !name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return None;
    }
    Some((visibility.to_string(), name.to_string()))
}

/// The visibility and sort key of a re-export block, if a block starts here.
///
/// The key is the path with everything from the opening brace collapsed to a
/// bare `{`: `learning_report::self_hosting_learning::{` sorts before
/// `learning_report::{` exactly as rustfmt orders them, because `{` is above
/// every letter in ASCII. Keying on the first segment alone would have merged
/// `learning_report::self_hosting_learning::{..}` into `learning_report::{..}`
/// and silently dropped the middle segment.
fn parse_reexport_module(line: &str) -> Option<(String, String)> {
    // `pub(crate) use` is as much a list entry as `pub use`; refusing to parse it
    // would split the run in two and leave a union merge sorting each half apart.
    let (visibility, rest) =
        ["pub", "pub(crate)", "pub(super)"]
            .into_iter()
            .find_map(|visibility| {
                line.strip_prefix(&format!("{visibility} use "))
                    .map(|rest| (visibility, rest))
            })?;
    let key = match rest.find('{') {
        Some(brace) => format!("{}{{", &rest[..brace]),
        None => rest.trim().trim_end_matches(';').trim().to_string(),
    };
    let (module, _) = key.split_once("::")?;
    if module.is_empty()
        || !module.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
    {
        return None;
    }
    Some((visibility.to_string(), key))
}

/// Sort key that matches rustfmt's ordering inside a `use` group: values and
/// functions first, then types, then screaming-case constants.
fn rustfmt_item_key(item: &str) -> (u8, String) {
    let group = if item.starts_with(|character: char| character.is_ascii_lowercase()) {
        0
    } else if item.chars().any(|character| character.is_ascii_lowercase()) {
        1
    } else {
        2
    };
    (group, item.to_string())
}

/// Re-emit `pub use module::{items};` the way rustfmt would.
fn render_reexport(visibility: &str, module: &str, items: &[String]) -> Vec<String> {
    if items.len() == 1 && !items[0].contains('{') {
        let single = format!("{visibility} use {module}::{};", items[0]);
        if single.len() <= 100 {
            return vec![single];
        }
    }
    let joined = items.join(", ");
    let single = format!("{visibility} use {module}::{{{joined}}};");
    if single.len() <= 100 {
        return vec![single];
    }

    let mut lines = vec![format!("{visibility} use {module}::{{")];
    let mut current = String::new();
    for (index, item) in items.iter().enumerate() {
        let last = index + 1 == items.len();
        let piece = if last {
            format!("{item},")
        } else {
            format!("{item},")
        };
        let candidate = if current.is_empty() {
            format!("    {piece}")
        } else {
            format!("{current} {piece}")
        };
        if candidate.len() > 100 && !current.is_empty() {
            lines.push(current);
            current = format!("    {piece}");
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.push("};".to_string());
    lines
}

/// The brace items of a re-export block, or `None` when the block nests braces
/// and cannot be merged item-wise.
fn reexport_items(block: &[String]) -> Option<Vec<String>> {
    let text = block.join(" ");
    let body = text.trim_end_matches(';');
    let inner = match body.split_once('{') {
        Some((_, rest)) => rest.trim_end().strip_suffix('}')?,
        None => {
            let (_, item) = body.split_once("::")?;
            return Some(vec![item.trim().to_string()]);
        }
    };
    if inner.contains('{') || inner.contains('}') {
        return None;
    }
    let mut items: Vec<String> = inner
        .split(',')
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect();
    items.sort_by(|left, right| rustfmt_item_key(left).cmp(&rustfmt_item_key(right)));
    items.dedup();
    Some(items)
}

/// Canonicalize the `mod` runs and the `pub use` run of a Rust declaration list.
fn normalize_rust_declarations(source: &str) -> Result<String, String> {
    let lines: Vec<String> = source.lines().map(str::to_string).collect();
    let mut output: Vec<String> = Vec::with_capacity(lines.len());
    let mut index = 0;

    while index < lines.len() {
        // A maximal run of bare `mod name;` declarations.
        if parse_mod_declaration(&lines[index]).is_some() {
            let start = index;
            while index < lines.len() && parse_mod_declaration(&lines[index]).is_some() {
                index += 1;
            }
            let mut declarations: Vec<(String, String)> = lines[start..index]
                .iter()
                .filter_map(|line| parse_mod_declaration(line))
                .collect();
            declarations.sort_by(|left, right| left.1.cmp(&right.1));
            declarations.dedup();
            let mut seen: BTreeMap<String, String> = BTreeMap::new();
            for (visibility, name) in &declarations {
                if let Some(previous) = seen.insert(name.clone(), visibility.clone()) {
                    return Err(format!(
                        "module `{name}` is declared twice with different visibility \
                         (`{previous}` and `{visibility}`); resolve that by hand"
                    ));
                }
            }
            for (visibility, name) in declarations {
                output.push(if visibility.is_empty() {
                    format!("mod {name};")
                } else {
                    format!("{visibility} mod {name};")
                });
            }
            continue;
        }

        // A maximal run of `pub use module::...;` blocks.
        if parse_reexport_module(&lines[index]).is_some() {
            let mut blocks: Vec<(String, String, Vec<String>)> = Vec::new();
            while index < lines.len() {
                let Some((visibility, module)) = parse_reexport_module(&lines[index]) else {
                    break;
                };
                let start = index;
                while index < lines.len() && !lines[index].trim_end().ends_with(';') {
                    index += 1;
                }
                if index < lines.len() {
                    index += 1;
                }
                blocks.push((visibility, module, lines[start..index].to_vec()));
            }
            output.extend(canonical_reexports(blocks)?);
            continue;
        }

        output.push(lines[index].clone());
        index += 1;
    }

    let mut text = output.join("\n");
    if source.ends_with('\n') {
        text.push('\n');
    }
    Ok(text)
}

/// Sort re-export blocks by module and fold repeats of the same module into one.
///
/// A module that appears exactly once keeps its committed bytes: rustfmt owns
/// the wrapping of a `use` group, and re-deriving it here would fight
/// `cargo fmt --check` over lines this script has no reason to touch. Only the
/// repeats a union merge creates are re-rendered.
fn canonical_reexports(blocks: Vec<(String, String, Vec<String>)>) -> Result<Vec<String>, String> {
    // Grouped by module alone, not by visibility: the module name is what the
    // list is ordered on, and the same module re-exported at two visibilities is
    // a real ambiguity a script must not silently pick a side in.
    let mut by_module: BTreeMap<String, (BTreeSet<String>, Vec<Vec<String>>)> = BTreeMap::new();
    for (visibility, module, block) in blocks {
        let entry = by_module.entry(module).or_default();
        entry.0.insert(visibility);
        entry.1.push(block);
    }

    let mut lines = Vec::new();
    for (module, (visibilities, mut occurrences)) in by_module {
        occurrences.sort();
        occurrences.dedup();
        if occurrences.len() == 1 {
            lines.extend(occurrences.into_iter().next().expect("one occurrence"));
            continue;
        }
        if visibilities.len() > 1 {
            return Err(format!(
                "re-exports from `{module}` are declared at more than one visibility \
                 ({}); resolve that by hand",
                visibilities.into_iter().collect::<Vec<_>>().join(" and ")
            ));
        }
        let visibility = visibilities.into_iter().next().expect("one visibility");
        let Some(prefix) = module.strip_suffix("::{") else {
            return Err(format!(
                "`{module}` is re-exported {} times without a brace group, so the repeats \
                 differ in more than their items; resolve that by hand",
                occurrences.len()
            ));
        };
        let mut items = Vec::new();
        for block in &occurrences {
            let Some(block_items) = reexport_items(block) else {
                return Err(format!(
                    "re-exports from `{module}` are declared {} times and at least one block \
                     nests braces; resolve that by hand",
                    occurrences.len()
                ));
            };
            items.extend(block_items);
        }
        items.sort_by(|left, right| rustfmt_item_key(left).cmp(&rustfmt_item_key(right)));
        items.dedup();
        lines.extend(render_reexport(&visibility, prefix, &items));
    }
    Ok(lines)
}

// ---------------------------------------------------------------------------
// Directory-mirroring lists
// ---------------------------------------------------------------------------

/// The module sources a Rust directory offers: `name.rs` files and `name/`
/// directories that carry their own `mod.rs`.
fn module_sources(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return names;
    };
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if path.join("mod.rs").is_file() {
                names.push(stem.to_string());
            }
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") && stem != "mod"
        {
            names.push(stem.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Modules a declaration list names, honouring `#[path = "..."]` overrides.
fn declared_modules(source: &str) -> Vec<String> {
    let lines: Vec<String> = source.lines().map(str::to_string).collect();
    let mut declared = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some((_, name)) = parse_mod_declaration(line) else {
            continue;
        };
        let override_path = index
            .checked_sub(1)
            .and_then(|previous| lines.get(previous))
            .and_then(|previous| previous.trim().strip_prefix("#[path = \""))
            .and_then(|rest| rest.split('"').next())
            .map(str::to_string);
        match override_path {
            Some(path) => {
                let path = Path::new(&path);
                let stem = if path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
                    path.parent().and_then(|parent| parent.file_name())
                } else {
                    path.file_stem()
                };
                if let Some(stem) = stem.and_then(|stem| stem.to_str()) {
                    declared.push(stem.to_string());
                }
            }
            None => declared.push(name),
        }
    }
    declared.sort();
    declared.dedup();
    declared
}

/// The worker module list, regenerated from the directory it mirrors.
fn render_js_module_list(dir_label: &str, modules: &[String]) -> String {
    let mut text = String::new();
    text.push_str(&format!(
        "// Generated by `rust-script scripts/normalize-ordered-lists.rs --write`.\n\
         //\n\
         // The browser worker loads every module in `{dir_label}/`. Issue #991 moved\n\
         // this list out of `formal_ai_worker.js` so it can be union merged: two branches\n\
         // that each add a worker module now produce a superset instead of a conflict, and\n\
         // the generator above restores this exact content from the directory itself.\n\
         \n\
         self.FORMAL_AI_WORKER_MODULES = Object.freeze([\n"
    ));
    let prefix = dir_label.rsplit('/').next().unwrap_or(dir_label);
    for module in modules {
        text.push_str(&format!("  \"{prefix}/{module}\",\n"));
    }
    text.push_str("]);\n");
    text
}

fn js_modules_in(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return names;
    };
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        // `formal_ai_worker.js` is the entry point that loads this list, not
        // a module in it: listing it would make the entry import itself.
        if name.ends_with(".js") && name != "formal_ai_worker.js" {
            names.push(name.to_string());
        }
    }
    names.sort();
    names
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Unchanged,
    Rewritten,
    Stale,
    Failed(String),
}

fn canonical_content(root: &Path, file: &ListFile, current: &str) -> Result<String, String> {
    match file.list_kind.as_str() {
        "rust_declarations" => {
            let normalized = normalize_rust_declarations(current)?;
            if let Some(directory) = &file.declares_directory {
                let mut expected = module_sources(&root.join(directory));
                // An extracted list file sits next to the modules it declares
                // but is not itself one of them.
                if let Some(own_stem) = Path::new(&file.path)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                {
                    expected.retain(|name| name != own_stem);
                }
                // A module the registry marks inert is a draft awaiting its
                // leaf, not drift: the declaration list stays canonical
                // without it, and the day the leaf lands the registry entry
                // goes away and the mirror becomes exact again.
                expected.retain(|name| !file.inert.contains(name));
                let mut declared = declared_modules(&normalized);
                let registrations =
                    registered_module_paths::registered_paths(root, &root.join(directory))?;
                for name in &expected {
                    let target = root.join(directory).join(format!("{name}.rs"));
                    if target
                        .canonicalize()
                        .is_ok_and(|target| registrations.contains(&target))
                    {
                        declared.push(name.clone());
                    }
                }
                let missing: Vec<&String> = expected
                    .iter()
                    .filter(|name| !declared.contains(name))
                    .collect();
                if !missing.is_empty() {
                    return Err(format!(
                        "{} does not declare {} module(s) present in {directory}/: {}",
                        file.path,
                        missing.len(),
                        missing
                            .iter()
                            .map(|name| name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
            }
            Ok(normalized)
        }
        "js_module_list" => {
            let directory = file
                .declares_directory
                .as_ref()
                .ok_or_else(|| format!("{} has no declares_directory", file.path))?;
            let modules = js_modules_in(&root.join(directory));
            if modules.is_empty() {
                return Err(format!("{directory}/ has no JavaScript modules to list"));
            }
            Ok(render_js_module_list(directory, &modules))
        }
        other => Err(format!("{}: unknown list kind `{other}`", file.path)),
    }
}

fn process(root: &Path, file: &ListFile, write: bool) -> Outcome {
    let path = root.join(&file.path);
    let current = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => return Outcome::Failed(format!("{}: {error}", file.path)),
    };
    let canonical = match canonical_content(root, file, &current) {
        Ok(canonical) => canonical,
        Err(error) => return Outcome::Failed(error),
    };
    if canonical == current {
        return Outcome::Unchanged;
    }
    if !write {
        return Outcome::Stale;
    }
    match fs::write(&path, canonical) {
        Ok(()) => Outcome::Rewritten,
        Err(error) => Outcome::Failed(format!("{}: {error}", file.path)),
    }
}

#[cfg(not(test))]
fn main() {
    let write = std::env::args().any(|argument| argument == "--write");
    let root = std::env::current_dir().expect("Failed to get current directory");

    let policy = match fs::read_to_string(root.join(POLICY)) {
        Ok(policy) => policy,
        Err(error) => {
            println!("::error::Could not read {POLICY}: {error}");
            std::process::exit(1);
        }
    };
    let files = parse_registry(&policy);
    if files.is_empty() {
        println!("::error::{POLICY} registers no ordered lists.");
        std::process::exit(1);
    }

    println!(
        "\n{} union-merged list(s) registered in {POLICY}:\n",
        files.len()
    );

    let mut stale: Vec<String> = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    for file in &files {
        match process(&root, file, write) {
            Outcome::Unchanged => println!("  canonical  {}", file.path),
            Outcome::Rewritten => println!("  rewritten  {}", file.path),
            Outcome::Stale => {
                println!("  STALE      {}", file.path);
                stale.push(file.path.clone());
            }
            Outcome::Failed(error) => {
                println!("  FAILED     {error}");
                failures.push(error);
            }
        }
    }

    if !failures.is_empty() {
        println!(
            "\n::error::{} ordered list(s) could not be normalized.",
            failures.len()
        );
        std::process::exit(1);
    }
    if !stale.is_empty() {
        println!(
            "\n::error::{} ordered list(s) are not in canonical order.",
            stale.len()
        );
        println!(
            "Run `rust-script scripts/normalize-ordered-lists.rs --write` and commit the result.\n"
        );
        std::process::exit(1);
    }
    println!("\nEvery registered list is canonical.\n");
}

#[cfg(test)]
#[path = "normalize-ordered-lists-tests.rs"]
mod tests;
