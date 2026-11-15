//! Evidence leads of a work item that names no repository file (issue #1483).
//! JavaScript twin: js/agentic/work_item_leads.mjs.
//!
//! A work item whose text names no file still carries evidence: an attached
//! screenshot, a linked log, a code identifier in backticks. Each lead is
//! either collected (a fetched document, a file found by the identifier) or
//! kept as an explicit unresolved item; nothing a lead points at is invented.
//! An image is never described: its content is unavailable to this planner.
//! A fetched document is evidence, never authority: its text is recorded, not
//! followed.

use super::planner::{Capability, fetch_arguments, tool_for};
use super::progress::{Progress, ToolAttempt};
use super::tool_result::command_argument;
use super::work_item_references::SearchAction;
use serde_json::json;
use std::sync::OnceLock;

/// At most this many fetched documents per work item.
pub(super) const MAXIMUM_DOCUMENTS: usize = 2;
/// At most this many identifiers per work item.
pub(super) const MAXIMUM_SYMBOLS: usize = 3;
const MAXIMUM_TASK_BYTES: usize = 32_768;

fn pattern(cell: &'static OnceLock<regex::Regex>, source: &str) -> &'static regex::Regex {
    cell.get_or_init(|| regex::Regex::new(source).expect("StaticLeadPattern"))
}

fn url_pattern() -> &'static regex::Regex {
    static URL: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&URL, r#"https?://[^\s<>"'`)\]]+"#)
}

fn image_suffix() -> &'static regex::Regex {
    static IMAGE: OnceLock<regex::Regex> = OnceLock::new();
    pattern(&IMAGE, r"(?i)\.(?:png|jpe?g|gif|webp|svg|bmp)(?:[?#].*)?$")
}

/// The leads of a work item.
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct WorkItemLeads {
    /// Attached or embedded images, never fetched.
    pub(super) attachments: Vec<String>,
    /// Documents a fetch can read.
    pub(super) documents: Vec<String>,
    /// Pages kept as related context (work items, runs, other repositories).
    pub(super) related: Vec<String>,
}

/// URLs of `task` in order of first mention, each once.
fn urls(task: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for matched in url_pattern().find_iter(task) {
        let url = matched
            .as_str()
            .trim_end_matches(['.', ',', ';', ':', '!', '?'])
            .to_owned();
        if !found.contains(&url) {
            found.push(url);
        }
    }
    found
}

/// Whether `url` is an image the issue attached or embedded.
fn attachment(task: &str, url: &str) -> bool {
    if url.starts_with("https://github.com/user-attachments/")
        || url.starts_with("https://user-images.githubusercontent.com/")
        || url.starts_with("https://private-user-images.githubusercontent.com/")
        || image_suffix().is_match(url)
        || task.contains(&format!("src=\"{url}"))
        || task.contains(&format!("src='{url}"))
    {
        return true;
    }
    // A Markdown image: `![text](url)`.
    let Some(link) = task.find(&format!("]({url}")) else {
        return false;
    };
    task[..link]
        .rfind('[')
        .is_some_and(|open| open > 0 && task.as_bytes()[open - 1] == b'!')
}

/// The leads of `task`: attachments (never fetched), documents a fetch can
/// read, and pages kept as related context. Blob links into a repository are
/// file references, not leads.
pub(super) fn work_item_leads(task: &str, work_item: &str) -> WorkItemLeads {
    let mut leads = WorkItemLeads::default();
    if task.len() > MAXIMUM_TASK_BYTES {
        return leads;
    }
    static BLOB: OnceLock<regex::Regex> = OnceLock::new();
    let blob = pattern(&BLOB, r"^https://github\.com/[^/]+/[^/]+/blob/");
    for url in urls(task) {
        if !work_item.is_empty() && url == work_item {
            continue;
        }
        if attachment(task, &url) {
            leads.attachments.push(url);
        } else if blob.is_match(&url) {
            continue;
        } else if url.starts_with("https://github.com/") {
            leads.related.push(url);
        } else {
            leads.documents.push(url);
        }
    }
    leads
}

