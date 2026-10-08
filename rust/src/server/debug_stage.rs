//! What a step-through debug session shows for one stage (issue #667, R383).
//!
//! The turn's recipe as Mermaid source with the stage highlighted; the
//! `path:symbol` source location — with its line and an excerpt — of the code
//! that emits the stage, in the Rust and in the JavaScript runtime
//! (`debug_stage_sources`); and, as a separate field, the method-registry
//! method the turn's route resolves to with its handler in both runtimes. The
//! JavaScript twin is `js/server/debug-stage.mjs`.
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
//! - a stage's emitter is the function `data/meta/debug-stage-sources.lino`
//!   lists for the stage's `source_event` ([`stage_emitter`]);
//! - each symbol is located by its definition line in the source tree. A
//!   binary without its source tree records no location, never a guessed one.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use serde_json::{Map, Value};

use super::debug_stage_sources::{
    StageSources, defines, read_stage_sources, read_text, stage_emitter, tree_files,
};
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
const RUST: &str = "rust";
const JS: &str = "js";
const RUST_KEYWORD: &str = "fn";
const JS_KEYWORD: &str = "function";
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

/// Where the code that emits one stage is defined in each runtime.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StageView {
    /// The Rust emitter, when found.
    pub rust: Option<SourceLocation>,
    /// The JavaScript emitter, when found.
    pub js: Option<SourceLocation>,
}

/// The method a turn resolves to, where its handler is defined in each
/// runtime, and where each stage's emitter is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnView {
    /// The registry method name, empty when the route resolves to none.
    pub method: String,
    /// The Rust handler definition, when found.
    pub rust: Option<SourceLocation>,
    /// The JavaScript handler definition, when found.
    pub js: Option<SourceLocation>,
    /// One entry per stage of the turn, in order.
    pub stages: Vec<StageView>,
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
        return locate(root, RUST_TREE, ".rs", RUST_KEYWORD, &symbol);
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
        RUST_KEYWORD,
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
    locate(root, JS_TREE, ".js", JS_KEYWORD, &symbol)
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

/// One lookup per process and key.
fn cached<T: Clone>(
    cache: &'static OnceLock<Mutex<HashMap<String, T>>>,
    key: String,
    compute: impl FnOnce() -> T,
) -> T {
    let mut entries = cache
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let value = entries.entry(key).or_insert_with(compute).clone();
    drop(entries);
    value
}

/// Where the code that emits a stage of event `kind` is defined in
/// `runtime`, given the routed handler's symbol there. Mirrors `stageLocation`.
#[must_use]
pub fn stage_location(
    sources: &StageSources,
    runtime: &str,
    kind: &str,
    handler: &str,
    root: &Path,
) -> Option<SourceLocation> {
    let emitter = stage_emitter(sources, runtime, kind, handler, root)?;
    let keyword = if runtime == RUST {
        RUST_KEYWORD
    } else {
        JS_KEYWORD
    };
    locate_in(
        &read_text(root, &emitter.path),
        &emitter.path,
        keyword,
        &emitter.symbol,
    )
}

/// The routed method with its handler in both runtimes, and for every stage
/// the code that emits it in both runtimes. Each location is looked up once
/// per process. Mirrors `describeTurn`.
#[must_use]
pub fn describe_turn(stages: &[ThinkingStep]) -> TurnView {
    static HANDLERS: OnceLock<Mutex<HashMap<String, Locations>>> = OnceLock::new();
    static SOURCES: OnceLock<Mutex<HashMap<String, Arc<StageSources>>>> = OnceLock::new();
    static STAGES: OnceLock<Mutex<HashMap<String, Option<SourceLocation>>>> = OnceLock::new();
    let method = turn_method(stages);
    let Some(root) = source_root() else {
        return TurnView {
            method,
            stages: vec![StageView::default(); stages.len()],
            ..TurnView::default()
        };
    };
    let base = root.display().to_string();
    let (rust, js) = if method.is_empty() {
        (None, None)
    } else {
        cached(&HANDLERS, format!("{base}\u{0}{method}"), || {
            (rust_location(&method, &root), js_location(&method, &root))
        })
    };
    let sources = cached(&SOURCES, base.clone(), || {
        Arc::new(read_stage_sources(&root))
    });
    let at = |runtime: &str, kind: &str, handler: Option<&SourceLocation>| {
        let handler = handler.map_or("", |found| found.symbol.as_str());
        cached(
            &STAGES,
            format!("{base}\u{0}{runtime}\u{0}{kind}\u{0}{handler}"),
            || {
                (!kind.is_empty())
                    .then(|| stage_location(&sources, runtime, kind, handler, &root))
                    .flatten()
            },
        )
    };
    let views = stages
        .iter()
        .map(|stage| StageView {
            rust: at(RUST, &stage.source_event, rust.as_ref()),
            js: at(JS, &stage.source_event, js.as_ref()),
        })
        .collect();
    TurnView {
        method,
        rust,
        js,
        stages: views,
    }
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
