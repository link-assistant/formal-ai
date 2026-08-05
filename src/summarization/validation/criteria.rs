//! The per-criterion checks and the metric's independent `CommonMark` oracle.
//!
//! Split out of [`super`] to keep each file inside the repository's own
//! thousand-line ceiling; the criteria are unchanged.

use std::collections::BTreeSet;

use super::super::file::{display_file_format, RepositoryFileFormalization};
use super::super::{SummarizationConfig, SummarizationMode};
use super::{formalize_repository_file, CriterionOutcome};

pub(super) fn check_identity(path: &str, summary: &str) -> CriterionOutcome {
    let passed = summary.contains(path);
    outcome(
        "identity_names_path",
        true,
        passed,
        evidence(&[("path", path.to_owned())]),
    )
}

pub(super) fn check_format(
    formalized: &RepositoryFileFormalization,
    summary: &str,
) -> CriterionOutcome {
    let label = display_file_format(&formalized.format);
    let passed = summary.contains(label);
    outcome(
        "format_declared",
        true,
        passed,
        evidence(&[
            ("format", formalized.format.clone()),
            ("label", label.to_owned()),
        ]),
    )
}

pub(super) fn check_size(
    formalized: &RepositoryFileFormalization,
    summary: &str,
) -> CriterionOutcome {
    let passed = summary.contains(&formalized.line_count.to_string())
        && summary.contains(&formalized.byte_count.to_string());
    outcome(
        "size_reported",
        true,
        passed,
        evidence(&[
            ("lines", formalized.line_count.to_string()),
            ("bytes", formalized.byte_count.to_string()),
        ]),
    )
}

pub(super) fn check_content_retained(
    formalized: &RepositoryFileFormalization,
    summary: &str,
    config: &SummarizationConfig,
) -> CriterionOutcome {
    let applicable = !formalized.statements.is_empty();
    let retained = super::super::summarize(&formalized.statements, config);
    let rendered = super::super::deformalize(&retained);
    let passed = !rendered.trim().is_empty() && summary.contains(rendered.trim());
    outcome(
        "content_retained",
        applicable,
        passed,
        evidence(&[
            ("statements", formalized.statements.len().to_string()),
            ("retained", retained.len().to_string()),
        ]),
    )
}

pub(super) fn check_content_grounded(
    path: &str,
    content: &str,
    summary: &str,
    formalized: &RepositoryFileFormalization,
) -> CriterionOutcome {
    // Labels the summarizer itself introduces — the format name, the detected
    // meta-language, and embedded block languages — are metadata about the file
    // rather than claims quoted from it, so they are grounded by construction.
    let mut vocabulary: BTreeSet<&str> = BTreeSet::new();
    vocabulary.insert(display_file_format(&formalized.format));
    vocabulary.insert(formalized.format.as_str());
    if let Some(meta) = formalized.meta_language.as_ref() {
        vocabulary.insert(meta.label.as_str());
    }
    for block in &formalized.embedded_grammars {
        vocabulary.insert(block.language.as_str());
    }

    // Inline-code delimiters are markup, not content: a summary that renders
    // `Topic`/`Short` as Topic/Short quoted the file faithfully. Grounding is
    // therefore checked against the file with its code fences and code spans
    // unwrapped, so the criterion still catches invented or dropped text —
    // `crates.io-<version>-orange` summarized as `crates.io--orange` remains a
    // failure — without penalizing correct markup removal.
    let unwrapped: String = content.chars().filter(|ch| *ch != '`').collect();

    let ungrounded: Vec<String> = identifier_tokens(summary)
        .into_iter()
        .filter(|token| {
            !vocabulary.contains(token.as_str())
                && !unwrapped.contains(token.as_str())
                && !path.contains(token.as_str())
        })
        .collect();
    let detail = if ungrounded.is_empty() {
        "all identifier tokens grounded".to_owned()
    } else {
        evidence(&[("ungrounded", ungrounded.join(", "))])
    };
    outcome("content_grounded", true, ungrounded.is_empty(), detail)
}

pub(super) fn check_compression(content: &str, summary: &str) -> CriterionOutcome {
    // Tiny files legitimately summarize into something as long as themselves —
    // "x.txt is a text file with 1 lines and 3 bytes." is longer than "hi.".
    // The criterion applies once a file is big enough for compression to mean
    // something.
    let applicable = content.len() >= COMPRESSION_FLOOR_BYTES;
    let passed = summary.len() < content.len();
    outcome(
        "compression",
        applicable,
        passed,
        evidence(&[
            ("summary_bytes", summary.len().to_string()),
            ("file_bytes", content.len().to_string()),
        ]),
    )
}

/// Files below this size are exempt from the compression criterion.
pub const COMPRESSION_FLOOR_BYTES: usize = 400;

pub(super) fn check_embedded_grammars(
    formalized: &RepositoryFileFormalization,
    content: &str,
    summary: &str,
) -> CriterionOutcome {
    let expected = fenced_block_languages(content);
    let applicable = formalized.format == "markdown" && !expected.is_empty();
    let recorded: Vec<&str> = formalized
        .embedded_grammars
        .iter()
        .map(|block| block.language.as_str())
        .collect();
    let counted = recorded.len() == expected.len();
    let named: BTreeSet<&str> = recorded.iter().copied().collect();
    let listed = named.iter().all(|language| summary.contains(*language));
    let passed = counted && listed;
    outcome(
        "embedded_grammar_recursion",
        applicable,
        passed,
        evidence(&[
            ("fences", expected.len().to_string()),
            ("recorded", recorded.len().to_string()),
            ("languages", recorded.join(",")),
        ]),
    )
}

