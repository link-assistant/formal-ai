//! The lineage route (issue #1180 R10): a chat question about where a
//! repository path came from is answered from the repository's own history.
//!
//! The route claims a prompt only when two things hold: the prompt carries a
//! seeded `lineage_cue` phrase, and it names a path that exists in the working
//! repository. A cue word alone never claims (issue #1175): with no existing
//! path there is nothing the history can answer, and the prompt goes on down
//! the handler table. The answer lists the path's commits from
//! `git log --follow`, oldest first, and names the issue the introducing
//! commit was made for and the pull request whose merge delivered it — the
//! same trailer and merge evidence `formal-ai repository-history import`
//! records. Every sentence is a seed response; the issue and pull-request
//! numbers are read back through the seed's own trailer and merge patterns.

use std::path::{Component, Path, PathBuf};
use std::process::Command;

use super::{HistoryRules, MemoryEvent, NUMBER_PLACEHOLDER, lineage_for_path};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

/// The intent (and response record) the route answers with.
const LINEAGE_INTENT: &str = "repository_lineage";
/// The response record for an issue or pull request the history does not record.
const UNRECORDED_INTENT: &str = "repository_lineage_unrecorded";

/// Whether `ch` can sit inside a repository path token.
const fn is_path_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_')
}

/// The repository paths a lineage question names, in prompt order.
///
/// Empty unless the prompt carries a seeded lineage cue. Otherwise every
/// maximal run of path characters that holds a `/` or a `.` (a trailing
/// sentence `.` dropped), so a path quoted in backticks, followed by a
/// question mark, or run into unspaced CJK text is still read whole. An
/// absolute path or one that climbs with `..` is never a repository path.
#[must_use]
pub fn lineage_subjects(prompt: &str, rules: &HistoryRules) -> Vec<String> {
    let lowered = prompt.to_lowercase();
    let cued = rules.lineage_cues.iter().any(|(_, phrases)| {
        phrases
            .iter()
            .any(|phrase| lowered.contains(&phrase.to_lowercase()))
    });
    if !cued {
        return Vec::new();
    }
    let mut subjects: Vec<String> = Vec::new();
    let mut current = String::new();
    for ch in prompt.chars().chain(std::iter::once(' ')) {
        if is_path_char(ch) {
            current.push(ch);
            continue;
        }
        let run = current.trim_end_matches('.').trim_start_matches('-');
        let shaped = run.contains('/') || run.contains('.');
        if shaped
            && !run.starts_with('/')
            && !Path::new(run)
                .components()
                .any(|part| matches!(part, Component::ParentDir))
            && !subjects.iter().any(|known| known == run)
        {
            subjects.push(run.to_owned());
        }
        current.clear();
    }
    subjects
}

/// The number a seed pattern such as `issue-%number%` or `pr:%number%`
/// carries in `value`, when `value` has the pattern's shape.
fn pattern_value<'a>(value: &'a str, pattern: &str) -> Option<&'a str> {
    let (prefix, suffix) = pattern.split_once(NUMBER_PLACEHOLDER)?;
    let number = value.strip_prefix(prefix)?.strip_suffix(suffix)?;
    (!number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())).then_some(number)
}

/// Fill a seed response's `{placeholder}` slots.
fn template(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, language).unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// Render the lineage of `path` from its formalized commits (chronological),
/// or `None` when the path has no history to report.
///
/// The issue is the introducing commit's trailer conversation, the pull
/// request its merge evidence; either one the history does not record is
/// said to be unrecorded, never guessed.
#[must_use]
pub fn lineage_answer(
    language: &str,
    path: &str,
    events: &[MemoryEvent],
    rules: &HistoryRules,
) -> Option<String> {
    let first = events.first()?;
    let commit_prefix = rules
        .record("commit")
        .map_or("", |rule| rule.id_prefix.as_str());
    let short = |event: &MemoryEvent| -> String {
        let sha = event.id.strip_prefix(commit_prefix).unwrap_or(&event.id);
        sha.chars().take(9).collect()
    };
    let commits: Vec<String> = events
        .iter()
        .map(|event| {
            let date: String = event
                .sent_at
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(10)
                .collect();
            let subject = event
                .content
                .as_deref()
                .and_then(|content| content.lines().next())
                .unwrap_or("");
            format!("- {} {date} {subject}", short(event))
        })
        .collect();
    let issue = rules.trailers.iter().find_map(|trailer| {
        first
            .conversation_id
            .as_deref()
            .and_then(|conversation| pattern_value(conversation, &trailer.conversation))
    });
    let pull = first
        .evidence
        .iter()
        .find_map(|evidence| pattern_value(evidence, &rules.merge.evidence));
    let unrecorded = template(UNRECORDED_INTENT, language, &[]);
    let numbered =
        |number: Option<&str>| number.map_or_else(|| unrecorded.clone(), |n| format!("#{n}"));
    let count = events.len().to_string();
    Some(template(
        LINEAGE_INTENT,
        language,
        &[
            ("path", path),
            ("count", &count),
            ("commits", &commits.join("\n")),
            ("first", &short(first)),
            ("issue", &numbered(issue)),
            ("pull", &numbered(pull)),
        ],
    ))
}

/// The top level of the working repository the process runs in.
fn working_repository() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!root.is_empty()).then(|| PathBuf::from(root))
}

/// The solver route: a cued lineage question naming an existing repository
/// path is answered from that path's `git log --follow` history.
pub fn handle_repository_lineage(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let rules = HistoryRules::load(None);
    let candidates = lineage_subjects(prompt, &rules);
    if candidates.is_empty() {
        return None;
    }
    let root = working_repository()?;
    let path = candidates
        .into_iter()
        .find(|candidate| root.join(candidate).exists())?;
    let events = lineage_for_path(&root, &path, &rules).ok()?;
    let language = crate::language::detect(prompt).slug();
    let body = lineage_answer(language, &path, &events, &rules)?;
    log.append(
        "history:lineage",
        format!("path={path};commits={}", events.len()),
    );
    for event in &events {
        log.append("history:commit", event.id.clone());
    }
    Some(crate::solver_handlers::finalize_simple(
        prompt,
        log,
        LINEAGE_INTENT,
        "response:repository_lineage",
        &body,
        0.85,
    ))
}
