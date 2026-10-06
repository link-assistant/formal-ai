//! Repository history as formal context for reasoning (issue #1180, E145).
//!
//! Commits, issues, pull requests, comments, reviews, and Actions runs are
//! formalized into [`MemoryEvent`] records in the same `demo_memory` store
//! every other surface reads, so the existing memory query language answers
//! lineage questions (which issue asked for X, which pull request delivered
//! it) with no schema change: `kind` carries one of `commit | issue |
//! pull_request | review | ci_run` and `evidence` carries `path:`,
//! `symbol:`, `issue:`, `pr:`, `commit:`, and `label:` pointers its
//! `CONTAINS` filter already selects on. GitHub capture stays with
//! `formal-ai github-logs`; this importer reads the JSON files that
//! collector writes, while commits come from `git log` on the working
//! repository, with changed `.rs`/`.js`/`.ts` paths diffed as syntax items
//! (`self_ast::ast_census`, `es_meta::extract`) rather than bare paths.
//! Kind spellings, id prefixes, roles, and trailer patterns live in
//! `data/seed/history-formalization.lino`; the same values ship as
//! [`HistoryRules::defaults`]. Every importer is incremental by watermark
//! ([`RepositoryHistoryCursor`]) and [`write_repository_history`]
//! deduplicates on event id, so a re-run with no new history appends zero
//! events and a crash between the store and cursor writes can only
//! re-import the last batch, never double it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::memory::MemoryEvent;
use crate::seed::parser::{escape_value, parse_lino};

/// Registered path of the rules seed this module reads.
pub const SEED_PATH: &str = "data/seed/history-formalization.lino";

/// Root record name of the incremental watermark document.
pub const CURSOR_ROOT: &str = "repository_history_cursor";

/// Placeholder the seed's trailer and merge patterns carry for the captured number.
const NUMBER_PLACEHOLDER: &str = "%number%";

/// One `git log` record, split into fields by unit separators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCommit {
    pub sha: String,
    pub author: String,
    pub committer_date: String,
    pub subject: String,
    pub body: String,
    pub changed_paths: Vec<String>,
}

/// One syntax-item delta a commit caused in one changed path.
///
/// A grammar
/// node kind for Rust paths (the `ast_census` histogram) or `token_count`
/// for ECMAScript paths, so a lineage query can ask which meanings changed,
/// not only which files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolChange {
    pub path: String,
    pub source: String,
    pub item: String,
    pub delta: i64,
}

/// Import failures carry a stable `snake_case` code plus runtime detail;
/// they never carry prose typed into this file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryHistoryImportError {
    code: String,
    detail: Option<String>,
}

impl RepositoryHistoryImportError {
    fn with_detail(code: &str, detail: String) -> Self {
        Self {
            code: code.to_owned(),
            detail: Some(detail),
        }
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }
}

impl fmt::Display for RepositoryHistoryImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(detail) => write!(f, "{}: {}", self.code, detail),
            None => f.write_str(&self.code),
        }
    }
}

impl std::error::Error for RepositoryHistoryImportError {}

/// One `record` row of the rules seed: how a source surface maps onto the
/// memory event schema.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RecordRule {
    pub kind: String,
    pub id_prefix: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// One captured-number pattern (`trailer` rows, and the `merge` row).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PatternRule {
    pub pattern: String,
    pub evidence: String,
    pub conversation: String,
}

/// The parsed content of `data/seed/history-formalization.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryRules {
    pub records: Vec<RecordRule>,
    pub trailers: Vec<PatternRule>,
    pub merge: PatternRule,
    pub path_evidence_prefix: String,
    pub symbol_evidence_prefix: String,
    pub census_suffixes: Vec<String>,
    pub es_suffixes: Vec<String>,
}

fn record_rule(
    kind: &str,
    id_prefix: &str,
    role: &str,
    content: &str,
    timestamp: &str,
) -> RecordRule {
    RecordRule {
        kind: kind.to_owned(),
        id_prefix: id_prefix.to_owned(),
        role: role.to_owned(),
        content: content.to_owned(),
        timestamp: timestamp.to_owned(),
    }
}

fn pattern_rule(pattern: &str, evidence: &str, conversation: &str) -> PatternRule {
    PatternRule {
        pattern: pattern.to_owned(),
        evidence: evidence.to_owned(),
        conversation: conversation.to_owned(),
    }
}

