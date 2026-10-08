//! What a step-through debug session shows for one stage (issue #667, R383).
//!
//! The turn's recipe as Mermaid source with the stage highlighted, and the
//! method-registry `path:symbol` source location — with its line and an
//! excerpt — of the handler the turn's route resolves to, in the Rust and in
//! the JavaScript runtime. The JavaScript twin is `js/server/debug-stage.mjs`.
//!
//! Everything is derived from live data:
//! - the method is [`MethodRegistry::method_for_route`] over the route the
//!   turn's `formalize` stage names (else its `dispatch_handler` stage);
//! - the Rust symbol is the method's row in the `HANDLER_FUNCTIONS` table of
//!   `rust/src/solver_dispatch.rs`, or the rule interpreter's `run_handler`
//!   for a method `data/seed/handler-rules.lino` implements;
//! - the JavaScript symbol is the browser handler of
//!   `data/seed/browser-handler-precedence.lino` named `try` + the method
//!   (case and underscores aside), or the worker's `runHandlerRuleSet`;
//! - each symbol is located by its definition line in the source tree. A
//!   binary without its source tree records no location, never a guessed one.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, PoisonError};

use serde_json::{Map, Value};

use crate::method_registry::MethodRegistry;
use crate::thinking::ThinkingStep;

/// Mirrors `const DISPATCH_TABLE`.
const DISPATCH_TABLE: &str = "rust/src/solver_dispatch.rs";
const DISPATCH_TABLE_START: &str = "const HANDLER_FUNCTIONS";
const DISPATCH_TABLE_END: &str = "];";
const RULES_SEED: &str = "data/seed/handler-rules.lino";
const BROWSER_SEED: &str = "data/seed/browser-handler-precedence.lino";
const SEED_HANDLER_PREFIX: &str = "  handler ";
const RUST_TREE: &str = "rust/src";
const JS_TREE: &str = "js/worker";
const RUST_RULE_RUNNER_PATH: &str = "rust/src/rule_interpreter.rs";
const RUST_RULE_RUNNER_SYMBOL: &str = "run_handler";
const JS_RULE_RUNNER: &str = "runHandlerRuleSet";
const BROWSER_HANDLER_PREFIX: &str = "try";
const ROUTE_STEPS: [&str; 2] = ["formalize", "dispatch_handler"];
const DEFINITION_MODIFIERS: [&str; 7] = [
    "pub(crate) ",
    "pub(super) ",
    "pub ",
    "export ",
    "async ",
    "const ",
    "unsafe ",
];
/// The most lines an excerpt shows. Mirrors `EXCERPT_LINES`.
pub const EXCERPT_LINES: usize = 40;
const CURRENT_CLASS: &str = "current";
const CURRENT_STYLE: &str = "stroke-width:4px";

/// One located definition: `path:symbol`, its 1-based line and an excerpt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    /// The repository-relative file.
    pub path: String,
    /// The defined function.
    pub symbol: String,
    /// The 1-based line of the definition.
    pub line: usize,
    /// The definition through its closing brace, at most [`EXCERPT_LINES`].
    pub excerpt: String,
}

/// The method a turn resolves to and where it is defined in each runtime.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnView {
    /// The registry method name, empty when the route resolves to none.
    pub method: String,
    /// The Rust definition, when found.
    pub rust: Option<SourceLocation>,
    /// The JavaScript definition, when found.
    pub js: Option<SourceLocation>,
}

fn read_text(root: &Path, relative: &str) -> String {
    std::fs::read_to_string(root.join(relative)).unwrap_or_default()
}

