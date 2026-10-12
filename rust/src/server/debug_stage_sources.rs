//! Which code emits each stage of a debugged turn (issue #667, R383).
//!
//! The JavaScript twin is `js/server/debug-stage-sources.mjs`.
//!
//! A stage is a curated solver event (`source_event`: `impulse`, `language`,
//! `calculation:engine`, `response`, ...). `data/meta/debug-stage-sources.lino`,
//! written by `scripts/generate-debug-stage-sources.mjs` from the source tree
//! and checked current in CI, lists for every curated event kind the
//! functions that append it in each runtime, and each runtime's source trees
//! and solver entry. When one function appends a kind, it is the stage's
//! emitter. When several do, the emitter is the one reachable in the fewest
//! calls (at most [`REACH_DEPTH`]) from the turn's routed handler, else from
//! the solver entry; a kind none of them reaches records no location, never a
//! guessed one. Calls are read by name from the function bodies of the
//! runtime's trees.

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// The stage-sources table. Mirrors `STAGE_SOURCES`.
pub const STAGE_SOURCES: &str = "data/meta/debug-stage-sources.lino";
/// The most calls between a root and an emitter. Mirrors `REACH_DEPTH`.
pub const REACH_DEPTH: usize = 2;
const DEFINITION_MODIFIERS: [&str; 7] = [
    "pub(crate) ",
    "pub(super) ",
    "pub ",
    "export ",
    "async ",
    "const ",
    "unsafe ",
];
const BODY_LINES: usize = 4000;
/// The indentation of a row inside the table's root record.
const ROW_INDENT: &str = "  ";

/// A file and a function it defines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// The repository-relative file.
    pub path: String,
    /// The defined function.
    pub symbol: String,
}

/// One runtime's definition keyword, source trees and solver entries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Runtime {
    /// `fn` or `function`.
    pub keyword: String,
    /// `(directory, extension)` pairs.
    pub trees: Vec<(String, String)>,
    /// Where a solve starts.
    pub entries: Vec<Site>,
}

/// The parsed stage-sources table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageSources {
    /// Runtime name to its definition.
    pub runtimes: HashMap<String, Runtime>,
    /// Event kind to runtime name to the functions appending it, in order.
    pub events: HashMap<String, HashMap<String, Vec<Site>>>,
}

/// The text of `relative` under `root`, empty when it cannot be read.
#[must_use]
pub fn read_text(root: &Path, relative: &str) -> String {
    std::fs::read_to_string(root.join(relative)).unwrap_or_default()
}

const fn is_identifier(character: char, first: bool) -> bool {
    character == '_'
        || character == '$'
        || character.is_ascii_alphabetic()
        || (!first && character.is_ascii_digit())
}

/// The name `line` defines with `keyword`. Mirrors `definedName`.
#[must_use]
pub fn defined_name<'line>(line: &'line str, keyword: &str) -> Option<&'line str> {
    let mut rest = line.trim();
    let mut stripped = true;
    while stripped {
        stripped = false;
        for modifier in DEFINITION_MODIFIERS {
            if let Some(after) = rest.strip_prefix(modifier) {
                rest = after;
                stripped = true;
            }
        }
    }
    let after = rest.strip_prefix(keyword)?.strip_prefix(' ')?;
    let end = after
        .char_indices()
        .find(|&(at, character)| !is_identifier(character, at == 0))
        .map_or(after.len(), |(at, _)| at);
    let name = &after[..end];
    (!name.is_empty() && (after[end..].starts_with('(') || after[end..].starts_with('<')))
        .then_some(name)
}

/// Whether `line` defines `symbol` with `keyword`. Mirrors `defines`.
#[must_use]
pub fn defines(line: &str, keyword: &str, symbol: &str) -> bool {
    defined_name(line, keyword) == Some(symbol)
}

/// The files under `relative` ending in `extension`, sorted. Mirrors `treeFiles`.
#[must_use]
pub fn tree_files(root: &Path, relative: &str, extension: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root.join(relative)) else {
        return Vec::new();
    };
    let mut files: Vec<String> = entries
        .flatten()
        .flat_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let child = format!("{relative}/{name}");
            if entry.path().is_dir() {
                tree_files(root, &child, extension)
            } else if name.ends_with(extension) {
                vec![child]
            } else {
                Vec::new()
            }
        })
        .collect();
    files.sort();
    files
}

/// The index of the line closing the definition at `index`. Mirrors `bodyEnd`.
fn body_end(lines: &[&str], index: usize) -> usize {
    let first = lines[index];
    let indent = &first[..first.len() - first.trim_start().len()];
    let opens = first.matches('{').count();
    if opens > 0 && opens == first.matches('}').count() {
        return index;
    }
    let closing = format!("{indent}}}");
    lines
        .iter()
        .enumerate()
        .skip(index + 1)
        .take(BODY_LINES - 1)
        .find(|(_, line)| **line == closing)
        .map_or(index, |(at, _)| at)
}

fn unquote(token: &str) -> &str {
    token
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(token)
}

fn site(values: &[&str]) -> Site {
    Site {
        path: values.first().copied().unwrap_or_default().to_owned(),
        symbol: values.get(1).copied().unwrap_or_default().to_owned(),
    }
}

/// The stage-sources table under `root`, empty without one. Mirrors
/// `readStageSources`.
#[must_use]
pub fn read_stage_sources(root: &Path) -> StageSources {
    parse_stage_sources(&read_text(root, STAGE_SOURCES))
}

