//! Code evidence for fact checking (issue #1179 R4).
//!
//! Answers bounded queries about the repository's own code — "is function `f`
//! called outside `#[cfg(test)]` / `rust/tests/`?", "does symbol `S` exist at
//! path `P`?" — the way `agentic_coding::self_ast` already reasons about the
//! meta algorithm: every candidate file is first parsed through the sole
//! CST/AST engine in this repo (`meta_language::LinkNetwork`), and only a
//! clean, text-preserving parse certifies the document whose lines are then
//! inspected. A file the engine cannot certify yields no evidence rather than
//! evidence from a guess.
//!
//! The bounded queries recognize symbols as whole words, tracking test scope
//! (`#[cfg(test)]` modules and the `tests/` tree). The query vocabulary is
//! data: `code_query_cue` rows of `data/seed/fact-check-sources.lino` decide
//! which phrasings map to which query, so a new bounded query shape is a seed
//! row plus one answer arm.

use std::fs;
use std::path::{Path, PathBuf};

use crate::relative_meta_logic::{RelativeEvidence, Stance, TruthValue};

use super::tier_for_context;

/// A bounded code query, recognized from phrasings declared in the seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeQuery {
    /// "Is `symbol` called outside tests and not only from the test tree?"
    FunctionCalledOutsideTests { symbol: String },
    /// "Does `symbol` exist in `path`?"
    SymbolExistsAtPath { symbol: String, path: String },
}

impl CodeQuery {
    /// The seed query id this variant answers.
    #[must_use]
    pub const fn query_id(&self) -> &'static str {
        match self {
            Self::FunctionCalledOutsideTests { .. } => "function_called_outside_tests",
            Self::SymbolExistsAtPath { .. } => "symbol_exists_at_path",
        }
    }
}

/// Recognize a bounded code query from `text` using the `code_query_cue`
/// phrases of `data/seed/fact-check-sources.lino`.
///
/// The symbol and path are the backticked spans of the text — the corpus's
/// way of naming code identifiers — and neither is invented when absent.
#[must_use]
pub fn parse_code_query(text: &str) -> Option<CodeQuery> {
    let lowered = text.to_lowercase();
    let (query_id, _) = cue_phrases()
        .into_iter()
        .filter(|(_, phrase)| lowered.contains(phrase))
        .min_by_key(|(_, phrase)| phrase.len())?;
    match query_id.as_str() {
        "function_called_outside_tests" => Some(CodeQuery::FunctionCalledOutsideTests {
            symbol: backticked_spans(text).into_iter().next()?,
        }),
        "symbol_exists_at_path" => {
            let spans = backticked_spans(text);
            let path = spans
                .iter()
                .find(|span| span.contains('/') && span.contains('.'))?;
            let symbol = spans
                .iter()
                .find(|span| !span.contains('/') && !span.is_empty())?;
            Some(CodeQuery::SymbolExistsAtPath {
                symbol: symbol.clone(),
                path: path.clone(),
            })
        }
        _ => None,
    }
}

/// Where and how often a symbol is referenced, split by test scope.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CallSiteCount {
    /// References from production code (not inside a `#[cfg(test)]` module and
    /// not from the test tree).
    pub production: usize,
    /// References from test scope.
    pub test: usize,
}

impl CallSiteCount {
    /// Whether the symbol is referenced at all.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.production > 0 || self.test > 0
    }
}

/// Count references to `symbol` in one document, split by scope.
///
/// Pure over the document text so tests can pin the scope tracking exactly;
/// [`code_evidence_for`] decides which documents are certified. A
/// `#[cfg(test)]` module is a whole `mod … { … }` block closed by an
/// unindented brace in formatted Rust, which is what the exit condition
/// relies on.
#[must_use]
pub fn count_call_sites(content: &str, symbol: &str) -> CallSiteCount {
    let mut count = CallSiteCount::default();
    if symbol.is_empty() {
        return count;
    }
    let mut in_test_module = false;
    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("#[cfg(test") {
            in_test_module = true;
            continue;
        }
        if in_test_module && raw_line.starts_with('}') {
            in_test_module = false;
            continue;
        }
        if references_symbol(trimmed, symbol) {
            if in_test_module {
                count.test += 1;
            } else {
                count.production += 1;
            }
        }
    }
    count
}

/// Whether a whole-word reference to `symbol` occurs in `line`.
fn references_symbol(line: &str, symbol: &str) -> bool {
    let mut search = 0usize;
    while let Some(found) = line[search..].find(symbol) {
        let start = search + found;
        let end = start + symbol.len();
        let boundary_before = line[..start]
            .chars()
            .next_back()
            .is_none_or(|previous| !is_symbol_char(previous));
        let boundary_after = line[end..]
            .chars()
            .next()
            .is_none_or(|next| !is_symbol_char(next));
        if boundary_before && boundary_after {
            return true;
        }
        search = end;
    }
    false
}

const fn is_symbol_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