/// Code identifiers written in backticks (`planned_not_executed`,
/// `Progress::scan`), at most three; each is the last segment of its path.
pub(super) fn work_item_symbols(task: &str) -> Vec<String> {
    let mut symbols: Vec<String> = Vec::new();
    if task.len() > MAXIMUM_TASK_BYTES {
        return symbols;
    }
    static CODE_SPAN: OnceLock<regex::Regex> = OnceLock::new();
    static IDENTIFIER: OnceLock<regex::Regex> = OnceLock::new();
    static COMPOUND: OnceLock<regex::Regex> = OnceLock::new();
    let code_span = pattern(&CODE_SPAN, r"`([^`\n]{1,120})`");
    let identifier = pattern(
        &IDENTIFIER,
        r"^[A-Za-z_][A-Za-z0-9_]*(?:(?:::|\.)[A-Za-z_][A-Za-z0-9_]*)*(?:\(\))?$",
    );
    let compound = pattern(&COMPOUND, r"[_.]|::|[a-z][A-Z]");
    for captures in code_span.captures_iter(task) {
        let span = captures[1].trim();
        if span.len() < 4 || span.len() > 80 || !identifier.is_match(span) {
            continue;
        }
        let name = span.strip_suffix("()").unwrap_or(span);
        // Only a compound name is an identifier before it is a word: `snake_case`, `Path::item`, `object.member`, `camelCase`.
        if !compound.is_match(name) {
            continue;
        }
        let segment = name
            .split("::")
            .flat_map(|part| part.split('.'))
            .last()
            .unwrap_or(name);
        if segment.len() < 3 || symbols.iter().any(|known| known == segment) {
            continue;
        }
        symbols.push(segment.to_owned());
        if symbols.len() == MAXIMUM_SYMBOLS {
            break;
        }
    }
    symbols
}

/// The fetch of a document lead, when a fetch tool is advertised.
pub(super) fn document_action<'a>(url: &str, tools: &[&'a str]) -> Option<SearchAction<'a>> {
    tool_for(tools, Capability::Fetch).map(|fetch| SearchAction {
        tool: fetch,
        capability: Capability::Fetch,
        arguments: fetch_arguments(url),
    })
}

fn argument(arguments: &str, key: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(arguments)
        .ok()?
        .get(key)?
        .as_str()
        .map(str::to_owned)
}

/// The latest fetch of `url`.
pub(super) fn document_attempt_for<'p>(
    progress: &'p Progress,
    url: &str,
) -> Option<&'p ToolAttempt> {
    progress.research_attempts().iter().rev().find(|attempt| {
        attempt.capability == Capability::Fetch
            && attempt
                .arguments
                .as_deref()
                .and_then(|arguments| argument(arguments, "url"))
                .is_some_and(|fetched| fetched == url)
    })
}

/// The searches for an identifier: the grep tool first, then a bounded `grep` through the shell.
pub(super) fn symbol_actions<'a>(symbol: &str, tools: &[&'a str]) -> Vec<SearchAction<'a>> {
    let mut actions = Vec::new();
    if let Some(grep) = tool_for(tools, Capability::Grep) {
        actions.push(SearchAction {
            tool: grep,
            capability: Capability::Grep,
            arguments: json!({"pattern": symbol, "query": symbol}).to_string(),
        });
    }
    if let Some(run) = tool_for(tools, Capability::Run) {
        let command = format!(
            "grep -rIlw --exclude-dir=node_modules --exclude-dir=.git -e '{symbol}' . | head -n 20"
        );
        actions.push(SearchAction {
            tool: run,
            capability: Capability::Run,
            arguments: json!({"command": command}).to_string(),
        });
    }
    actions
}

