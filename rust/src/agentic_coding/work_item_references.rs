//! Resolving the file names a work item mentions against the repository (issue #1558).
//! Twin of `js/agentic/work_item_references.mjs`.
//!
//! A name in issue text is a reference, not a path: `hive-delivery-attempt.mjs`
//! written as link text names `scripts/hive-delivery-attempt.mjs`, `Node.js`
//! names a runtime, and a link into another repository names nothing the
//! workspace holds. Each reference is resolved in this order: the exact path,
//! then a unique match by file name through the search tool, then candidates
//! ranked by how many words around the reference their directories share.
//! Every step is derived from the transcript, so an action the client already
//! answered is never planned again; a failed read moves the reference to the
//! search, and a reference nothing resolves becomes an explicit unresolved item.

use std::sync::OnceLock;

use super::planner::{Capability, tool_for};
use super::progress::{Progress, ToolAttempt, source_read_path};
use super::tool_result::command_argument;
use serde_json::json;

/// File references are evidence candidates, never shell commands or write authority.
/// With `dot_slash`, a leading `./` is dropped instead of rejecting the token.
pub(super) fn work_item_evidence_paths(task: &str, dot_slash: bool) -> Vec<&str> {
    if task.len() > 32_768 {
        return Vec::new();
    }
    static PATTERN: OnceLock<regex::Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        regex::Regex::new(
            r"^[A-Za-z0-9_-][A-Za-z0-9_./-]*\.(md|toml|json|rs|mjs|js|ts|yml|yaml|lino)$",
        )
        .expect("StaticWorkItemPathPattern")
    });
    let mut paths = Vec::new();
    for word in
        task.split(|character: char| character.is_whitespace() || "`\'\"()[],;".contains(character))
    {
        let mut path = word
            .trim_start_matches(['`', '\'', '"', '(', '['])
            .trim_end_matches(['`', '\'', '"', ')', ',', '.', ';', ':', ']']);
        if dot_slash {
            path = path.strip_prefix("./").unwrap_or(path);
        }
        if !pattern.is_match(path)
            || path.split('/').any(|part| matches!(part, "" | "." | ".."))
            || paths.contains(&path)
        {
            continue;
        }
        paths.push(path);
        if paths.len() == 8 {
            break;
        }
    }
    paths
}

/// Names that are runtimes or products before they are files (`Node.js`).
const PRODUCT_NAMES: [&str; 13] = [
    "Node.js",
    "Deno.js",
    "Bun.js",
    "Next.js",
    "Nuxt.js",
    "Vue.js",
    "React.js",
    "Express.js",
    "Three.js",
    "D3.js",
    "Ember.js",
    "Backbone.js",
    "Socket.io.js",
];
/// At most this many ranked candidates are read for one reference.
pub(super) const MAXIMUM_CANDIDATE_READS: usize = 3;
const MAXIMUM_REFERENCES: usize = 8;
const CONTEXT_WORDS: usize = 8;

/// One file named by a work item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkItemReference {
    /// The name as the text wrote it (the file name for a link).
    pub(super) name: String,
    /// The path the text points at (the linked path for a link).
    pub(super) target: String,
    /// The repository path to read, `None` for a file of another repository.
    pub(super) path: Option<String>,
    /// `owner/repository` of the link, lowercased.
    pub(super) repository: Option<String>,
    pub(super) external: bool,
    /// Lowercase words around the mention, for ranking same-named candidates.
    pub(super) context: Vec<String>,
}

/// `owner/repository` of a GitHub issue or pull request URL, lowercased.
pub(super) fn work_item_repository(work_item: &str) -> Option<String> {
    static ITEM: OnceLock<regex::Regex> = OnceLock::new();
    let item = ITEM.get_or_init(|| {
        regex::Regex::new(
            r"^https://github\.com/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/(?:issues|pull)/[0-9]+",
        )
        .expect("StaticWorkItemRepositoryPattern")
    });
    let captures = item.captures(work_item)?;
    Some(format!("{}/{}", &captures[1], &captures[2]).to_lowercase())
}

fn base_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn clean_parts(path: &str) -> bool {
    path.split('/').all(|part| !matches!(part, "" | "." | ".."))
}

