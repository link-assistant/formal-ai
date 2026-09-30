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

/// One syntax-item delta a commit caused in one changed path: a grammar
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

/// Import failures carry a stable snake_case code plus runtime detail;
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
                record_rule("commit", "commit:", "author", "subject_and_body", "committer_date"),
                record_rule("issue", "issue:", "reporter", "title_and_body", "created_at"),
                record_rule("pull_request", "pull:", "author", "title_and_body", "created_at"),
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
                    match rules.trailers.iter_mut().find(|row| row.pattern == rule.pattern) {
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
                        rules.path_evidence_prefix = prefix.to_owned();
                    }
                }
                "symbol_evidence" => {
                    let prefix = node.find_child_value("prefix");
                    if !prefix.is_empty() {
                        rules.symbol_evidence_prefix = prefix.to_owned();
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
        if let Some(root) = repo_root {
            if let Ok(text) = fs::read_to_string(root.join(SEED_PATH)) {
                return Self::from_seed_text(&text);
            }
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
    let Some((prefix, suffix)) = pattern.split_once(NUMBER_PLACEHOLDER) else {
        return None;
    };
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

fn run_git(repo_root: &Path, args: &[&str]) -> Result<String, RepositoryHistoryImportError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| {
            RepositoryHistoryImportError::with_detail("git_spawn_failed", format!("{error}"))
        })?;
    if !output.status.success() {
        return Err(RepositoryHistoryImportError::with_detail(
            "git_command_failed",
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// `git log` format: record separator, five unit-separated fields, unit
/// separator, then the `--name-only` paths.
const LOG_FORMAT: &str = "\u{1e}%H\u{1f}%an\u{1f}%cI\u{1f}%s\u{1f}%b\u{1f}";

/// Parse `git log --format=<LOG_FORMAT> --name-only` output into raw
/// commits in chronological order.
#[must_use]
pub fn parse_log_output(text: &str) -> Vec<RawCommit> {
    let mut commits = Vec::new();
    for chunk in text.split('\u{1e}') {
        let fields: Vec<&str> = chunk.split('\u{1f}').collect();
        if fields.len() < 6 {
            continue;
        }
        let changed_paths = fields[5]
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        commits.push(RawCommit {
            sha: fields[0].trim().to_owned(),
            author: fields[1].trim().to_owned(),
            committer_date: fields[2].trim().to_owned(),
            subject: fields[3].trim().to_owned(),
            body: fields[4].trim_end().to_owned(),
            changed_paths,
        });
    }
    commits.reverse();
    commits
}

/// Import commits newer than `since_sha` (all of history when `None`) as
/// memory events, chronological.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when `git log` cannot run.
pub fn import_commits(
    repo_root: &Path,
    since_sha: Option<&str>,
    rules: &HistoryRules,
) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError> {
    import_commits_impl(repo_root, since_sha, None, rules)
}

/// Import the history of one path (`git log --follow`) as memory events,
/// chronological. This is the lineage surface R1180-8 pins.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when `git log` cannot run.
pub fn import_commits_for_path(
    repo_root: &Path,
    since_sha: Option<&str>,
    path: &str,
    rules: &HistoryRules,
) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError> {
    import_commits_impl(repo_root, since_sha, Some(path), rules)
}

fn import_commits_impl(
    repo_root: &Path,
    since_sha: Option<&str>,
    path: Option<&str>,
    rules: &HistoryRules,
) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError> {
    let range = since_sha.map_or_else(|| String::from("HEAD"), |sha| format!("{}..HEAD", sha));
    let mut args: Vec<String> = vec![
        String::from("log"),
        String::from("--no-show-signature"),
        String::from(LOG_FORMAT),
        String::from("--name-only"),
    ];
    if path.is_some() {
        args.push(String::from("--follow"));
    }
    args.push(range);
    if let Some(path) = path {
        args.push(String::from("--"));
        args.push(path.to_owned());
    }
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let raws = parse_log_output(&run_git(repo_root, &arg_refs)?);
    Ok(raws
        .iter()
        .map(|raw| formalize_raw_commit(repo_root, raw, rules))
        .collect())
}

/// Formalize one raw commit: symbol diff per changed path, merge-commit
/// resolution, trailer scan, then the record mapping.
fn formalize_raw_commit(repo_root: &Path, raw: &RawCommit, rules: &HistoryRules) -> MemoryEvent {
    let mut symbols = Vec::new();
    for path in &raw.changed_paths {
        symbols.extend(diff_symbols(repo_root, &raw.sha, path, rules));
    }
    let merge = merge_subject_for(repo_root, &raw.sha);
    formalize_commit(raw, &symbols, merge.as_deref(), rules)
}

/// Map one raw commit onto a memory event using the seed's `commit` record.
/// `merge` is the subject of the merge commit that delivered this commit
/// into the default branch (when one exists), scanned for the merge
/// pattern's pull-request number.
#[must_use]
pub fn formalize_commit(
    raw: &RawCommit,
    symbols: &[SymbolChange],
    merge: Option<&str>,
    rules: &HistoryRules,
) -> MemoryEvent {
    let defaults = HistoryRules::defaults();
    let rule = effective_record(rules, "commit", &defaults);
    let mut evidence = Vec::new();
    for path in &raw.changed_paths {
        evidence.push(format!("{}{}", rules.path_evidence_prefix, path));
    }
    let mut seen_symbols = BTreeSet::new();
    for change in symbols {
        if seen_symbols.insert(change.item.clone()) {
            evidence.push(format!("{}{}", rules.symbol_evidence_prefix, change.item));
        }
    }
    let mut conversation = None;
    for trailer in &rules.trailers {
        if let Some(number) = pattern_number(&raw.body, &trailer.pattern) {
            let filled = trailer.evidence.replace(NUMBER_PLACEHOLDER, &number);
            if !filled.is_empty() && !evidence.contains(&filled) {
                evidence.push(filled);
            }
            if conversation.is_none() && !trailer.conversation.is_empty() {
                conversation = Some(trailer.conversation.replace(NUMBER_PLACEHOLDER, &number));
            }
        }
    }
    if let Some(subject) = merge {
        if let Some(number) = pattern_number(subject, &rules.merge.pattern) {
            let filled = rules.merge.evidence.replace(NUMBER_PLACEHOLDER, &number);
            if !filled.is_empty() && !evidence.contains(&filled) {
                evidence.push(filled);
            }
        }
    }
    let content = if raw.body.is_empty() {
        raw.subject.clone()
    } else {
        format!("{}\n\n{}", raw.subject, raw.body)
    };
    rule_event(
        rule,
        format!("{}{}", rule.id_prefix, raw.sha),
        None,
        content,
        raw.committer_date.clone(),
        conversation,
        evidence,
    )
}

/// The subject of the oldest merge commit that has `sha` as an ancestor —
/// for pull-request workflows this is the merge that delivered the commit.
/// Commits pushed directly to the default branch have none.
fn merge_subject_for(repo_root: &Path, sha: &str) -> Option<String> {
    let range = format!("{}..HEAD", sha);
    let out = run_git(
        repo_root,
        &["log", "--merges", "--ancestry-path", "--format=%s", &range],
    )
    .ok()?;
    out.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .next_back()
        .map(ToOwned::to_owned)
}

/// The blob of `path` at `rev`, or `None` when it did not exist there
/// (added files, the root commit's parent). Subprocess failures are read
/// the same way: an unreadable pre-image is no pre-image.
fn blob_at(repo_root: &Path, rev: &str, path: &str) -> Option<String> {
    let spec = format!("{}:{}", rev, path);
    run_git(repo_root, &["show", &spec]).ok()
}

/// Diff one changed path's syntax items across the commit.
#[must_use]
pub fn diff_symbols(
    repo_root: &Path,
    sha: &str,
    path: &str,
    rules: &HistoryRules,
) -> Vec<SymbolChange> {
    let name = path.to_ascii_lowercase();
    if rules.census_suffixes.iter().any(|suffix| name.ends_with(suffix.as_str())) {
        let before = blob_at(repo_root, &format!("{}^", sha), path).unwrap_or_default();
        let after = blob_at(repo_root, sha, path).unwrap_or_default();
        return diff_census(path, &before, &after);
    }
    if rules.es_suffixes.iter().any(|suffix| name.ends_with(suffix.as_str())) {
        let before = blob_at(repo_root, &format!("{}^", sha), path).unwrap_or_default();
        let after = blob_at(repo_root, sha, path).unwrap_or_default();
        return diff_es(path, &before, &after);
    }
    Vec::new()
}

fn diff_census(path: &str, before: &str, after: &str) -> Vec<SymbolChange> {
    let mut histogram = BTreeMap::new();
    for (kind, count) in census_kinds(before) {
        histogram.insert(kind, -count);
    }
    for (kind, count) in census_kinds(after) {
        *histogram.entry(kind).or_insert(0) += count;
    }
    histogram
        .into_iter()
        .filter(|(_, delta)| *delta != 0)
        .map(|(item, delta)| SymbolChange {
            path: path.to_owned(),
            source: String::from("ast_census"),
            item,
            delta,
        })
        .collect()
}

/// The named-node histogram of the Rust census. Without the `meta-language`
/// feature the census is unavailable and no symbols are recorded.
#[cfg(feature = "meta-language")]
fn census_kinds(source: &str) -> Vec<(String, i64)> {
    crate::agentic_coding::self_ast::ast_census(source)
        .node_kinds
        .into_iter()
        .map(|count| (count.kind, i64::try_from(count.count).unwrap_or(0)))
        .collect()
}

#[cfg(not(feature = "meta-language"))]
fn census_kinds(_source: &str) -> Vec<(String, i64)> {
    Vec::new()
}

fn diff_es(path: &str, before: &str, after: &str) -> Vec<SymbolChange> {
    let lower = path.to_ascii_lowercase();
    let language = if lower.ends_with(".ts") {
        crate::es_meta::SourceLanguage::TypeScript
    } else {
        crate::es_meta::SourceLanguage::JavaScript
    };
    let tokens = |source: &str| {
        crate::es_meta::extract(path, language, source)
            .map(|document| i64::try_from(document.token_count).unwrap_or(0))
            .unwrap_or(0)
    };
    let (before_tokens, after_tokens) = (tokens(before), tokens(after));
    if before_tokens == after_tokens {
        return Vec::new();
    }
    vec![SymbolChange {
        path: path.to_owned(),
        source: String::from("es_meta_extract"),
        item: String::from("token_count"),
        delta: after_tokens - before_tokens,
    }]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogFile {
    IssueList,
    PullList,
    RunList,
    Issue(u64),
    IssueComments(u64),
    Pull(u64),
    PullConversationComments(u64),
    PullReviewComments(u64),
    PullReviews(u64),
    Run(u64),
    Other,
}

fn parse_digits(text: &str) -> Option<u64> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<u64>().ok()
}

/// The github-logs file shapes (`rust/src/github_logs.rs`) this importer
/// consumes.
fn classify_log_file(name: &str) -> LogFile {
    let Some(stem) = name.strip_suffix(".json") else {
        return LogFile::Other;
    };
    match stem {
        "issues-recent" => return LogFile::IssueList,
        "pulls-recent" => return LogFile::PullList,
        "actions-runs-recent" => return LogFile::RunList,
        _ => {}
    }
    if let Some(rest) = stem.strip_prefix("issue-") {
        if let Some(number) = rest.strip_suffix("-comments").and_then(parse_digits) {
            return LogFile::IssueComments(number);
        }
        if let Some(number) = parse_digits(rest) {
            return LogFile::Issue(number);
        }
    }
    if let Some(rest) = stem.strip_prefix("pr-") {
        if let Some(number) = rest.strip_suffix("-conversation-comments").and_then(parse_digits) {
            return LogFile::PullConversationComments(number);
        }
        if let Some(number) = rest.strip_suffix("-review-comments").and_then(parse_digits) {
            return LogFile::PullReviewComments(number);
        }
        if let Some(number) = rest.strip_suffix("-reviews").and_then(parse_digits) {
            return LogFile::PullReviews(number);
        }
        if let Some(number) = parse_digits(rest) {
            return LogFile::Pull(number);
        }
    }
    if let Some(number) = stem.strip_prefix("run-").and_then(parse_digits) {
        return LogFile::Run(number);
    }
    LogFile::Other
}

fn json_root(text: &str) -> Option<serde_json::Value> {
    serde_json::from_str(text).ok()
}

fn value_items(root: serde_json::Value) -> Vec<serde_json::Value> {
    match root {
        serde_json::Value::Array(items) => items,
        value => vec![value],
    }
}

/// The first present field among `names`, covering both the gh CLI's
/// camelCase and the raw REST API's snake_case spellings.
fn field_str<'a>(value: &'a serde_json::Value, names: &[&str]) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| value.get(*name).and_then(serde_json::Value::as_str))
}

fn nested_login(value: &serde_json::Value) -> String {
    for holder in ["author", "user"] {
        if let Some(login) = value.get(holder).and_then(|holder| holder.get("login")) {
            if let Some(login) = login.as_str() {
                return login.to_owned();
            }
        }
    }
    String::new()
}

fn label_names(value: &serde_json::Value) -> Vec<String> {
    value
        .get("labels")
        .and_then(serde_json::Value::as_array)
        .map(|labels| {
            labels
                .iter()
                .filter_map(|label| label.get("name").and_then(serde_json::Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// One imported issue/PR/review plus the watermark field it advances.
struct ImportedIssuePr {
    event: MemoryEvent,
    updated_at: Option<String>,
}

/// Import issues and pull requests (with their comments and reviews) from a
/// github-logs output directory as memory events, filtered on
/// `updatedAt > since_updated_at`.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when the directory cannot be read.
pub fn import_issues_and_pulls(
    logs_dir: &Path,
    since_updated_at: Option<&str>,
    rules: &HistoryRules,
) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError> {
    let (events, _) = import_issues_and_pulls_inner(logs_dir, since_updated_at, rules)?;
    Ok(events)
}

fn import_issues_and_pulls_inner(
    logs_dir: &Path,
    since_updated_at: Option<&str>,
    rules: &HistoryRules,
) -> Result<(Vec<MemoryEvent>, Option<String>), RepositoryHistoryImportError> {
    let mut by_id: BTreeMap<String, MemoryEvent> = BTreeMap::new();
    let mut watermark: Option<String> = None;
    // List rows and view files describe the same records; the longer
    // content (the view) wins, and the watermark tracks the newest
    // updatedAt that passed.
    let mut record = |imported: ImportedIssuePr, watermark: &mut Option<String>| {
        if let Some(updated) = &imported.updated_at {
            if watermark.as_deref().is_none_or(|current| *updated > current) {
                *watermark = Some(updated.clone());
            }
        }
        let id = imported.event.id.clone();
        match by_id.get(&id) {
            Some(existing) => {
                let old = existing.content.as_deref().map_or(0, str::len);
                let new = imported.event.content.as_deref().map_or(0, str::len);
                if new > old {
                    by_id.insert(id, imported.event);
                }
            }
            None => {
                by_id.insert(id, imported.event);
            }
        }
    };
    for entry in sorted_entries(logs_dir)? {
        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let name = entry.file_name().into_string().unwrap_or_default();
        let classified = classify_log_file(&name);
        if matches!(
            classified,
            LogFile::IssueList | LogFile::Issue(_) | LogFile::PullList | LogFile::Pull(_)
        ) {
            let is_pull = matches!(classified, LogFile::PullList | LogFile::Pull(_));
            if let Some(root) = json_root(&text) {
                for item in value_items(root) {
                    if let Some(imported) = formalize_issue_or_pull(&item, is_pull, rules) {
                        if imported_passes(&imported, since_updated_at) {
                            record(imported, &mut watermark);
                        }
                    }
                }
            }
            continue;
        }
        let batch = match classified {
            LogFile::IssueComments(number) => comment_batch(&text, number, false, rules),
            LogFile::PullConversationComments(number) => comment_batch(&text, number, true, rules),
            LogFile::PullReviewComments(number) => {
                review_batch(&text, number, "review_comment", rules)
            }
            LogFile::PullReviews(number) => review_batch(&text, number, "review_decision", rules),
            _ => Vec::new(),
        };
        for imported in batch {
            record(imported, &mut watermark);
        }
    }
    Ok((by_id.into_values().collect(), watermark))
}

/// The issue and PR comment threads of one captured surface.
fn comment_batch(
    text: &str,
    number: u64,
    is_pull: bool,
    rules: &HistoryRules,
) -> Vec<ImportedIssuePr> {
    review_items(text)
        .enumerate()
        .map(|(index, item)| formalize_review(&item, number, is_pull, "comment", index, rules))
        .collect()
}

/// The review-level records (decisions or code comments) of one PR.
fn review_batch(
    text: &str,
    number: u64,
    intent: &str,
    rules: &HistoryRules,
) -> Vec<ImportedIssuePr> {
    review_items(text)
        .enumerate()
        .map(|(index, item)| formalize_review(&item, number, true, intent, index, rules))
        .collect()
}

fn review_items(text: &str) -> impl Iterator<Item = serde_json::Value> {
    json_root(text)
        .and_then(|root| root.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
}

fn imported_passes(imported: &ImportedIssuePr, since_updated_at: Option<&str>) -> bool {
    match (since_updated_at, &imported.updated_at) {
        (Some(watermark), Some(updated)) => updated.as_str() > watermark,
        (Some(_), None) => false,
        (None, _) => true,
    }
}

fn sorted_entries(dir: &Path) -> Result<Vec<fs::DirEntry>, RepositoryHistoryImportError> {
    let mut entries = Vec::new();
    let read = fs::read_dir(dir).map_err(|error| {
        RepositoryHistoryImportError::with_detail("logs_dir_read_failed", format!("{error}"))
    })?;
    for entry in read.flatten() {
        if entry.file_type().is_ok_and(|kind| !kind.is_dir()) {
            entries.push(entry);
        }
    }
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

fn formalize_issue_or_pull(
    value: &serde_json::Value,
    is_pull: bool,
    rules: &HistoryRules,
) -> Option<ImportedIssuePr> {
    let number = value.get("number")?.as_u64()?;
    let kind = if is_pull { "pull_request" } else { "issue" };
    let defaults = HistoryRules::defaults();
    let rule = effective_record(rules, kind, &defaults);
    let title = field_str(value, &["title"]).unwrap_or_default();
    let body = field_str(value, &["body"]).unwrap_or_default();
    let mut evidence = Vec::new();
    if let Some(url) = field_str(value, &["url"]) {
        evidence.push(format!("url:{}", url));
    }
    for label in label_names(value) {
        evidence.push(format!("label:{}", label));
    }
    for trailer in &rules.trailers {
        if let Some(number) = pattern_number(body, &trailer.pattern) {
            let filled = trailer.evidence.replace(NUMBER_PLACEHOLDER, &number);
            if !filled.is_empty() {
                evidence.push(filled);
            }
        }
    }
    let content = if body.is_empty() {
        title.to_owned()
    } else {
        format!("{}\n\n{}", title, body)
    };
    let event = rule_event(
        rule,
        format!("{}{}", rule.id_prefix, number),
        field_str(value, &["state"]).map(ToOwned::to_owned),
        content,
        stamped(value, &["createdAt", "created_at"]),
        Some(format!("issue-{}", number)),
        evidence,
    );
    Some(ImportedIssuePr {
        event,
        updated_at: field_str(value, &["updatedAt", "updated_at"]).map(ToOwned::to_owned),
    })
}

fn stamped(value: &serde_json::Value, names: &[&str]) -> String {
    field_str(value, names).unwrap_or_default().to_owned()
}

fn formalize_review(
    value: &serde_json::Value,
    parent_number: u64,
    parent_is_pull: bool,
    intent: &str,
    index: usize,
    rules: &HistoryRules,
) -> ImportedIssuePr {
    let defaults = HistoryRules::defaults();
    let rule = effective_record(rules, "review", &defaults);
    let parent_kind = if parent_is_pull { "pr" } else { "issue" };
    let parent = format!("{}-{}", parent_kind, parent_number);
    let own_id = value
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .map_or_else(|| index.to_string(), |id| id.to_string());
    let body = field_str(value, &["body"]).unwrap_or_default();
    let state = field_str(value, &["state"]).unwrap_or_default();
    let author = nested_login(value);
    let content = match (body.is_empty(), state.is_empty()) {
        (true, true) => author,
        (true, false) => format!("{}: {}", author, state),
        _ => format!("{}: {}: {}", author, state, body),
    };
    let event = rule_event(
        rule,
        format!("{}{}-{}-{}", rule.id_prefix, parent, intent, own_id),
        Some(intent.to_owned()),
        content,
        stamped(value, &["submittedAt", "submitted_at", "createdAt", "created_at"]),
        Some(parent),
        vec![format!("{}:{}", parent_kind, parent_number)],
    );
    ImportedIssuePr {
        event,
        updated_at: field_str(value, &["updatedAt", "updated_at"]).map(ToOwned::to_owned),
    }
}

/// Import Actions runs from a github-logs output directory, filtered on
/// `databaseId > since_run_id`.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when the directory cannot be read.
pub fn import_ci_runs(
    logs_dir: &Path,
    since_run_id: Option<u64>,
    rules: &HistoryRules,
) -> Result<Vec<MemoryEvent>, RepositoryHistoryImportError> {
    let defaults = HistoryRules::defaults();
    let rule = effective_record(rules, "ci_run", &defaults);
    let mut by_id: BTreeMap<String, MemoryEvent> = BTreeMap::new();
    for entry in sorted_entries(logs_dir)? {
        match classify_log_file(&entry.file_name().to_string_lossy()) {
            LogFile::RunList | LogFile::Run(_) => {}
            _ => continue,
        }
        let Ok(text) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Some(root) = json_root(&text) else {
            continue;
        };
        for run in value_items(root) {
            let Some(database_id) = run.get("databaseId").and_then(serde_json::Value::as_u64)
            else {
                continue;
            };
            if since_run_id.is_some_and(|watermark| database_id <= watermark) {
                continue;
            }
            let workflow = field_str(&run, &["workflowName", "workflow_name"]).unwrap_or_default();
            let conclusion = field_str(&run, &["conclusion"]).unwrap_or_default();
            let mut content = format!("{}: {}", workflow, conclusion);
            for line in failing_step_lines(&run) {
                content.push('\n');
                content.push_str(&line);
            }
            let mut evidence = Vec::new();
            if let Some(head_sha) = field_str(&run, &["headSha", "head_sha"]) {
                evidence.push(format!("commit:{}", head_sha));
            }
            let event = rule_event(
                rule,
                format!("{}{}", rule.id_prefix, database_id),
                field_str(&run, &["event"]).map(ToOwned::to_owned),
                content,
                stamped(&run, &["createdAt", "created_at"]),
                None,
                evidence,
            );
            // The pinned run views carry the richer jobs field, so longer
            // content wins over the list row.
            keep_richer(&mut by_id, event);
        }
    }
    Ok(by_id.into_values().collect())
}

/// Insert `event` unless a row with the same id already carries longer
/// content.
fn keep_richer(by_id: &mut BTreeMap<String, MemoryEvent>, event: MemoryEvent) {
    let old = by_id
        .get(&event.id)
        .and_then(|existing| existing.content.as_deref())
        .map_or(0, str::len);
    if event.content.as_deref().map_or(0, str::len) > old {
        by_id.insert(event.id.clone(), event);
    }
}

/// The failing job and step names of a run, from the `jobs` field the run
/// view capture selects.
fn failing_step_lines(run: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    let Some(jobs) = run.get("jobs").and_then(serde_json::Value::as_array) else {
        return lines;
    };
    for job in jobs {
        if field_str(job, &["conclusion"]) != Some("failure") {
            continue;
        }
        let job_name = field_str(job, &["name"]).unwrap_or_default();
        let mut failing_steps = Vec::new();
        for step in job
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            if field_str(step, &["conclusion"]) == Some("failure") {
                let step_name = field_str(step, &["name"]).unwrap_or_default();
                failing_steps.push(format!("{}/{}", job_name, step_name));
            }
        }
        if failing_steps.is_empty() {
            failing_steps.push(job_name.to_owned());
        }
        lines.extend(failing_steps);
    }
    lines
}

/// Append `events` to the store at `store_path`, skipping ids already
/// present, and return how many were appended.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when the store cannot be read
/// or written.
pub fn write_repository_history(
    events: &[MemoryEvent],
    store_path: &Path,
) -> Result<usize, RepositoryHistoryImportError> {
    let io = |error: std::io::Error| {
        RepositoryHistoryImportError::with_detail("memory_store_io", format!("{error}"))
    };
    let mut store = crate::memory::MemoryStore::load_from_file(store_path).map_err(io)?;
    let existing: BTreeSet<String> = store.events().iter().map(|event| event.id.clone()).collect();
    let mut appended = 0;
    for event in events {
        if existing.contains(&event.id) {
            continue;
        }
        store.append(event.clone());
        appended += 1;
    }
    store.save_to_file(store_path).map_err(io)?;
    Ok(appended)
}

/// The incremental watermark: the last commit sha, the newest issue/PR
/// `updatedAt`, and the highest run `databaseId` this importer has stored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepositoryHistoryCursor {
    pub last_commit_sha: Option<String>,
    pub last_issue_pr_updated_at: Option<String>,
    pub last_ci_run_database_id: Option<u64>,
}

impl RepositoryHistoryCursor {
    /// Read the cursor document at `path`; a missing file is a fresh start.
    #[must_use]
    pub fn read(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        let tree = parse_lino(&text);
        let Some(node) = tree.children.iter().find(|child| child.name == CURSOR_ROOT) else {
            return Self::default();
        };
        Self {
            last_commit_sha: optional_value(node.find_child_value("last_commit_sha")),
            last_issue_pr_updated_at: optional_value(
                node.find_child_value("last_issue_pr_updated_at"),
            ),
            last_ci_run_database_id: node
                .find_child_value("last_ci_run_database_id")
                .parse::<u64>()
                .ok(),
        }
    }

    /// Write the cursor document to `path`, creating parent directories.
    ///
    /// # Errors
    ///
    /// Returns [`RepositoryHistoryImportError`] on a filesystem failure.
    pub fn write(&self, path: &Path) -> Result<(), RepositoryHistoryImportError> {
        fn row(out: &mut String, name: &str, value: &str) {
            use std::fmt::Write as _;
            let _ = writeln!(out, "  {} \"{}\"", name, escape_value(value));
        }
        let mut out = format!("{}\n", CURSOR_ROOT);
        if let Some(sha) = &self.last_commit_sha {
            row(&mut out, "last_commit_sha", sha);
        }
        if let Some(updated) = &self.last_issue_pr_updated_at {
            row(&mut out, "last_issue_pr_updated_at", updated);
        }
        if let Some(run) = self.last_ci_run_database_id {
            row(&mut out, "last_ci_run_database_id", &run.to_string());
        }
        let io = |error: std::io::Error| {
            RepositoryHistoryImportError::with_detail("cursor_write_failed", format!("{error}"))
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        fs::write(path, out).map_err(io)
    }

    /// Advance over chronologically imported events: the last commit wins,
    /// and run ids take the maximum.
    pub fn advance(&mut self, events: &[MemoryEvent], rules: &HistoryRules) {
        let defaults = HistoryRules::defaults();
        let (Some(commit), Some(ci)) = (
            effective_record(rules, "commit", &defaults),
            effective_record(rules, "ci_run", &defaults),
        ) else {
            return;
        };
        for event in events {
            let kind = event.kind.as_deref();
            if kind == Some(commit.kind.as_str()) {
                if let Some(sha) = event.id.strip_prefix(commit.id_prefix.as_str()) {
                    self.last_commit_sha = Some(sha.to_owned());
                }
            } else if kind == Some(ci.kind.as_str()) {
                if let Some(number) = event
                    .id
                    .strip_prefix(ci.id_prefix.as_str())
                    .and_then(|number| number.parse::<u64>().ok())
                {
                    self.last_ci_run_database_id = Some(
                        self.last_ci_run_database_id.map_or(number, |current| current.max(number)),
                    );
                }
            }
        }
    }
}

fn optional_value(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

/// The `owner-name` slug of the repository at `repo_root`, from its origin
/// remote; `local-repository` when there is none.
#[must_use]
pub fn repository_slug(repo_root: &Path) -> String {
    let Ok(url) = run_git(repo_root, &["config", "--get", "remote.origin.url"]) else {
        return String::from("local-repository");
    };
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::from("local-repository");
    }
    let without_git = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    let segments: Vec<&str> = without_git
        .split(['/', ':'])
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.len() >= 2 {
        return format!("{}-{}", segments[segments.len() - 2], segments[segments.len() - 1]);
    }
    String::from("local-repository")
}

/// Where this importer persists a repository's history store and cursor.
#[must_use]
pub fn store_paths(memory_dir: &Path, repo_root: &Path) -> (PathBuf, PathBuf) {
    let base = memory_dir
        .join("repository-history")
        .join(repository_slug(repo_root));
    (base.join("events.lino"), base.join("cursor.lino"))
}

/// One incremental import pass: commits from the working repository, then
/// issues/pull requests/reviews and Actions runs from a github-logs
/// directory (when given), appended to the store under `memory_dir`.
/// Returns the number of events appended.
///
/// # Errors
///
/// Returns [`RepositoryHistoryImportError`] when a source or the store fails.
pub fn import_incremental(
    repo_root: &Path,
    logs_dir: Option<&Path>,
    memory_dir: &Path,
    rules: &HistoryRules,
) -> Result<usize, RepositoryHistoryImportError> {
    let (store_path, cursor_path) = store_paths(memory_dir, repo_root);
    let mut cursor = RepositoryHistoryCursor::read(&cursor_path);
    let mut events = import_commits(repo_root, cursor.last_commit_sha.as_deref(), rules)?;
    let mut issue_watermark = cursor.last_issue_pr_updated_at.clone();
    if let Some(dir) = logs_dir {
        let (imported, watermark) =
            import_issues_and_pulls_inner(dir, cursor.last_issue_pr_updated_at.as_deref(), rules)?;
        events.extend(imported);
        if watermark.is_some() {
            issue_watermark = watermark;
        }
        events.extend(import_ci_runs(dir, cursor.last_ci_run_database_id, rules)?);
    }
    let appended = write_repository_history(&events, &store_path)?;
    cursor.advance(&events, rules);
    cursor.last_issue_pr_updated_at = issue_watermark;
    cursor.write(&cursor_path)?;
    Ok(appended)
}
