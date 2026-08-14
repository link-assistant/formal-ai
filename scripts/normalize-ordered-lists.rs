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
//! [dependencies]
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
        || !name
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
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
    let (visibility, rest) = ["pub", "pub(crate)", "pub(super)"]
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
        || !module
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
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
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") && stem != "mod" {
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
        if name.ends_with(".js") {
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
                if let Some(own_stem) = Path::new(&file.path).file_stem().and_then(|stem| stem.to_str())
                {
                    expected.retain(|name| name != own_stem);
                }
                let declared = declared_modules(&normalized);
                let missing: Vec<&String> =
                    expected.iter().filter(|name| !declared.contains(name)).collect();
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
        println!("\n::error::{} ordered list(s) could not be normalized.", failures.len());
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("normalize-ordered-lists-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn registry() -> Vec<ListFile> {
        parse_registry(
            "merge_conflict_policy\n  \
             artifact rust_declaration_lists\n    \
             list_kind rust_declarations\n    \
             file\n      \
             path \"src/lib.rs\"\n    \
             file\n      \
             path \"tests/unit/mod.rs\"\n      \
             declares_directory \"tests/unit\"\n  \
             artifact branch_placeholder\n    \
             merge union\n    \
             file\n      \
             path \".gitkeep\"\n",
        )
    }

    #[test]
    fn the_registry_only_yields_files_that_declare_a_list_kind() {
        let files = registry();
        assert_eq!(files.len(), 2, "the placeholder artifact has no list_kind");
        assert_eq!(files[0].path, "src/lib.rs");
        assert_eq!(files[0].list_kind, "rust_declarations");
        assert_eq!(files[1].declares_directory.as_deref(), Some("tests/unit"));
    }

    #[test]
    fn a_union_merge_of_two_branches_is_restored_to_one_canonical_list() {
        // Exactly what `merge=union` produces when two branches each append a
        // module: both sides kept, in the order git happened to see them.
        let unioned = "mod alpha;\nmod zulu;\nmod beta;\nmod zulu;\n";
        assert_eq!(
            normalize_rust_declarations(unioned).unwrap(),
            "mod alpha;\nmod beta;\nmod zulu;\n",
            "the union is sorted and the repeated entry is dropped"
        );
    }

    #[test]
    fn code_around_a_declaration_run_is_left_untouched() {
        let source = "use std::sync::Mutex;\n\nmod zulu;\nmod alpha;\n\nfn helper() {}\n";
        assert_eq!(
            normalize_rust_declarations(source).unwrap(),
            "use std::sync::Mutex;\n\nmod alpha;\nmod zulu;\n\nfn helper() {}\n"
        );
    }

    #[test]
    fn visibility_is_preserved_and_a_genuine_disagreement_is_reported() {
        assert_eq!(
            normalize_rust_declarations("pub mod zulu;\nmod alpha;\n").unwrap(),
            "mod alpha;\npub mod zulu;\n"
        );
        let error = normalize_rust_declarations("pub mod alpha;\nmod alpha;\n").unwrap_err();
        assert!(error.contains("different visibility"), "{error}");
    }

    #[test]
    fn a_restricted_reexport_sorts_with_the_public_ones() {
        // `src/agentic_coding/modules.rs` interleaves `pub(crate) use` with
        // `pub use`. If the parser skipped the restricted lines they would end
        // the run, and a union merge would sort each fragment on its own.
        let unioned = "pub use zulu::Later;\npub(crate) use alpha::beta;\n";
        assert_eq!(
            normalize_rust_declarations(unioned).unwrap(),
            "pub(crate) use alpha::beta;\npub use zulu::Later;\n"
        );
        let folded = "pub(crate) use alpha::{beta};\npub(crate) use alpha::{gamma};\n";
        assert_eq!(
            normalize_rust_declarations(folded).unwrap(),
            "pub(crate) use alpha::{beta, gamma};\n",
            "a fold keeps the visibility the branches agreed on"
        );
        let error =
            normalize_rust_declarations("pub use alpha::{beta};\npub(crate) use alpha::{gamma};\n")
                .unwrap_err();
        assert!(error.contains("more than one visibility"), "{error}");
    }

    #[test]
    fn repeated_reexports_of_one_module_are_folded_into_a_single_block() {
        let unioned = "pub use zulu::Later;\npub use alpha::{beta};\npub use alpha::{gamma};\n";
        assert_eq!(
            normalize_rust_declarations(unioned).unwrap(),
            "pub use alpha::{beta, gamma};\npub use zulu::Later;\n",
            "two branches extending the same re-export merge into one sorted block"
        );
    }

    #[test]
    fn a_nested_reexport_path_keeps_its_middle_segment() {
        // The bug this test exists for: keyed on the first segment alone,
        // `learning_report::self_hosting_learning::{..}` folded into
        // `learning_report::{..}` and the middle segment vanished, so the file
        // stopped compiling. rustfmt sorts the brace group last, and so does the
        // key, because `{` is above every letter in ASCII.
        let source = "pub use learning_report::self_hosting_learning::{is_task, PATH};\n\
                      pub use learning_report::{LearningReport, REPORTS};\n";
        assert_eq!(normalize_rust_declarations(source).unwrap(), source);
    }

    #[test]
    fn a_reexport_that_appears_once_keeps_the_bytes_rustfmt_gave_it() {
        // rustfmt owns the wrapping of a `use` group; re-deriving it here would
        // fight `cargo fmt --check` over lines this script has no reason to touch.
        let formatted = "pub use zulu::{\n    a_value, ZuluType, ZULU_CONSTANT,\n};\npub use alpha::beta;\n";
        assert_eq!(
            normalize_rust_declarations(formatted).unwrap(),
            "pub use alpha::beta;\npub use zulu::{\n    a_value, ZuluType, ZULU_CONSTANT,\n};\n",
            "blocks are reordered, never reflowed"
        );
    }

    #[test]
    fn merged_items_use_the_rustfmt_group_order() {
        let unioned = "pub use alpha::{ZULU_CONST, TypeB};\npub use alpha::{a_function, TypeA};\n";
        assert_eq!(
            normalize_rust_declarations(unioned).unwrap(),
            "pub use alpha::{a_function, TypeA, TypeB, ZULU_CONST};\n",
            "values, then types, then screaming-case constants"
        );
    }

    #[test]
    fn a_long_reexport_wraps_the_way_rustfmt_wraps_it() {
        let items: Vec<String> = (0..12).map(|index| format!("SomeLongTypeName{index:02}")).collect();
        let lines = render_reexport("pub", "module", &items);
        assert_eq!(lines[0], "pub use module::{");
        assert_eq!(lines[lines.len() - 1], "};");
        assert!(
            lines.iter().all(|line| line.len() <= 100),
            "rustfmt keeps every line inside the 100 column limit"
        );
        assert!(lines[1].starts_with("    SomeLongTypeName00,"));
    }

    #[test]
    fn rustfmt_orders_values_before_types() {
        let mut items = vec![
            "TypeB".to_string(),
            "value_a".to_string(),
            "TypeA".to_string(),
            "value_b".to_string(),
        ];
        items.sort_by(|left, right| rustfmt_item_key(left).cmp(&rustfmt_item_key(right)));
        assert_eq!(items, ["value_a", "value_b", "TypeA", "TypeB"]);
    }

    #[test]
    fn a_path_attribute_names_the_directory_it_points_at() {
        let source = "mod alpha;\n\n#[path = \"ci-cd/mod.rs\"]\nmod ci_cd;\n";
        assert_eq!(declared_modules(source), ["alpha", "ci-cd"]);
    }

    #[test]
    fn a_stale_list_fails_the_check_and_is_repaired_by_write() {
        let root = temp_dir("stale");
        write(&root, "src/lib.rs", "mod zulu;\nmod alpha;\n");
        let file = ListFile {
            path: "src/lib.rs".to_string(),
            list_kind: "rust_declarations".to_string(),
            declares_directory: None,
        };

        assert_eq!(process(&root, &file, false), Outcome::Stale);
        assert_eq!(process(&root, &file, true), Outcome::Rewritten);
        assert_eq!(
            fs::read_to_string(root.join("src/lib.rs")).unwrap(),
            "mod alpha;\nmod zulu;\n"
        );
        assert_eq!(process(&root, &file, false), Outcome::Unchanged);
    }

    #[test]
    fn an_undeclared_module_file_fails_the_directory_mirror_check() {
        let root = temp_dir("mirror");
        write(&root, "tests/unit/alpha.rs", "// alpha\n");
        write(&root, "tests/unit/beta.rs", "// beta\n");
        write(&root, "tests/unit/specification/mod.rs", "// nested\n");
        write(&root, "tests/unit/mod.rs", "mod alpha;\n");
        let file = ListFile {
            path: "tests/unit/mod.rs".to_string(),
            list_kind: "rust_declarations".to_string(),
            declares_directory: Some("tests/unit".to_string()),
        };

        assert_eq!(module_sources(&root.join("tests/unit")), ["alpha", "beta", "specification"]);
        let Outcome::Failed(error) = process(&root, &file, false) else {
            panic!("an undeclared module must fail the check");
        };
        assert!(error.contains("beta"), "{error}");
        assert!(error.contains("specification"), "{error}");
    }

    #[test]
    fn the_worker_list_is_regenerated_from_the_directory_it_mirrors() {
        let root = temp_dir("worker");
        write(&root, "src/web/worker/formal_ai_worker_00.js", "// zero\n");
        write(&root, "src/web/worker/how_to_guide.js", "// guide\n");
        write(&root, "src/web/worker/notes.md", "not a module\n");
        write(&root, "src/web/worker-modules.js", "self.FORMAL_AI_WORKER_MODULES = [];\n");
        let file = ListFile {
            path: "src/web/worker-modules.js".to_string(),
            list_kind: "js_module_list".to_string(),
            declares_directory: Some("src/web/worker".to_string()),
        };

        assert_eq!(
            js_modules_in(&root.join("src/web/worker")),
            ["formal_ai_worker_00.js", "how_to_guide.js"],
            "only JavaScript modules are listed"
        );
        assert_eq!(process(&root, &file, true), Outcome::Rewritten);
        let rendered = fs::read_to_string(root.join("src/web/worker-modules.js")).unwrap();
        assert!(rendered.contains("\"worker/formal_ai_worker_00.js\""));
        assert!(rendered.contains("\"worker/how_to_guide.js\""));
        assert!(!rendered.contains("notes.md"));
        assert_eq!(process(&root, &file, false), Outcome::Unchanged);
    }

    #[test]
    fn the_worker_list_is_rendered_from_the_directory_contents() {
        let rendered = render_js_module_list(
            "src/web/worker",
            &["formal_ai_worker_00.js".to_string(), "how_to_guide.js".to_string()],
        );
        assert!(rendered.contains("self.FORMAL_AI_WORKER_MODULES = Object.freeze(["));
        assert!(rendered.contains("  \"worker/formal_ai_worker_00.js\",\n"));
        assert!(rendered.contains("  \"worker/how_to_guide.js\",\n"));
        assert!(rendered.ends_with("]);\n"));
    }
}