/// A segment may start with a dot (`.github/workflows/…`, issue #1560); `.` and
/// `..` are never segments.
fn valid_relative(path: &str) -> bool {
    path.chars().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '/' | '-')
    }) && clean_parts(path)
}

/// A capitalized `Name.js` with no directory is a product name before it is a file.
pub(super) fn product_name(name: &str) -> bool {
    if name.contains('/') {
        return false;
    }
    if PRODUCT_NAMES.contains(&name) {
        return true;
    }
    name.strip_suffix(".js").is_some_and(|stem| {
        let mut characters = stem.chars();
        characters
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
            && characters.all(|character| character.is_ascii_alphanumeric())
    })
}

/// Lowercase word set of the text around `offset`, for ranking same-named candidates.
fn context_words(task: &str, offset: usize, length: usize) -> Vec<String> {
    let before = task[..offset].split_whitespace().collect::<Vec<_>>();
    let before = &before[before.len().saturating_sub(CONTEXT_WORDS)..];
    let after = task[offset + length..]
        .split_whitespace()
        .take(CONTEXT_WORDS);
    let mut words: Vec<String> = Vec::new();
    for word in before.iter().copied().chain(after) {
        let lower = word.to_lowercase();
        for part in lower.split(|character: char| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit())
        }) {
            if part.len() >= 3 && !words.iter().any(|known| known == part) {
                words.push(part.to_owned());
            }
        }
    }
    words
}

/// `decodeURIComponent` over the path of a link; `None` for malformed escapes or bytes.
fn percent_decoded(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = text.get(index + 1..index + 3)?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

struct Link {
    offset: usize,
    length: usize,
    path: String,
    repository: String,
}

struct Mention<'a> {
    offset: usize,
    length: usize,
    name: &'a str,
    link: Option<usize>,
}

/// The file references of `task`, in order of first mention.
pub(super) fn work_item_references(task: &str, work_item: &str) -> Vec<WorkItemReference> {
    let tokens = work_item_evidence_paths(task, true);
    if tokens.is_empty() {
        return Vec::new();
    }
    let home = work_item_repository(work_item);
    static BLOB_LINK: OnceLock<regex::Regex> = OnceLock::new();
    let blob_link = BLOB_LINK.get_or_init(|| {
        regex::Regex::new(
            r#"https://github\.com/([A-Za-z0-9_.-]+)/([A-Za-z0-9_.-]+)/blob/[^/\s)\]>"'`]+/([^\s)\]>"'`#?]+)"#,
        )
        .expect("StaticBlobLinkPattern")
    });
    let mut links = Vec::new();
    for captures in blob_link.captures_iter(task) {
        let Some(path) = percent_decoded(&captures[3]) else {
            continue;
        };
        if !valid_relative(&path) {
            continue;
        }
        let whole = captures.get(0).expect("WholeMatch");
        links.push(Link {
            offset: whole.start(),
            length: whole.len(),
            path,
            repository: format!("{}/{}", &captures[1], &captures[2]).to_lowercase(),
        });
    }
    let mut mentions = links
        .iter()
        .enumerate()
        .map(|(index, link)| Mention {
            offset: link.offset,
            length: link.length,
            name: link.path.as_str(),
            link: Some(index),
        })
        .collect::<Vec<_>>();
    let mut cursor = 0;
    for token in tokens {
        let offset = if let Some(found) = task[cursor..].find(token) {
            cursor += found + token.len();
            cursor - token.len()
        } else {
            task.find(token).unwrap_or(0)
        };
        mentions.push(Mention {
            offset,
            length: token.len(),
            name: token,
            link: None,
        });
    }
    mentions.sort_by_key(|mention| mention.offset);
    let mut references: Vec<WorkItemReference> = Vec::new();
    for mention in mentions {
        let name = mention.name;
        // Link text names the linked file: `[name.mjs](…/blob/…/dir/name.mjs)`.
        let link = mention.link.map(|index| &links[index]).or_else(|| {
            links.iter().find(|candidate| {
                candidate.path == name || candidate.path.ends_with(&format!("/{name}"))
            })
        });
        let target = link.map_or(name, |link| link.path.as_str());
        let repository = link.map(|link| link.repository.clone());
        if let Some(known) = references.iter_mut().find(|reference| {
            reference.repository == repository
                && (reference.target == target
                    || reference.name == base_name(target)
                        && (!target.contains('/') || !reference.target.contains('/')))
        }) {
            if target.contains('/') && !known.target.contains('/') {
                target.clone_into(&mut known.target);
                if !known.external {
                    known.path = Some(target.to_owned());
                }
            }
            continue;
        }
        if references.len() == MAXIMUM_REFERENCES {
            break;
        }
        let external = repository.is_some() && home.is_some() && repository != home;
        references.push(WorkItemReference {
            name: if mention.link.is_some() {
                base_name(target).to_owned()
            } else {
                name.to_owned()
            },
            target: target.to_owned(),
            path: (!external).then(|| target.to_owned()),
            repository,
            external,
            context: context_words(task, mention.offset, mention.length),
        });
    }
    references
}

