//! Status and definition questions about this repository (issue #1180 R10).
//!
//! Two questions the #1180 probe asks are answered from the repository
//! itself, through the lineage route's handler:
//!
//! - "why does the self-development status fail?" — a seeded `status_cue`
//!   names the status. The answer reads the rule the seed's `status_rule`
//!   points at: the script that checks it (with the issue its introducing
//!   commit was made for), the commit range since the last release tag the
//!   script measures (with how many of its commits carry the ledger's session
//!   trailer), and the floor and ratcheted target the self-hosting ledger
//!   records. It states that the measurement itself is not run here.
//! - "what does `evaluate_calculation` do?" — a seeded `definition_cue` and an
//!   identifier-shaped token the self-AST census records. The answer locates
//!   the symbol (kind, file, line span), quotes its documentation comment from
//!   the source, and names the commit and issue that first wrote its name in
//!   a file of its language (`git log -S`).
//!
//! A cue alone never claims (issue #1175): the status needs the seeded status
//! phrase, and the definition needs a census symbol. Every sentence is a seed
//! response.

use std::path::Path;

use super::lineage::{introducing_issue, template, working_repository};
use super::{HistoryRules, MemoryEvent, lineage_for_path, lineage_for_symbol};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{LinoNode, parse_lino};

/// The intent of a status explanation.
const STATUS_INTENT: &str = "repository_status_explanation";
/// The intent of a symbol definition.
const DEFINITION_INTENT: &str = "repository_definition";
/// The most census symbols one definition answer describes.
const MAX_DEFINITIONS: usize = 3;

/// One symbol the self-AST census records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CensusSymbol {
    /// The census target, relative to the seed's `source_root`.
    pub target: String,
    pub kind: String,
    pub name: String,
    pub start: usize,
    pub end: usize,
}

/// The seeded phrase a cue list finds in `prompt`, if any.
fn cued_phrase(prompt: &str, cues: &[(String, Vec<String>)]) -> Option<String> {
    let lowered = prompt.to_lowercase();
    cues.iter()
        .flat_map(|(_, phrases)| phrases.iter())
        .find(|phrase| lowered.contains(&phrase.to_lowercase()))
        .cloned()
}

/// The status phrase a status question names, when it names one.
#[must_use]
pub fn status_subject(prompt: &str, rules: &HistoryRules) -> Option<String> {
    let qa = &rules.repository_qa;
    if qa.status_script.is_empty() {
        return None;
    }
    cued_phrase(prompt, &qa.status_cues)
}

/// Whether `token` is shaped like a code identifier, not a plain word.
///
/// A plain word ("parse", "new") is also a census symbol name, so only a
/// token with an underscore or an interior capital after a lower-case letter
/// is read as one.
fn identifier_shaped(token: &str) -> bool {
    let starts_like_identifier = token
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_');
    let camel = token
        .chars()
        .zip(token.chars().skip(1))
        .any(|(left, right)| left.is_ascii_lowercase() && right.is_ascii_uppercase());
    starts_like_identifier && token.len() > 1 && (token.contains('_') || camel)
}

/// The identifier-shaped tokens a definition question names, in order.
///
/// Empty unless the prompt carries a seeded definition cue.
#[must_use]
pub fn definition_subjects(prompt: &str, rules: &HistoryRules) -> Vec<String> {
    if cued_phrase(prompt, &rules.repository_qa.definition_cues).is_none() {
        return Vec::new();
    }
    let mut names: Vec<String> = Vec::new();
    for token in
        prompt.split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
    {
        if identifier_shaped(token) && !names.iter().any(|known| known == token) {
            names.push(token.to_owned());
        }
    }
    names
}

/// The symbols one census document records under its target.
fn census_document_symbols(text: &str) -> Vec<CensusSymbol> {
    let mut target = String::new();
    let mut symbols = Vec::new();
    let mut in_symbols = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("target ") {
            value.trim().clone_into(&mut target);
            continue;
        }
        if trimmed == "symbols" {
            in_symbols = true;
            continue;
        }
        if !in_symbols {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if let [kind, name, start, end] = fields.as_slice()
            && let (Ok(start), Ok(end)) = (start.parse::<usize>(), end.parse::<usize>())
        {
            symbols.push(CensusSymbol {
                target: target.clone(),
                kind: (*kind).to_owned(),
                name: (*name).to_owned(),
                start,
                end,
            });
        }
    }
    symbols
}