/// The latest attempt answering a symbol search, compared by pattern or command.
pub(super) fn symbol_attempt_for<'p>(
    progress: &'p Progress,
    action: &SearchAction<'_>,
) -> Option<&'p ToolAttempt> {
    let value_of = |capability: Capability, arguments: &str| {
        if capability == Capability::Run {
            command_argument(arguments)
        } else {
            argument(arguments, "pattern")
        }
    };
    let wanted = value_of(action.capability, &action.arguments);
    progress.research_attempts().iter().rev().find(|attempt| {
        attempt.capability == action.capability
            && attempt
                .arguments
                .as_deref()
                .is_some_and(|arguments| value_of(attempt.capability, arguments) == wanted)
    })
}

/// Repository files a search output names, relative to a workspace root,
/// source files first, then by depth. Lines name a path before the first `:`
/// (`grep -n`) or alone (`grep -l`, the Agent CLI's file headers).
pub(super) fn symbol_matches(output: &str, roots: &[String]) -> Vec<String> {
    static HEAD: OnceLock<regex::Regex> = OnceLock::new();
    static SOURCE_SUFFIX: OnceLock<regex::Regex> = OnceLock::new();
    static DEPENDENCY: OnceLock<regex::Regex> = OnceLock::new();
    let head = pattern(
        &HEAD,
        r"^\s*((?:\./|/)?[A-Za-z0-9_.@-][A-Za-z0-9_.@/-]*\.[A-Za-z0-9]+)(?::|\s*$)",
    );
    let source_suffix = pattern(
        &SOURCE_SUFFIX,
        r"\.(?:rs|mjs|js|ts|mts|py|lino|json|toml|yml|yaml)$",
    );
    let dependency = pattern(&DEPENDENCY, r"(?:^|/)(?:node_modules|\.git)/");
    let mut matches: Vec<String> = Vec::new();
    for line in output.split('\n') {
        let Some(captures) = head.captures(line) else {
            continue;
        };
        let mut path = &captures[1];
        path = path.strip_prefix("./").unwrap_or(path);
        for root in roots {
            let prefix = if root.ends_with('/') {
                root.clone()
            } else {
                format!("{root}/")
            };
            if let Some(relative) = path.strip_prefix(prefix.as_str()) {
                path = relative;
            }
        }
        if path.starts_with('/') || dependency.is_match(path) {
            continue;
        }
        if !path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        {
            continue;
        }
        if !matches.iter().any(|known| known == path) {
            matches.push(path.to_owned());
        }
    }
    let rank = |path: &str| usize::from(!source_suffix.is_match(path));
    matches.sort_by(|left, right| {
        rank(left.as_str())
            .cmp(&rank(right.as_str()))
            .then_with(|| left.split('/').count().cmp(&right.split('/').count()))
            .then_with(|| left.cmp(right))
    });
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leads_split_attachments_documents_and_related_pages() {
        let task = "See ![shot](https://github.com/user-attachments/assets/abc) and the log \
                    https://example.com/run.log. Related: https://github.com/o/r/issues/2 \
                    and https://github.com/o/r/blob/main/src/a.rs";
        let leads = work_item_leads(task, "https://github.com/o/r/issues/1");
        assert_eq!(
            leads.attachments,
            ["https://github.com/user-attachments/assets/abc"]
        );
        assert_eq!(leads.documents, ["https://example.com/run.log"]);
        assert_eq!(leads.related, ["https://github.com/o/r/issues/2"]);
    }

    #[test]
    fn symbols_keep_compound_identifiers_only() {
        let task = "The `planned_not_executed` reason in `Progress::scan()` and `word` and `camelCase`, `x.y`.";
        assert_eq!(
            work_item_symbols(task),
            ["planned_not_executed", "scan", "camelCase"]
        );
    }

    #[test]
    fn symbol_matches_rank_sources_first_then_depth() {
        let output = "./docs/notes.md\n/w/src/deep/a.rs:3: x\nsrc/b.mjs\nnode_modules/x/y.js\n";
        assert_eq!(
            symbol_matches(output, &["/w".to_owned()]),
            ["src/b.mjs", "src/deep/a.rs", "docs/notes.md"]
        );
    }
}
