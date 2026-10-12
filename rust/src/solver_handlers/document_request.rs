//! Document-generation request handler (issue #425).
//!
//! Open-ended "make me a PDF / document / report with `<subject>`" prompts ask the
//! deterministic solver to research arbitrary data and render a binary file —
//! two capabilities a symbolic engine does not have. Rather than fall through to
//! the unknown opener, this handler recognizes the request as a *document task*
//! and returns the formal decomposition the universal algorithm produces: scope
//! the deliverable, enumerate the items, classify them by the stated criteria,
//! assemble the structure, then export it to the requested format. The plan is
//! the seeded `document_generation_plan` response in the prompt language, and
//! the cue, format and marker vocabularies are the `document_*` tables of
//! `data/seed/handler-rules.lino` (issue #918), so no word of any language and
//! no sentence of the answer lives here.

use crate::document_formats::{
    DOCUMENT_FORMAT_ENGINE, DocumentConversion, convert_document_format,
    cross_format_document_concepts, supported_document_formats,
};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::normal_markov::quoted_segments;
use crate::rule_interpreter::{handler_table_row, handler_table_rows, handler_table_value};
use crate::seed;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::is_agent_request;

/// Recognize a document-generation request and answer with a formal plan.
///
/// The gate is the conjunction of an authoring action ("make"/"сделай"/…) and a
/// document artifact (an explicit format such as "PDF" or a generic document
/// noun). Prompts that also name a software artifact ("app", "bot", "script", …)
/// defer to the software-project handler, which runs later in the dispatch table.
pub fn try_document_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lowercased = normalized.to_lowercase();
    // Agent-mode prompts ("[agent] … create file report.txt …") are file
    // operations owned by the agent-workspace handler, not document tasks.
    if is_agent_request(&lowercased) {
        return None;
    }
    if mentions_software_artifact(&lowercased) {
        return None;
    }
    if let Some(answer) = try_document_conversion_request(prompt, &lowercased, log) {
        return Some(answer);
    }
    if !mentions_authoring_action(&lowercased) {
        return None;
    }
    let (label, explicit) = detect_document_format(&lowercased)?;
    log.append("document_request:format", label.to_owned());

    let language = detect_language(prompt).slug();
    let body = render_document_plan(language, explicit.then_some(label));
    Some(finalize_simple(
        prompt,
        log,
        "document_generation_plan",
        "response:document_generation_plan",
        &body,
        0.6,
    ))
}

fn try_document_conversion_request(
    prompt: &str,
    lowercased: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !mentions_conversion_action(lowercased) {
        return None;
    }

    let mentions = format_mentions(lowercased);
    let (source_format, target_format) = conversion_formats(lowercased, &mentions)?;
    let source_text = extract_document_source_text(prompt)?;
    let conversion = convert_document_format(source_format, target_format, &source_text)?;

    log.append(
        "document_conversion_engine",
        DOCUMENT_FORMAT_ENGINE.to_owned(),
    );
    log.append(
        "document_conversion_source_format",
        conversion.source_format.clone(),
    );
    log.append(
        "document_conversion_target_format",
        conversion.target_format.clone(),
    );
    log.append(
        "document_conversion_concepts",
        cross_format_document_concepts().join(","),
    );
    log.append(
        "document_conversion_output_bytes",
        conversion.output.len().to_string(),
    );
    if let Some(package_bytes) = conversion.package_bytes.as_ref() {
        log.append(
            "document_conversion_package_bytes",
            package_bytes.len().to_string(),
        );
    }

    let body = render_conversion_answer(&conversion);
    Some(finalize_simple(
        prompt,
        log,
        "document_format_conversion",
        "response:document_format_conversion",
        &body,
        0.85,
    ))
}

pub(super) fn looks_like_document_conversion_request(prompt: &str, lowercased: &str) -> bool {
    mentions_conversion_action(lowercased)
        && conversion_formats(lowercased, &format_mentions(lowercased)).is_some()
        && extract_document_source_text(prompt).is_some()
}