/// Mirrors `seedHandlers`: the `  handler <name>` rows of a seed document.
fn seed_handlers(text: &str) -> Vec<String> {
    text.split('\n')
        .filter_map(|line| line.strip_prefix(SEED_HANDLER_PREFIX))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

/// Mirrors `dispatchSymbol`: the function the dispatch table binds `method` to.
fn dispatch_symbol(text: &str, method: &str) -> Option<String> {
    let start = text.find(DISPATCH_TABLE_START)?;
    let table = &text[start..];
    let table = table
        .find(DISPATCH_TABLE_END)
        .map_or(table, |end| &table[..end]);
    table.split('\n').find_map(|line| {
        let row = line.trim().strip_prefix("(\"")?;
        let (name, rest) = row.split_once("\",")?;
        if name != method {
            return None;
        }
        let rest = rest.trim();
        let symbol = rest
            .split_once(')')
            .map_or(rest, |(symbol, _)| symbol)
            .trim();
        (!symbol.is_empty()).then(|| symbol.to_owned())
    })
}

/// Mirrors `defines`: whether `line` defines `symbol` with `keyword`.
fn defines(line: &str, keyword: &str, symbol: &str) -> bool {
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
    let head = format!("{keyword} {symbol}");
    rest.strip_prefix(&head)
        .is_some_and(|after| after.starts_with('(') || after.starts_with('<'))
}

/// Mirrors `treeFiles`: the files under `relative` ending in `extension`, sorted.
fn tree_files(root: &Path, relative: &str, extension: &str) -> Vec<String> {
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

/// Mirrors `excerpt`: the definition through its closing brace, capped.
fn excerpt(lines: &[&str], index: usize) -> String {
    let first = lines[index];
    let indent = &first[..first.len() - first.trim_start().len()];
    let closing = format!("{indent}}}");
    let mut out: Vec<&str> = Vec::new();
    for (at, line) in lines.iter().enumerate().skip(index) {
        if out.len() >= EXCERPT_LINES {
            break;
        }
        out.push(*line);
        if at > index && *line == closing {
            break;
        }
    }
    out.join("\n")
}

fn locate_in(text: &str, path: &str, keyword: &str, symbol: &str) -> Option<SourceLocation> {
    let lines: Vec<&str> = text.split('\n').collect();
    let index = lines
        .iter()
        .position(|line| defines(line, keyword, symbol))?;
    Some(SourceLocation {
        path: path.to_owned(),
        symbol: symbol.to_owned(),
        line: index + 1,
        excerpt: excerpt(&lines, index),
    })
}

/// Mirrors `locate`: the first definition of `symbol` in the sorted tree.
fn locate(
    root: &Path,
    tree: &str,
    extension: &str,
    keyword: &str,
    symbol: &str,
) -> Option<SourceLocation> {
    tree_files(root, tree, extension)
        .iter()
        .find_map(|file| locate_in(&read_text(root, file), file, keyword, symbol))
}

/// Where `method` is defined in Rust under `root`. Mirrors `rustLocation`.
#[must_use]
pub fn rust_location(method: &str, root: &Path) -> Option<SourceLocation> {
    if let Some(symbol) = dispatch_symbol(&read_text(root, DISPATCH_TABLE), method) {
        return locate(root, RUST_TREE, ".rs", "fn", &symbol);
    }
    if !seed_handlers(&read_text(root, RULES_SEED))
        .iter()
        .any(|name| name == method)
    {
        return None;
    }
    locate_in(
        &read_text(root, RUST_RULE_RUNNER_PATH),
        RUST_RULE_RUNNER_PATH,
        "fn",
        RUST_RULE_RUNNER_SYMBOL,
    )
}

/// Where `method` is defined in the browser worker under `root`. Mirrors
/// `jsLocation`.
#[must_use]
pub fn js_location(method: &str, root: &Path) -> Option<SourceLocation> {
    let wanted = format!("{BROWSER_HANDLER_PREFIX}{}", method.replace('_', "")).to_lowercase();
    let handler = seed_handlers(&read_text(root, BROWSER_SEED))
        .into_iter()
        .find(|name| name.to_lowercase() == wanted);
    let symbol = handler.or_else(|| {
        seed_handlers(&read_text(root, RULES_SEED))
            .iter()
            .any(|name| name == method)
            .then(|| JS_RULE_RUNNER.to_owned())
    })?;
    locate(root, JS_TREE, ".js", "function", &symbol)
}

/// The registry method the turn's route resolves to. Mirrors `turnMethod`.
#[must_use]
pub fn turn_method(stages: &[ThinkingStep]) -> String {
    for kind in ROUTE_STEPS {
        let route = stages
            .iter()
            .find(|stage| stage.step == kind)
            .map_or("", |stage| stage.detail.trim());
        if !route.is_empty() {
            return MethodRegistry::shared()
                .method_for_route(route)
                .map(|method| method.name.clone())
                .unwrap_or_default();
        }
    }
    String::new()
}

/// The source tree the server was built from: the crate's parent directory,
/// else the working directory, whichever holds the dispatch table.
fn source_root() -> Option<PathBuf> {
    let built = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    [Some(built), std::env::current_dir().ok()]
        .into_iter()
        .flatten()
        .find(|root| root.join(DISPATCH_TABLE).is_file())
}

type Locations = (Option<SourceLocation>, Option<SourceLocation>);

/// The method and both source locations of a turn, each location looked up
/// once per process and method. Mirrors `describeTurn`.
#[must_use]
pub fn describe_turn(stages: &[ThinkingStep]) -> TurnView {
    static CACHE: OnceLock<Mutex<HashMap<String, Locations>>> = OnceLock::new();
    let method = turn_method(stages);
    if method.is_empty() {
        return TurnView::default();
    }
    let Some(root) = source_root() else {
        return TurnView {
            method,
            ..TurnView::default()
        };
    };
    let mut cache = CACHE
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let (rust, js) = cache
        .entry(method.clone())
        .or_insert_with(|| (rust_location(&method, &root), js_location(&method, &root)))
        .clone();
    drop(cache);
    TurnView { method, rust, js }
}

/// Mirrors `label`: a Mermaid node label, quotes entity-escaped.
fn label(text: &str) -> String {
    text.replace('"', "#quot;")
}

/// The turn's stages as a Mermaid flowchart: top-level stages chained in
/// order, a sub-stage hanging off its parent, the stage at `current`
/// highlighted. Mirrors `stageDiagram`.
#[must_use]
pub fn stage_diagram(stages: &[ThinkingStep], current: usize) -> String {
    let mut lines = vec![String::from("flowchart TD")];
    for (index, stage) in stages.iter().enumerate() {
        lines.push(format!("    s{index}[\"{index} {}\"]", label(&stage.step)));
    }
    let mut previous: Option<usize> = None;
    for (index, stage) in stages.iter().enumerate() {
        let parent = stage
            .parent_id
            .as_deref()
            .and_then(|parent| stages.iter().position(|candidate| candidate.id == parent));
        if let Some(parent) = parent {
            lines.push(format!("    s{parent} -.-> s{index}"));
            continue;
        }
        if let Some(previous) = previous {
            lines.push(format!("    s{previous} --> s{index}"));
        }
        previous = Some(index);
    }
    lines.push(format!("    classDef {CURRENT_CLASS} {CURRENT_STYLE}"));
    if current < stages.len() {
        lines.push(format!("    class s{current} {CURRENT_CLASS}"));
    }
    lines.join("\n")
}

/// The event fields of one runtime's location. Mirrors `locationFields`.
pub fn location_fields(
    fields: &mut Map<String, Value>,
    prefix: &str,
    location: Option<&SourceLocation>,
) {
    fields.insert(
        format!("{prefix}_source"),
        Value::from(location.map_or_else(String::new, |found| {
            format!("{}:{}", found.path, found.symbol)
        })),
    );
    fields.insert(
        format!("{prefix}_line"),
        Value::from(location.map_or(0, |found| found.line)),
    );
    fields.insert(
        format!("{prefix}_excerpt"),
        Value::from(location.map_or("", |found| found.excerpt.as_str())),
    );
}