/// Every symbol the self-AST census under `root` records.
#[must_use]
pub fn census_symbols(root: &Path, rules: &HistoryRules) -> Vec<CensusSymbol> {
    let census = &rules.repository_qa.census_dir;
    if census.is_empty() {
        return Vec::new();
    }
    let mut pending = vec![root.join(census)];
    let mut symbols = Vec::new();
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        let mut paths: Vec<std::path::PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|found| found.path()))
            .collect();
        paths.sort();
        for path in paths {
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "lino")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                symbols.extend(census_document_symbols(&text));
            }
        }
    }
    symbols
}

/// The documentation comment directly above line `start` (1-based) of
/// `source`, joined into one paragraph.
///
/// Attribute lines between the comment and the item are skipped. `None`
/// when the item carries no comment.
#[must_use]
pub fn symbol_doc(source: &str, start: usize, rules: &HistoryRules) -> Option<String> {
    let qa = &rules.repository_qa;
    if qa.doc_prefix.is_empty() || start < 2 {
        return None;
    }
    let lines: Vec<&str> = source.lines().collect();
    let mut comment: Vec<String> = Vec::new();
    for line in lines.iter().take(start - 1).rev() {
        let trimmed = line.trim();
        if let Some(text) = trimmed.strip_prefix(qa.doc_prefix.as_str()) {
            comment.push(text.trim().to_owned());
        } else if !qa.attribute_prefix.is_empty()
            && trimmed.starts_with(qa.attribute_prefix.as_str())
        {
            if !comment.is_empty() {
                break;
            }
        } else {
            break;
        }
    }
    comment.reverse();
    let paragraph = comment
        .iter()
        .take_while(|line| !line.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    (!paragraph.is_empty()).then_some(paragraph)
}

/// The seed's "not recorded" wording for a missing issue or commit.
fn unrecorded(language: &str) -> String {
    template("repository_lineage_unrecorded", language, &[])
}

/// The short id of a formalized commit.
fn short_sha(event: &MemoryEvent, rules: &HistoryRules) -> String {
    let prefix = rules
        .record("commit")
        .map_or("", |rule| rule.id_prefix.as_str());
    event
        .id
        .strip_prefix(prefix)
        .unwrap_or(&event.id)
        .chars()
        .take(9)
        .collect()
}

/// The issue number a commit was made for, as `#n` or the unrecorded wording.
fn issue_of(event: Option<&MemoryEvent>, rules: &HistoryRules, language: &str) -> String {
    event
        .and_then(|first| introducing_issue(first, rules))
        .map_or_else(|| unrecorded(language), |number| format!("#{number}"))
}

/// Upper-case the first character, for a template that opens with a slot.
fn capitalized(text: &str) -> String {
    let mut characters = text.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(characters).collect()
    })
}

/// Describe one census symbol from its source, documentation and lineage.
#[must_use]
pub fn definition_answer(
    root: &Path,
    symbol: &CensusSymbol,
    language: &str,
    rules: &HistoryRules,
) -> String {
    let qa = &rules.repository_qa;
    let file = if qa.source_root.is_empty() {
        symbol.target.clone()
    } else {
        format!("{}/{}", qa.source_root, symbol.target)
    };
    let doc = std::fs::read_to_string(root.join(&file))
        .ok()
        .and_then(|source| symbol_doc(&source, symbol.start, rules))
        .map_or_else(
            || template("repository_definition_undocumented", language, &[]),
            |doc| {
                template(
                    "repository_definition_documented",
                    language,
                    &[("doc", &doc)],
                )
            },
        );
    // The name is searched across every file of the same language, so a
    // symbol that moved between files (or whose file moved) is traced to
    // the commit that first wrote it, not to the move.
    let pathspec = Path::new(&file).extension().map_or_else(
        || file.clone(),
        |extension| format!("*.{}", extension.to_string_lossy()),
    );
    let events = lineage_for_symbol(root, &pathspec, &symbol.name, rules).unwrap_or_default();
    let first = events.first();
    let commit = first.map_or_else(|| unrecorded(language), |event| short_sha(event, rules));
    template(
        DEFINITION_INTENT,
        language,
        &[
            ("name", &symbol.name),
            ("kind", &symbol.kind),
            ("file", &file),
            ("start", &symbol.start.to_string()),
            ("end", &symbol.end.to_string()),
            ("doc", &doc),
            ("first", &commit),
            ("issue", &issue_of(first, rules, language)),
        ],
    )
}