pub(super) fn check_meta_language(
    formalized: &RepositoryFileFormalization,
    summary: &str,
) -> CriterionOutcome {
    let valid = formalized
        .meta_language
        .as_ref()
        .filter(|meta| meta.is_valid());
    let applicable = valid.is_some();
    let passed = valid.is_some_and(|meta| {
        summary.contains(&meta.label) && summary.contains(&meta.syntax_link_count.to_string())
    });
    outcome(
        "meta_language_evidence",
        applicable,
        passed,
        valid.map_or_else(
            || "no valid meta-language parse".to_owned(),
            |meta| {
                evidence(&[
                    ("label", meta.label.clone()),
                    ("links", meta.syntax_link_count.to_string()),
                ])
            },
        ),
    )
}

pub(super) fn check_determinism(
    path: &str,
    content: &str,
    config: &SummarizationConfig,
    summary: &str,
) -> CriterionOutcome {
    let repeated = formalize_repository_file(path, content).summary(config);
    let passed = repeated == summary;
    outcome(
        "determinism",
        true,
        passed,
        evidence(&[("summary_bytes", summary.len().to_string())]),
    )
}

pub(super) fn check_mode_ladder(
    formalized: &RepositoryFileFormalization,
    config: &SummarizationConfig,
) -> CriterionOutcome {
    let short = formalized.summary(&config.clone().with_mode(SummarizationMode::Short));
    let standard = formalized.summary(&config.clone().with_mode(SummarizationMode::Standard));
    let full = formalized.summary(&config.clone().with_mode(SummarizationMode::Full));
    let passed = short.len() <= standard.len() && standard.len() <= full.len();
    outcome(
        "mode_ladder",
        true,
        passed,
        evidence(&[
            ("short", short.len().to_string()),
            ("standard", standard.len().to_string()),
            ("full", full.len().to_string()),
        ]),
    )
}

/// Render a criterion's evidence as space-separated `field=value` pairs.
///
/// The evidence a report carries is machine-readable data, not prose: keeping
/// it a list of named fields rather than a sentence template means the field
/// names stay language-neutral (R379) and a reader — or a script — can split a
/// detail back into its fields without parsing English.
fn evidence(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) const fn outcome(
    name: &'static str,
    applicable: bool,
    passed: bool,
    detail: String,
) -> CriterionOutcome {
    CriterionOutcome {
        name,
        applicable,
        // A criterion that does not apply is never counted as passed, so the
        // report cannot inflate itself with vacuous truths.
        passed: applicable && passed,
        detail,
    }
}

/// Does this file reach the recursive case the `embedded_grammar_recursion`
/// criterion scores?
///
/// This mirrors that criterion's applicability test, so the stratified draw
/// promotes a file the metric will actually be able to score rather than one
/// that merely looks like Markdown.
pub(super) fn carries_embedded_grammar(path: &str, content: &str) -> bool {
    let markdown = std::path::Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
    markdown && !fenced_block_languages(content).is_empty()
}

/// Independent `CommonMark` fence scanner used as the metric's oracle.
///
/// This deliberately does *not* call the summarizer's own fence scanner: a
/// criterion that asked the implementation to grade itself would pass by
/// construction.
pub(super) fn fenced_block_languages(markdown: &str) -> Vec<String> {
    let mut languages = Vec::new();
    let mut open: Option<(char, usize)> = None;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        let marker = fence_marker(trimmed);
        match (open, marker) {
            (Some((ch, len)), Some((candidate_ch, candidate_len)))
                if candidate_ch == ch
                    && candidate_len >= len
                    && trimmed[candidate_len..].trim().is_empty() =>
            {
                open = None;
            }
            (None, Some((ch, len))) => {
                open = Some((ch, len));
                languages.push(fence_language(&trimmed[len..]));
            }
            // A non-closing marker inside an open block, or an ordinary line
            // outside one, is content rather than structure.
            _ => {}
        }
    }
    languages
}

fn fence_marker(trimmed_line: &str) -> Option<(char, usize)> {
    let ch = trimmed_line.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let len = trimmed_line.chars().take_while(|c| *c == ch).count();
    (len >= 3).then_some((ch, len))
}

fn fence_language(info_string: &str) -> String {
    info_string
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Identifier-shaped tokens (`snake_case`, `CamelCase`, `dotted.paths`) a
/// summary may only contain if the file or its path contains them too.
fn identifier_tokens(summary: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for raw in summary.split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')')) {
        let token =
            raw.trim_matches(|c: char| matches!(c, '.' | ':' | '`' | '"' | '\'' | '!' | '?'));
        if token.len() < 4 {
            continue;
        }
        let looks_like_identifier = token.contains('_')
            || token.contains('/')
            || (token.contains('.') && !token.ends_with('.'))
            || is_camel_case(token);
        if looks_like_identifier && token.chars().all(is_identifier_char) {
            tokens.push(token.to_owned());
        }
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

/// `CamelCase` in the strict sense: an interior capital that follows a
/// lower-case letter. A merely capitalized English word ("Markdown", "It") is
/// not an identifier and must not be demanded of the file's text.
fn is_camel_case(token: &str) -> bool {
    token
        .chars()
        .zip(token.chars().skip(1))
        .any(|(previous, next)| previous.is_lowercase() && next.is_uppercase())
}

const fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-' | ':')
}