fn mentions_conversion_action(lowercased: &str) -> bool {
    handler_table_row("document_conversion_action", lowercased).is_some()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormatMention {
    index: usize,
    format: &'static str,
}

/// Every mention of a document format the `document_format_alias` table
/// names, in the order the request writes them.
fn format_mentions(lowercased: &str) -> Vec<FormatMention> {
    let mut mentions = Vec::new();
    for (alias, format) in handler_table_rows("document_format_alias") {
        mentions.extend(
            find_alias_positions(lowercased, alias)
                .into_iter()
                .map(|index| FormatMention {
                    index,
                    format: format.as_str(),
                }),
        );
    }
    mentions.sort_by_key(|mention| mention.index);
    mentions.dedup_by(|left, right| left.index == right.index && left.format == right.format);
    mentions
}

fn conversion_formats<'a>(
    lowercased: &str,
    mentions: &'a [FormatMention],
) -> Option<(&'a str, &'a str)> {
    let explicit_source = mentions
        .iter()
        .find(|mention| {
            has_direction_marker_before(lowercased, mention.index, "document_source_marker")
        })
        .map(|mention| mention.format);
    let explicit_target = mentions
        .iter()
        .find(|mention| {
            has_direction_marker_before(lowercased, mention.index, "document_target_marker")
        })
        .map(|mention| mention.format);

    match (explicit_source, explicit_target) {
        (Some(source), Some(target)) if source != target => Some((source, target)),
        (Some(source), _) => mentions
            .iter()
            .rev()
            .find(|mention| mention.format != source)
            .map(|mention| (source, mention.format)),
        (_, Some(target)) => mentions
            .iter()
            .find(|mention| mention.format != target)
            .map(|mention| (mention.format, target)),
        _ => {
            let source = mentions.first()?.format;
            mentions
                .iter()
                .rev()
                .find(|mention| mention.format != source)
                .map(|mention| (source, mention.format))
        }
    }
}

/// Whether a direction marker of the `markers` table (`from`, `into`, `из`, `为` …)
/// ends the text before `index`.
fn has_direction_marker_before(haystack: &str, index: usize, markers: &str) -> bool {
    let before = haystack[..index].trim_end();
    handler_table_rows(markers)
        .iter()
        .any(|(marker, _)| before.ends_with(marker.trim_end()))
}

fn find_alias_positions(haystack: &str, alias: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    let mut offset = 0usize;
    while let Some(relative) = haystack[offset..].find(alias) {
        let index = offset + relative;
        let end = index + alias.len();
        if is_token_boundary(haystack, index, end) {
            positions.push(index);
        }
        offset = end;
    }
    positions
}

fn is_token_boundary(haystack: &str, start: usize, end: usize) -> bool {
    let before = haystack[..start].chars().next_back();
    let after = haystack[end..].chars().next();
    !before.is_some_and(is_ascii_word_char) && !after.is_some_and(is_ascii_word_char)
}

const fn is_ascii_word_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

fn extract_document_source_text(prompt: &str) -> Option<String> {
    first_fenced_code_block(prompt)
        .or_else(|| quoted_segments(prompt).last().cloned())
        .or_else(|| text_after_colon(prompt))
        .and_then(non_empty)
}

fn first_fenced_code_block(text: &str) -> Option<String> {
    let fence_start = text.find("```")?;
    let after_fence = &text[fence_start + 3..];
    let content_start = after_fence
        .find('\n')
        .map_or(fence_start + 3, |newline| fence_start + 3 + newline + 1);
    let content = &text[content_start..];
    let fence_end = content.find("```")?;
    Some(content[..fence_end].trim_matches('\n').to_owned())
}