impl HistoryRules {
    /// The seed's exact values, for fixture repositories with no seed on
    /// disk and for validating the registered copy.
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            records: vec![
                record_rule(
                    "commit",
                    "commit:",
                    "author",
                    "subject_and_body",
                    "committer_date",
                ),
                record_rule(
                    "issue",
                    "issue:",
                    "reporter",
                    "title_and_body",
                    "created_at",
                ),
                record_rule(
                    "pull_request",
                    "pull:",
                    "author",
                    "title_and_body",
                    "created_at",
                ),
                record_rule("review", "review:", "author", "body", "created_at"),
                record_rule(
                    "ci_run",
                    "ci_run:",
                    "workflow",
                    "workflow_and_conclusion",
                    "created_at",
                ),
            ],
            trailers: vec![
                pattern_rule("Refs #%number%", "issue:%number%", "issue-%number%"),
                pattern_rule("Closes #%number%", "issue:%number%", "issue-%number%"),
            ],
            merge: pattern_rule("Merge pull request #%number%", "pr:%number%", ""),
            path_evidence_prefix: String::from("path:"),
            symbol_evidence_prefix: String::from("symbol:"),
            census_suffixes: vec![String::from(".rs")],
            es_suffixes: vec![String::from(".js"), String::from(".ts")],
        }
    }

    /// Parse the rules seed. Rows that fail validation fall back to the
    /// default of their group, so a partially edited seed still imports.
    #[must_use]
    pub fn from_seed_text(text: &str) -> Self {
        let mut rules = Self::defaults();
        for node in parse_lino(text).children {
            match node.name.as_str() {
                "record" => {
                    let kind = node.find_child_value("kind");
                    if kind.is_empty() {
                        continue;
                    }
                    let rule = record_rule(
                        kind,
                        &owned_or(node.find_child_value("id_prefix"), "commit:"),
                        &owned_or(node.find_child_value("role"), "author"),
                        node.find_child_value("content"),
                        node.find_child_value("timestamp"),
                    );
                    match rules.records.iter_mut().find(|row| row.kind == rule.kind) {
                        Some(existing) => *existing = rule,
                        None => rules.records.push(rule),
                    }
                }
                "trailer" => {
                    let pattern = node.find_child_value("pattern");
                    if pattern.is_empty() {
                        continue;
                    }
                    let rule = pattern_rule(
                        pattern,
                        node.find_child_value("evidence"),
                        node.find_child_value("conversation"),
                    );
                    // A seed row replaces the default with the same pattern,
                    // so re-stating one overrides it instead of duplicating it.
                    match rules
                        .trailers
                        .iter_mut()
                        .find(|row| row.pattern == rule.pattern)
                    {
                        Some(existing) => *existing = rule,
                        None => rules.trailers.push(rule),
                    }
                }
                "merge" => {
                    let pattern = node.find_child_value("pattern");
                    if !pattern.is_empty() {
                        rules.merge = pattern_rule(
                            pattern,
                            node.find_child_value("evidence"),
                            node.find_child_value("conversation"),
                        );
                    }
                }
                "path_evidence" => {
                    let prefix = node.find_child_value("prefix");
                    if !prefix.is_empty() {
                        prefix.clone_into(&mut rules.path_evidence_prefix);
                    }
                }
                "symbol_evidence" => {
                    let prefix = node.find_child_value("prefix");
                    if !prefix.is_empty() {
                        prefix.clone_into(&mut rules.symbol_evidence_prefix);
                    }
                }
                "source" => {
                    let suffixes: Vec<String> = node
                        .children
                        .iter()
                        .filter(|child| child.name == "applies_to" && !child.id.is_empty())
                        .map(|child| child.id.clone())
                        .collect();
                    match (suffixes.is_empty(), node.find_child_value("id")) {
                        (false, "ast_census") => rules.census_suffixes = suffixes,
                        (false, "es_meta_extract") => rules.es_suffixes = suffixes,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        rules
    }

    /// The registered embedded seed when present, else the seed on disk
    /// under `repo_root`, else the defaults.
    #[must_use]
    pub fn load(repo_root: Option<&Path>) -> Self {
        if let Some(text) = crate::seed::seed_files()
            .into_iter()
            .find(|(registered, _)| *registered == SEED_PATH)
            .map(|(_, text)| text)
        {
            return Self::from_seed_text(text);
        }
        if let Some(root) = repo_root
            && let Ok(text) = fs::read_to_string(root.join(SEED_PATH))
        {
            return Self::from_seed_text(&text);
        }
        Self::defaults()
    }

    #[must_use]
    pub fn record(&self, kind: &str) -> Option<&RecordRule> {
        self.records.iter().find(|rule| rule.kind == kind)
    }
}

/// The seed's row for `kind`, or the default row when a hand-built rules
/// value dropped it. `from_seed_text` merges over `defaults()`, so every
/// default kind is present in any parsed rules.
fn effective_record<'a>(
    rules: &'a HistoryRules,
    kind: &str,
    defaults: &'a HistoryRules,
) -> &'a RecordRule {
    rules.record(kind).unwrap_or_else(|| {
        defaults
            .records
            .iter()
            .find(|rule| rule.kind == kind)
            .unwrap_or(&defaults.records[0])
    })
}

fn owned_or(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_owned()
    } else {
        value.to_owned()
    }
}

/// Build the memory event one formalizer produces, so every surface maps
/// onto the schema through the same shape.
fn rule_event(
    rule: &RecordRule,
    id: String,
    intent: Option<String>,
    content: String,
    sent_at: String,
    conversation: Option<String>,
    evidence: Vec<String>,
) -> MemoryEvent {
    MemoryEvent {
        id,
        kind: Some(rule.kind.clone()),
        role: Some(rule.role.clone()),
        intent,
        content: Some(content),
        sent_at: Some(sent_at),
        conversation_id: conversation,
        evidence,
        ..MemoryEvent::default()
    }
}

/// Capture the `%number%` of `pattern` from `haystack`, matching the prefix
/// anywhere in the text (commit bodies still carry literal `\n` escapes, so
/// a line-prefix match would miss real trailers) and requiring the suffix
/// after the digits.
fn pattern_number(haystack: &str, pattern: &str) -> Option<String> {
    let (prefix, suffix) = pattern.split_once(NUMBER_PLACEHOLDER)?;
    let mut from = 0;
    while let Some(at) = haystack[from..].find(prefix) {
        let rest = &haystack[from + at + prefix.len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if !digits.is_empty() && rest[digits.len()..].starts_with(suffix) {
            return Some(digits);
        }
        from += at + prefix.len().max(1);
    }
    None
}

// Split by capture source to keep the issue #1180 importer within the file-size gate.
mod commits;
mod cursor;
mod github;

pub use commits::{
    diff_symbols, formalize_commit, import_commits, import_commits_for_path, parse_log_output,
};
pub use cursor::{
    RepositoryHistoryCursor, import_incremental, repository_slug, store_paths,
    write_repository_history,
};
pub use github::{import_ci_runs, import_issues_and_pulls};
