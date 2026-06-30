//! Document originality / plagiarism checks over supplied text or attachments.
//!
//! The recogniser is deliberately role-driven: code names the language-neutral
//! semantic gates, while every natural-language surface lives in the seed data.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, response_for};

use super::finalize_simple;
use super::web_requests::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};
use super::web_search_intent::WebSearchQueryKind;

const TARGET_PLACEHOLDER: &str = concat!("{", "target", "}");

pub fn try_document_originality_check(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let lexicon = seed::lexicon();
    let attachments = extract_attached_file_names(prompt);
    let has_action = lexicon
        .mentions_role(seed::ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION, normalized)
        || lexicon.mentions_role_raw(seed::ROLE_DOCUMENT_ORIGINALITY_CHECK_ACTION, normalized);
    let has_subject = lexicon.mentions_role(seed::ROLE_DOCUMENT_ORIGINALITY_SUBJECT, normalized)
        || lexicon.mentions_role_raw(seed::ROLE_DOCUMENT_ORIGINALITY_SUBJECT, normalized);
    let has_document = !attachments.is_empty()
        || lexicon.mentions_role(seed::ROLE_DOCUMENT_ORIGINALITY_DOCUMENT, normalized)
        || lexicon.mentions_role_raw(seed::ROLE_DOCUMENT_ORIGINALITY_DOCUMENT, normalized);

    if !(has_action && has_subject && has_document) {
        return None;
    }

    let language = detect_language(prompt).slug();
    let sample_present = has_text_sample(prompt);
    let query = document_originality_query(prompt, &attachments);

    log.append("language", language.to_owned());
    log.append("document_originality_check:request", query.clone());
    for attachment in &attachments {
        log.append("document_originality_check:attachment", attachment.clone());
        log.append("read_local_file:request", attachment.clone());
    }
    if sample_present {
        log.append(
            "document_originality_check:text_sample",
            "present".to_owned(),
        );
    }
    log.append("web_search:request", query.clone());
    log.append(
        "web_search:query_kind",
        WebSearchQueryKind::DocumentOriginalityCheck.as_str(),
    );
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));

    let body = document_originality_body(language, &attachments, sample_present);
    Some(finalize_simple(
        prompt,
        log,
        "document_originality_check",
        "response:document_originality_check",
        &body,
        0.84,
    ))
}

fn extract_attached_file_names(prompt: &str) -> Vec<String> {
    let mut in_section = false;
    let mut names = Vec::new();
    for line in prompt.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("Attached files:") {
            in_section = true;
            continue;
        }
        if !in_section {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("OCR text:")
            || trimmed.starts_with("Text excerpt:")
            || trimmed.starts_with("Text sample:")
            || trimmed.starts_with("Text omitted:")
        {
            continue;
        }
        let Some((_, rest)) = trimmed.split_once(". ") else {
            continue;
        };
        let Some((name, _)) = rest.split_once(" (") else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() {
            names.push(name.to_owned());
        }
    }
    names
}

fn has_text_sample(prompt: &str) -> bool {
    prompt.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("OCR text:")
            || trimmed.starts_with("Text excerpt:")
            || trimmed.starts_with("Text sample:")
    })
}

fn text_sample(prompt: &str) -> Option<String> {
    for line in prompt.lines() {
        let trimmed = line.trim();
        for prefix in ["Text excerpt:", "Text sample:", "OCR text:"] {
            if let Some(value) = trimmed.strip_prefix(prefix) {
                let sample = value
                    .split_whitespace()
                    .take(14)
                    .collect::<Vec<_>>()
                    .join(" ");
                if !sample.is_empty() {
                    return Some(sample);
                }
            }
        }
    }
    None
}

fn document_originality_query(prompt: &str, attachments: &[String]) -> String {
    if let Some(sample) = text_sample(prompt) {
        return format!("\"{sample}\" plagiarism originality");
    }
    if let Some(name) = attachments.first() {
        return format!("{name} plagiarism originality uniqueness");
    }
    "document plagiarism originality uniqueness".to_owned()
}

fn document_originality_body(
    language: &str,
    attachments: &[String],
    sample_present: bool,
) -> String {
    let target = if attachments.is_empty() {
        "provided text".to_owned()
    } else {
        attachments.join(", ")
    };
    let template_intent = if sample_present {
        "document_originality_check_sample_present"
    } else {
        "document_originality_check_sample_missing"
    };
    let template = response_for(template_intent, language)
        .or_else(|| response_for(template_intent, "en"))
        .unwrap_or_else(|| {
            "Recognized an originality and plagiarism check for `{target}`.".to_owned()
        });
    template.replace(TARGET_PLACEHOLDER, &target)
}