fn text_after_colon(text: &str) -> Option<String> {
    text.find(':')
        .or_else(|| text.find('：'))
        .map(|index| text[index + 1..].trim().to_owned())
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

/// The seeded `document_format_conversion*` lines over the converted output.
fn render_conversion_answer(conversion: &DocumentConversion) -> String {
    let line = |intent: &str, slots: &[(&str, &str)]| {
        seed::render_response(intent, "en", slots).map_or_else(String::new, |text| text + "\n")
    };
    let formats = supported_document_formats().join(", ");
    let mut body = line(
        "document_format_conversion",
        &[
            ("engine", DOCUMENT_FORMAT_ENGINE),
            ("source", conversion.source_format.as_str()),
            ("target", conversion.target_format.as_str()),
            ("formats", formats.as_str()),
        ],
    );
    let capabilities = &conversion.target_capabilities;
    if !capabilities.native_concepts.is_empty() {
        let concepts = capabilities.native_concepts.join(", ");
        body.push_str(&line(
            "document_format_conversion_native_concepts",
            &[("concepts", concepts.as_str())],
        ));
    }
    if !capabilities.fallbacks.is_empty() {
        body.push_str(&line("document_format_conversion_fallbacks", &[]));
        for (concept, fallback) in &capabilities.fallbacks {
            body.push_str(&line(
                "document_format_conversion_fallback",
                &[
                    ("concept", concept.as_str()),
                    ("fallback", fallback.as_str()),
                ],
            ));
        }
    }
    if let Some(package_bytes) = conversion.package_bytes.as_ref() {
        let bytes = package_bytes.len().to_string();
        body.push_str(&line(
            "document_format_conversion_package",
            &[("bytes", bytes.as_str())],
        ));
    }
    let fence =
        handler_table_value("document_output_fence", &conversion.target_format).unwrap_or_default();
    body.push_str(&["\n```", fence, "\n", conversion.output.trim_end(), "\n```"].concat());
    body
}

/// True when the prompt names a software artifact (the `document_software_artifact`
/// table), in which case the request is a build task for the software-project
/// handler rather than a document task.
fn mentions_software_artifact(lowercased: &str) -> bool {
    handler_table_row("document_software_artifact", lowercased).is_some()
}

/// True when the prompt carries an authoring verb of the
/// `document_authoring_action` table; its Russian and Hindi rows are stems, so
/// they survive inflection.
fn mentions_authoring_action(lowercased: &str) -> bool {
    handler_table_row("document_authoring_action", lowercased).is_some()
}

/// The requested document container: an explicit format label (`PDF`, `DOCX`,
/// …) with `true`, or the generic document label with `false`; `None` when no
/// document artifact is named, so a plain "create X" prompt is left for other
/// handlers. The tables are read in priority order: the explicit formats, an
/// ebook marker beside a book noun, then the generic document nouns (a bare
/// "file" is not one: it is too weak on its own).
fn detect_document_format(lowercased: &str) -> Option<(&'static str, bool)> {
    if let Some(label) = handler_table_row("document_format", lowercased) {
        return Some((label, true));
    }
    if let Some(label) = handler_table_row("document_ebook_marker", lowercased)
        && handler_table_row("document_book_noun", lowercased).is_some()
    {
        return Some((label, true));
    }
    handler_table_row("document_noun", lowercased).map(|label| (label, false))
}

/// Whether the request names a document format the plan renders.
///
/// The `document_format` claim evidence of issue #1175 R3.
#[must_use]
pub fn names_document_format(normalized: &str) -> bool {
    let lowercased = normalized.to_lowercase();
    detect_document_format(&lowercased).is_some() || !format_mentions(&lowercased).is_empty()
}

/// Render the seeded document-generation plan in the prompt language, with the
/// seeded format suffix when an explicit format is named.
fn render_document_plan(language: &str, label: Option<&str>) -> String {
    let suffix = label
        .and_then(|label| {
            seed::localized_response("document_generation_plan_format", language)
                .map(|text| text.replace(concat!("{", "label", "}"), label))
        })
        .unwrap_or_default();
    seed::localized_response("document_generation_plan", language)
        .map(|text| text.replace("{format_suffix}", &suffix))
        .unwrap_or_default()
}
