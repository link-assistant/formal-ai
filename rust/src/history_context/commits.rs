//! Git commit capture and syntax deltas for repository history (issue #1180).

use super::*;

pub(super) fn run_git(
    repo_root: &Path,
    args: &[&str],
) -> Result<String, RepositoryHistoryImportError> {
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
    if rules
        .census_suffixes
        .iter()
        .any(|suffix| name.ends_with(suffix.as_str()))
    {
        let before = blob_at(repo_root, &format!("{}^", sha), path).unwrap_or_default();
        let after = blob_at(repo_root, sha, path).unwrap_or_default();
        return diff_census(path, &before, &after);
    }
    if rules
        .es_suffixes
        .iter()
        .any(|suffix| name.ends_with(suffix.as_str()))
    {
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