/// Roots a relative path is joined with when the client binds it to an absolute path.
pub(super) fn workspace_roots(workspace: Option<String>, host_root: Option<String>) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();
    for root in [workspace, host_root].into_iter().flatten() {
        if root.starts_with('/') && !roots.contains(&root) {
            roots.push(root);
        }
    }
    roots
}

fn join_root(root: &str, path: &str) -> String {
    if root.ends_with('/') {
        format!("{root}{path}")
    } else {
        format!("{root}/{path}")
    }
}

/// Whether a read of `attempted` read the relative `path` (exactly, `./path`, or `<root>/path`).
pub(super) fn same_path(attempted: &str, path: &str, roots: &[String]) -> bool {
    attempted == path
        || attempted.strip_prefix("./") == Some(path)
        || roots.iter().any(|root| attempted == join_root(root, path))
}

/// A path from a search result, made relative to a workspace root when it lies under one.
fn relative_match<'a>(line: &'a str, roots: &[String]) -> &'a str {
    let mut path = line.trim();
    path = path.strip_prefix("./").unwrap_or(path);
    for root in roots {
        let prefix = if root.ends_with('/') {
            root.clone()
        } else {
            format!("{root}/")
        };
        if let Some(relative) = path.strip_prefix(prefix.as_str()) {
            return relative;
        }
    }
    path
}

fn dependency_or_vcs(path: &str) -> bool {
    ["node_modules/", ".git/"]
        .iter()
        .any(|directory| path.starts_with(directory) || path.contains(&format!("/{directory}")))
}

/// Search-result lines naming a file called like `reference`, outside dependency and VCS directories.
pub(super) fn search_matches(
    output: &str,
    reference: &WorkItemReference,
    roots: &[String],
) -> Vec<String> {
    let name = base_name(&reference.target);
    let mut matches: Vec<String> = Vec::new();
    for line in output.split('\n') {
        let path = relative_match(line, roots);
        if path.is_empty() || base_name(path) != name || dependency_or_vcs(path) {
            continue;
        }
        if !(valid_relative(path) || path.strip_prefix('/').is_some_and(clean_parts)) {
            continue;
        }
        if !matches.iter().any(|known| known == path) {
            matches.push(path.to_owned());
        }
    }
    matches
}

/// Candidates ordered by suffix match, then words shared with the reference's context, then depth.
pub(super) fn ranked_candidates(matches: &[String], reference: &WorkItemReference) -> Vec<String> {
    let score = |path: &str| {
        let suffix = if reference.target.contains('/')
            && (path == reference.target || path.ends_with(&format!("/{}", reference.target)))
        {
            1000
        } else {
            0
        };
        let directory = path.rsplit_once('/').map_or("", |(directory, _)| directory);
        let lower = directory.to_lowercase();
        let mut parts: Vec<&str> = Vec::new();
        for part in lower.split(|character: char| {
            !(character.is_ascii_lowercase() || character.is_ascii_digit())
        }) {
            if !part.is_empty() && !parts.contains(&part) {
                parts.push(part);
            }
        }
        suffix
            + parts
                .iter()
                .filter(|part| reference.context.iter().any(|word| word == *part))
                .count()
    };
    let mut rows = matches
        .iter()
        .map(|path| (score(path), path.split('/').count(), path.clone()))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then(left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    rows.into_iter().map(|row| row.2).collect()
}

/// One search action for a reference.
pub(super) struct SearchAction<'a> {
    pub(super) tool: &'a str,
    pub(super) capability: Capability,
    pub(super) arguments: String,
}

