//! Captured GitHub issues, reviews, and CI runs for repository history (issue #1180).

use super::{
    BTreeMap, HistoryRules, MemoryEvent, NUMBER_PLACEHOLDER, Path, RepositoryHistoryImportError,
    effective_record, fs, pattern_number, rule_event,
};

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
        if let Some(number) = rest
            .strip_suffix("-conversation-comments")
            .and_then(parse_digits)
        {
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
/// camelCase and the raw REST API's `snake_case` spellings.
fn field_str<'a>(value: &'a serde_json::Value, names: &[&str]) -> Option<&'a str> {
    names
        .iter()
        .find_map(|name| value.get(*name).and_then(serde_json::Value::as_str))
}

fn nested_login(value: &serde_json::Value) -> String {
    for holder in ["author", "user"] {
        if let Some(login) = value.get(holder).and_then(|holder| holder.get("login"))
            && let Some(login) = login.as_str()
        {
            return login.to_owned();
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

pub(super) fn import_issues_and_pulls_inner(
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
        if let Some(updated) = &imported.updated_at
            && watermark
                .as_deref()
                .is_none_or(|current| updated.as_str() > current)
        {
            *watermark = Some(updated.clone());
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
                    if let Some(imported) = formalize_issue_or_pull(&item, is_pull, rules)
                        && imported_passes(&imported, since_updated_at)
                    {
                        record(imported, &mut watermark);
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
    entries.sort_by_key(std::fs::DirEntry::file_name);
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
        evidence.push(format!("url:{url}"));
    }
    for label in label_names(value) {
        evidence.push(format!("label:{label}"));
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
        format!("{title}\n\n{body}")
    };
    let event = rule_event(
        rule,
        format!("{}{}", rule.id_prefix, number),
        field_str(value, &["state"]).map(ToOwned::to_owned),
        content,
        stamped(value, &["createdAt", "created_at"]),
        Some(format!("issue-{number}")),
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
    let parent = format!("{parent_kind}-{parent_number}");
    let own_id = value
        .get("id")
        .and_then(serde_json::Value::as_u64)
        .map_or_else(|| index.to_string(), |id| id.to_string());
    let body = field_str(value, &["body"]).unwrap_or_default();
    let state = field_str(value, &["state"]).unwrap_or_default();
    let author = nested_login(value);
    let content = match (body.is_empty(), state.is_empty()) {
        (true, true) => author,
        (true, false) => format!("{author}: {state}"),
        _ => format!("{author}: {state}: {body}"),
    };
    let event = rule_event(
        rule,
        format!("{}{}-{}-{}", rule.id_prefix, parent, intent, own_id),
        Some(intent.to_owned()),
        content,
        stamped(
            value,
            &["submittedAt", "submitted_at", "createdAt", "created_at"],
        ),
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
            let mut content = format!("{workflow}: {conclusion}");
            for line in failing_step_lines(&run) {
                content.push('\n');
                content.push_str(&line);
            }
            let mut evidence = Vec::new();
            if let Some(head_sha) = field_str(&run, &["headSha", "head_sha"]) {
                evidence.push(format!("commit:{head_sha}"));
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
                failing_steps.push(format!("{job_name}/{step_name}"));
            }
        }
        if failing_steps.is_empty() {
            failing_steps.push(job_name.to_owned());
        }
        lines.extend(failing_steps);
    }
    lines
}