/// One field of the ledger's root record.
fn ledger_field<'a>(ledger: &'a LinoNode, name: &str) -> &'a str {
    ledger.find_child_value(name)
}

/// A basis-point share as a percentage with two decimals.
fn percentage(basis_points: &str) -> Option<String> {
    let value: u64 = basis_points.parse().ok()?;
    Some(format!("{}.{:02}%", value / 100, value % 100))
}

/// Run git in `root`, trimmed stdout on success.
fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Explain the self-development status rule from its script, the measured
/// commit range and the ledger, or `None` when the repository cannot say.
#[must_use]
pub fn status_answer(
    root: &Path,
    status: &str,
    language: &str,
    rules: &HistoryRules,
) -> Option<String> {
    let qa = &rules.repository_qa;
    let ledger_text = std::fs::read_to_string(root.join(&qa.status_ledger)).ok()?;
    let tree = parse_lino(&ledger_text);
    let ledger = tree.children.first()?;
    let tag = git(
        root,
        &[
            "describe",
            "--tags",
            "--match",
            &qa.status_tag_match,
            "--abbrev=0",
            "HEAD",
        ],
    )?;
    let range = format!("{tag}..HEAD");
    let commits = git(root, &["rev-list", "--count", &range])?;
    let trailer = ledger_field(ledger, "session_trailer");
    let marker = format!("{trailer}:");
    let bodies = git(root, &["log", "--format=%x1e%B", &range]).unwrap_or_default();
    let sessions = bodies
        .split('\u{1e}')
        .filter(|body| {
            body.lines()
                .any(|line| line.trim_start().starts_with(&marker))
        })
        .count();
    let newest = ledger.children.iter().rev().find(|row| {
        row.name == "release" && !ledger_field(row, "target_percentage_basis_points").is_empty()
    })?;
    let target = percentage(ledger_field(newest, "target_percentage_basis_points"))?;
    let events = lineage_for_path(root, &qa.status_script, rules).unwrap_or_default();
    let body = template(
        STATUS_INTENT,
        language,
        &[
            ("status", status),
            ("script", &qa.status_script),
            ("issue", &issue_of(events.first(), rules, language)),
            ("range", &range),
            ("commits", &commits),
            ("sessions", &sessions.to_string()),
            ("trailer", trailer),
            ("floor", ledger_field(ledger, "release_cycle_floor")),
            ("unit", ledger_field(ledger, "release_cycle_unit")),
            (
                "attribution",
                ledger_field(ledger, "release_cycle_attribution"),
            ),
            ("target", &target),
            ("target_tag", ledger_field(newest, "tag")),
            ("policy", ledger_field(ledger, "target_policy")),
            ("ledger", &qa.status_ledger),
        ],
    );
    Some(capitalized(&body))
}

/// Answer a status or definition question about the working repository, or
/// leave the prompt unclaimed.
pub(super) fn answer_repository_question(
    prompt: &str,
    rules: &HistoryRules,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language = crate::language::detect(prompt).slug();
    if let Some(status) = status_subject(prompt, rules) {
        let root = working_repository()?;
        let body = status_answer(&root, &status, language, rules)?;
        log.append(
            "history:status",
            format!("script={}", rules.repository_qa.status_script),
        );
        return Some(crate::solver_handlers::finalize_simple(
            prompt,
            log,
            STATUS_INTENT,
            "response:repository_status_explanation",
            &body,
            0.8,
        ));
    }
    let names = definition_subjects(prompt, rules);
    if names.is_empty() {
        return None;
    }
    let root = working_repository()?;
    let symbols = census_symbols(&root, rules);
    let found: Vec<&CensusSymbol> = names
        .iter()
        .flat_map(|name| symbols.iter().filter(move |symbol| &symbol.name == name))
        .take(MAX_DEFINITIONS)
        .collect();
    if found.is_empty() {
        return None;
    }
    let body = found
        .iter()
        .map(|symbol| definition_answer(&root, symbol, language, rules))
        .collect::<Vec<_>>()
        .join("\n\n");
    for symbol in &found {
        log.append(
            "history:definition",
            format!("symbol={};target={}", symbol.name, symbol.target),
        );
    }
    Some(crate::solver_handlers::finalize_simple(
        prompt,
        log,
        DEFINITION_INTENT,
        "response:repository_definition",
        &body,
        0.8,
    ))
}