/// A bounded `find` for files called `name`, outside dependency and VCS directories.
fn find_command(name: &str) -> String {
    let quoted = format!("'{name}'");
    [
        "find",
        ".",
        "-type",
        "f",
        "-name",
        quoted.as_str(),
        "-not",
        "-path",
        "'*/node_modules/*'",
        "-not",
        "-path",
        "'*/.git/*'",
        "|",
        "head",
        "-n",
        "100",
    ]
    .join(" ")
}

/// The search actions for `reference`: the glob tool first, then a bounded `find` through the shell.
pub(super) fn search_actions<'a>(
    reference: &WorkItemReference,
    tools: &[&'a str],
) -> Vec<SearchAction<'a>> {
    let name = base_name(&reference.target);
    let mut actions = Vec::new();
    if let Some(glob) = tool_for(tools, Capability::Glob) {
        actions.push(SearchAction {
            tool: glob,
            capability: Capability::Glob,
            arguments: json!({"pattern": format!("**/{name}")}).to_string(),
        });
    }
    if let Some(run) = tool_for(tools, Capability::Run) {
        actions.push(SearchAction {
            tool: run,
            capability: Capability::Run,
            arguments: json!({"command": find_command(name)}).to_string(),
        });
    }
    actions
}

fn glob_pattern(arguments: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(arguments)
        .ok()?
        .get("pattern")?
        .as_str()
        .map(str::to_owned)
}

/// The latest attempt answering `action`, compared by pattern or command, not by byte-identical arguments.
pub(super) fn attempt_for<'p>(
    progress: &'p Progress,
    action: &SearchAction<'_>,
) -> Option<&'p ToolAttempt> {
    let value_of = |arguments: &str| {
        if action.capability == Capability::Run {
            command_argument(arguments)
        } else {
            glob_pattern(arguments)
        }
    };
    let wanted = value_of(&action.arguments);
    progress.research_attempts().iter().rev().find(|attempt| {
        attempt.capability == action.capability
            && attempt
                .arguments
                .as_deref()
                .is_some_and(|arguments| value_of(arguments) == wanted)
    })
}

/// The latest read attempt of `path` under any binding of its workspace roots.
pub(super) fn read_attempt_for<'p>(
    progress: &'p Progress,
    path: &str,
    roots: &[String],
) -> Option<&'p ToolAttempt> {
    progress.research_attempts().iter().rev().find(|attempt| {
        attempt.capability == Capability::Read
            && attempt
                .arguments
                .as_deref()
                .and_then(source_read_path)
                .is_some_and(|attempted| same_path(&attempted, path, roots))
    })
}

/// Whether a read answered "no such file" (Agent CLI, POSIX tools, Node).
pub(super) fn not_found_read(text: &str) -> bool {
    static NOT_FOUND: OnceLock<regex::Regex> = OnceLock::new();
    let not_found = NOT_FOUND.get_or_init(|| {
        regex::Regex::new(
            r"(?i)^\s*(?:Error: )?(?:File not found|ENOENT|No such file or directory)\b",
        )
        .expect("StaticNotFoundPattern")
    });
    static POSIX: OnceLock<regex::Regex> = OnceLock::new();
    let posix = POSIX.get_or_init(|| {
        regex::Regex::new(r"^\s*(?:cat|head|sed): [^\n]*: No such file or directory")
            .expect("StaticPosixNotFoundPattern")
    });
    not_found.is_match(text) || posix.is_match(text)
}

/// The outcome of one read attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadOutcome {
    /// The read produced no source.
    Failed,
    Complete,
    Incomplete,
}

/// `Failed` when the read produced no source, `Complete` or `Incomplete` when it did.
pub(super) fn read_outcome(attempt: &ToolAttempt) -> ReadOutcome {
    let Some(observed) = attempt.source_observation() else {
        return ReadOutcome::Failed;
    };
    if !attempt.succeeded || observed.error.is_some() || observed.absent {
        return ReadOutcome::Failed;
    }
    if !observed.complete && not_found_read(&attempt.detail) {
        return ReadOutcome::Failed;
    }
    if observed.complete && observed.source.is_some() {
        ReadOutcome::Complete
    } else {
        ReadOutcome::Incomplete
    }
}

#[cfg(test)]
#[path = "../../tests/unit/agentic-coding/work_item_references.rs"]
mod tests;