/// Answer one bounded code query against the repository at `root`.
///
/// Every Rust file under `root` (bounded to 400 files, skipping `.git`,
/// `target`, and `node_modules`) is parsed through the meta-language network;
/// only clean, text-preserving parses are inspected. Production call sites
/// support a "called outside tests" claim; a test-only or absent symbol
/// contradicts it. Symbol existence is answered from the certified file at
/// the claimed path. With the optional engine disabled there is no
/// certification, so the collector answers nothing rather than guess.
#[must_use]
pub fn code_evidence_for(query: &CodeQuery, root: &Path) -> Vec<RelativeEvidence> {
    let tier = tier_for_context("code");
    let files = rust_files(root);
    match query {
        CodeQuery::FunctionCalledOutsideTests { symbol } => {
            let mut total = CallSiteCount::default();
            let mut first_production_site: Option<String> = None;
            for relative in &files {
                let Some(content) = read_certified(root, relative) else {
                    continue;
                };
                let mut in_file = count_call_sites(&content, symbol);
                if is_test_tree_path(relative) {
                    // A reference from the test tree is test scope even when
                    // the file itself has no #[cfg(test)] module.
                    in_file = CallSiteCount {
                        production: 0,
                        test: in_file.production + in_file.test,
                    };
                }
                if in_file.production > 0 && first_production_site.is_none() {
                    first_production_site = Some(relative.clone());
                }
                total.production += in_file.production;
                total.test += in_file.test;
            }
            if !total.any() {
                return Vec::new();
            }
            let evidence = if total.production > 0 {
                RelativeEvidence::new(
                    crate::seed::report_text(
                        "fact_check_code_references",
                        &[
                            (
                                "site",
                                first_production_site.as_deref().unwrap_or("<unlocated>"),
                            ),
                            ("production", &total.production.to_string()),
                            ("test", &total.test.to_string()),
                        ],
                    ),
                    tier,
                    Stance::Supports,
                    TruthValue::new(production_strength(total.production)),
                )
            } else {
                RelativeEvidence::new(
                    crate::seed::report_text(
                        "fact_check_code_test_only",
                        &[("symbol", symbol), ("sites", &total.test.to_string())],
                    ),
                    tier,
                    Stance::Contradicts,
                    TruthValue::new(0.9),
                )
            };
            vec![evidence]
        }
        CodeQuery::SymbolExistsAtPath { symbol, path } => {
            let exists = files
                .iter()
                .filter(|relative| relative.as_str() == path)
                .find_map(|relative| {
                    read_certified(root, relative)
                        .map(|content| references_symbol(&content, symbol))
                })
                .unwrap_or(false);
            vec![RelativeEvidence::new(
                format!("code:{path}"),
                tier,
                if exists {
                    Stance::Supports
                } else {
                    Stance::Contradicts
                },
                TruthValue::new(if exists { 1.0 } else { 0.9 }),
            )]
        }
    }
}

/// The support strength for `n` production references, saturating at `1`.
#[allow(clippy::cast_precision_loss)]
const fn production_strength(production: usize) -> f64 {
    0.1f64.mul_add(production as f64, 0.5).min(1.0)
}

/// Whether a repository-relative path sits in the test tree.
fn is_test_tree_path(relative: &str) -> bool {
    relative.starts_with("tests/") || relative.contains("/tests/")
}

/// Parse the file at `root`/`relative` through the meta-language network and
/// return its text only when the parse verified cleanly and reconstructs the
/// source — the same certification `agentic_coding::self_ast::ast_census`
/// applies before reasoning about a module.
fn read_certified(root: &Path, relative: &str) -> Option<String> {
    let content = fs::read_to_string(root.join(relative)).ok()?;
    parses_clean(&content).then_some(content)
}

/// Whether `source` parses as clean Rust through the sole CST/AST engine.
#[cfg(feature = "meta-language")]
fn parses_clean(source: &str) -> bool {
    let network = meta_language::LinkNetwork::parse(
        source,
        "rust",
        meta_language::ParseConfiguration::default(),
    );
    network.verify_full_match(None).is_clean() && network.reconstruct_text() == source
}

/// Without the optional engine there is no certification, so no code evidence.
#[cfg(not(feature = "meta-language"))]
fn parses_clean(_source: &str) -> bool {
    false
}

/// Repository-relative Rust sources under `root`, `/`-normalized and bounded
/// to 400 files so a monorepo walk stays cheap.
fn rust_files(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut queue = vec![PathBuf::from(root)];
    while let Some(directory) = queue.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if files.len() >= 400 {
                return files;
            }
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if !matches!(
                    entry.file_name().to_str(),
                    Some(".git" | "target" | "node_modules")
                ) {
                    queue.push(path);
                }
            } else if file_type.is_file()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|name| name.as_bytes().ends_with(b".rs"))
                && let Ok(relative) = path.strip_prefix(root)
            {
                files.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    files
}

/// The `code_query_cue` phrases of the seed: (query id, phrase) pairs.
fn cue_phrases() -> Vec<(String, String)> {
    crate::seed::parser::parse_lino(super::SOURCES_LINO)
        .children
        .into_iter()
        .filter(|node| node.name == "fact_check_sources")
        .flat_map(|sources| sources.children.into_iter())
        .filter(|record| record.name == "code_query_cue")
        .flat_map(|record| {
            let query = record.find_child_value("query").to_owned();
            record
                .children
                .into_iter()
                .filter(|child| child.name == "phrase")
                .map(move |child| (query.clone(), child.id))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The backtick-delimited spans of `text`, in order.
fn backticked_spans(text: &str) -> Vec<String> {
    let mut spans = Vec::new();
    let mut rest = text;
    while let Some((_, after)) = rest.split_once('`') {
        let Some((span, remainder)) = after.split_once('`') else {
            break;
        };
        if !span.trim().is_empty() {
            spans.push(span.trim().to_owned());
        }
        rest = remainder;
    }
    spans
}