/// The runtimes and per-kind emitters of a stage-sources table's text.
#[must_use]
pub fn parse_stage_sources(text: &str) -> StageSources {
    let mut sources = StageSources::default();
    let mut runtime: Option<String> = None;
    let mut event: Option<String> = None;
    for line in text.split('\n') {
        let trimmed = line.trim_start();
        if !line.starts_with(ROW_INDENT) || trimmed.starts_with('#') {
            continue;
        }
        let depth = line.len() - trimmed.len();
        let tokens: Vec<&str> = trimmed.split_whitespace().map(unquote).collect();
        let Some((head, values)) = tokens.split_first() else {
            continue;
        };
        let first = values.first().copied().unwrap_or_default().to_owned();
        match (depth, *head) {
            (2, "runtime") => {
                sources.runtimes.entry(first.clone()).or_default();
                runtime = Some(first);
                event = None;
            }
            (2, "event") => {
                sources.events.entry(first.clone()).or_default();
                event = Some(first);
                runtime = None;
            }
            (4, field) => {
                if let Some(entry) = runtime
                    .as_ref()
                    .and_then(|name| sources.runtimes.get_mut(name))
                {
                    match field {
                        "keyword" => entry.keyword = first,
                        "tree" => entry
                            .trees
                            .push((first, values.get(1).copied().unwrap_or_default().to_owned())),
                        "entry" => entry.entries.push(site(values)),
                        _ => {}
                    }
                } else if let Some(rows) =
                    event.as_ref().and_then(|kind| sources.events.get_mut(kind))
                    && field != "step"
                {
                    rows.entry(field.to_owned()).or_default().push(site(values));
                }
            }
            _ => {}
        }
    }
    sources
}

/// Every function a runtime's trees define, by name, with the names its body
/// calls. Mirrors `class FunctionIndex`.
#[derive(Debug, Default)]
pub struct FunctionIndex {
    calls: HashMap<String, HashSet<String>>,
}

/// The names `body` calls: identifiers followed by `(`.
fn called_names(body: &str, into: &mut HashSet<String>) {
    let characters: Vec<(usize, char)> = body.char_indices().collect();
    let mut at = 0;
    while at < characters.len() {
        let (start, character) = characters[at];
        if !is_identifier(character, true) {
            at += 1;
            continue;
        }
        let mut end = at;
        while end < characters.len() && is_identifier(characters[end].1, false) {
            end += 1;
        }
        let stop = characters
            .get(end)
            .map_or(body.len(), |&(offset, _)| offset);
        let mut next = end;
        while next < characters.len() && characters[next].1.is_whitespace() {
            next += 1;
        }
        if characters.get(next).is_some_and(|&(_, after)| after == '(') {
            into.insert(body[start..stop].to_owned());
        }
        at = end;
    }
}

impl FunctionIndex {
    /// Index the definitions of `runtime` under `root`.
    #[must_use]
    pub fn new(root: &Path, runtime: &Runtime) -> Self {
        let mut calls: HashMap<String, HashSet<String>> = HashMap::new();
        for (tree, extension) in &runtime.trees {
            for file in tree_files(root, tree, extension) {
                let text = read_text(root, &file);
                let lines: Vec<&str> = text.split('\n').collect();
                for (index, line) in lines.iter().enumerate() {
                    let Some(name) = defined_name(line, &runtime.keyword) else {
                        continue;
                    };
                    let body = lines[index..=body_end(&lines, index)].join("\n");
                    called_names(&body, calls.entry(name.to_owned()).or_default());
                }
            }
        }
        Self { calls }
    }

    /// The first of `candidates` (in their order) at the smallest call depth
    /// from `roots`, at most [`REACH_DEPTH`]. Mirrors `nearest`.
    #[must_use]
    pub fn nearest<'site>(&self, roots: &[&str], candidates: &'site [Site]) -> Option<&'site Site> {
        let mut level: HashSet<String> = roots.iter().map(|root| (*root).to_owned()).collect();
        let mut seen = level.clone();
        for _ in 0..=REACH_DEPTH {
            if level.is_empty() {
                break;
            }
            if let Some(hit) = candidates
                .iter()
                .find(|candidate| level.contains(&candidate.symbol))
            {
                return Some(hit);
            }
            let mut next = HashSet::new();
            for name in &level {
                for called in self.calls.get(name).into_iter().flatten() {
                    if seen.insert(called.clone()) {
                        next.insert(called.clone());
                    }
                }
            }
            level = next;
        }
        None
    }
}

/// One index per process, root and runtime. Mirrors `functionIndex`.
fn function_index(root: &Path, name: &str, runtime: &Runtime) -> Arc<FunctionIndex> {
    static INDEXES: OnceLock<Mutex<HashMap<String, Arc<FunctionIndex>>>> = OnceLock::new();
    let key = format!("{}\u{0}{name}", root.display());
    let mut indexes = INDEXES
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let index = indexes
        .entry(key)
        .or_insert_with(|| Arc::new(FunctionIndex::new(root, runtime)))
        .clone();
    drop(indexes);
    index
}

/// The function that emits a stage of event `kind` in `runtime`, given the
/// turn's routed handler symbol there (empty when the turn has none).
/// Mirrors `stageEmitter`.
#[must_use]
pub fn stage_emitter(
    sources: &StageSources,
    runtime: &str,
    kind: &str,
    handler: &str,
    root: &Path,
) -> Option<Site> {
    let candidates = sources.events.get(kind)?.get(runtime)?;
    if candidates.len() < 2 {
        return candidates.first().cloned();
    }
    let definition = sources.runtimes.get(runtime)?;
    let index = function_index(root, runtime, definition);
    let from_handler = (!handler.is_empty())
        .then(|| index.nearest(&[handler], candidates))
        .flatten();
    let entries: Vec<&str> = definition
        .entries
        .iter()
        .map(|entry| entry.symbol.as_str())
        .collect();
    from_handler
        .or_else(|| index.nearest(&entries, candidates))
        .cloned()
}
