//! Document originality / plagiarism checks over supplied text or attachments.
//!
//! The recogniser is deliberately role-driven: code names the language-neutral
//! semantic gates, while every natural-language surface lives in the seed data.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::relative_meta_logic::{SourceTier, ASSUMED_TRUE_PRIOR};
use crate::seed::{self, response_for};
use crate::statement_verification::{
    assess_market_price_claims, extract_market_price_claims, MarketPriceAssessment,
    StatementVerificationPlan, TRUSTED_SOURCE_POLICY,
};

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
    log.append("web_search:request", query);
    log.append(
        "web_search:query_kind",
        WebSearchQueryKind::DocumentOriginalityCheck.as_str(),
    );
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));

    let market_assessments = log_statement_verification(prompt, log);

    let body =
        document_originality_body(language, &attachments, sample_present, &market_assessments);
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
    full_text_sample(prompt).and_then(|sample| {
        let sample = sample
            .split_whitespace()
            .take(14)
            .collect::<Vec<_>>()
            .join(" ");
        if sample.is_empty() {
            None
        } else {
            Some(sample)
        }
    })
}

/// Extract the full (untruncated) text excerpt supplied with the prompt, used
/// to split individual statements for grounding.
fn full_text_sample(prompt: &str) -> Option<String> {
    let samples = full_text_samples(prompt);
    if samples.is_empty() {
        None
    } else {
        Some(samples.join("\n\n"))
    }
}

fn full_text_samples(prompt: &str) -> Vec<String> {
    let mut samples = Vec::new();
    let mut current: Option<String> = None;
    for line in prompt.lines() {
        let trimmed = line.trim();
        if let Some(value) = text_sample_prefix_value(trimmed) {
            push_current_text_sample(&mut samples, &mut current);
            current = Some(value.trim().to_owned());
            continue;
        }
        let Some(sample) = current.as_mut() else {
            continue;
        };
        if trimmed.is_empty() || is_attachment_context_boundary(trimmed) {
            push_current_text_sample(&mut samples, &mut current);
            continue;
        }
        if !sample.is_empty() {
            sample.push('\n');
        }
        sample.push_str(trimmed);
    }
    push_current_text_sample(&mut samples, &mut current);
    samples
}

fn push_current_text_sample(samples: &mut Vec<String>, current: &mut Option<String>) {
    if let Some(sample) = current.take() {
        let sample = sample.trim();
        if !sample.is_empty() {
            samples.push(sample.to_owned());
        }
    }
}

fn text_sample_prefix_value(trimmed: &str) -> Option<&str> {
    ["Text excerpt:", "Text sample:", "OCR text:"]
        .into_iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix))
}

fn is_attachment_context_boundary(trimmed: &str) -> bool {
    trimmed.eq_ignore_ascii_case("Attached files:")
        || is_attachment_file_line(trimmed)
        || trimmed.starts_with("Text omitted:")
        || trimmed.starts_with("Text unavailable:")
        || trimmed.starts_with("OCR unavailable:")
}

fn is_attachment_file_line(trimmed: &str) -> bool {
    let Some((index, rest)) = trimmed.split_once(". ") else {
        return false;
    };
    !index.is_empty()
        && index.chars().all(|character| character.is_ascii_digit())
        && rest.contains(" (")
}

/// Replay the per-statement relative-meta-logic verification plan into the
/// append-only event log. Each statement is assumed true, grounded by a
/// dedicated web-search query, and weighed under the trusted-source policy —
/// original first sources first, reposts ignored.
fn log_statement_verification(prompt: &str, log: &mut EventLog) -> Vec<MarketPriceAssessment> {
    log.append(
        "relative_meta_logic:assumed_prior",
        format!("{ASSUMED_TRUE_PRIOR:.6}"),
    );
    for tier in TRUSTED_SOURCE_POLICY {
        log.append(
            "relative_meta_logic:trusted_source_tier",
            format!("{}:weight={:.6}", tier.slug(), tier.weight()),
        );
    }
    log.append(
        "relative_meta_logic:ignored_source_tier",
        SourceTier::Unoriginal.slug().to_owned(),
    );

    let Some(sample) = full_text_sample(prompt) else {
        return Vec::new();
    };
    let plan = StatementVerificationPlan::from_sample(&sample);
    log.append(
        "statement_verification:statement_count",
        plan.len().to_string(),
    );
    for statement_plan in &plan.statements {
        log.append(
            "statement_verification:statement",
            statement_plan.statement.clone(),
        );
        log.append("statement_verification:query", statement_plan.query.clone());
        log.append("web_search:request", statement_plan.query.clone());
        log.append(
            "web_search:query_kind",
            WebSearchQueryKind::DocumentOriginalityCheck.as_str(),
        );
        log.append(
            "statement_verification:assessment",
            statement_plan.assessment.trace_payload(),
        );
    }

    let claims = extract_market_price_claims(&sample);
    let market_assessments = assess_market_price_claims(&claims);
    if !claims.is_empty() {
        log.append("market_price_claim:claim_count", claims.len().to_string());
    }
    for claim in &claims {
        log.append("market_price_claim:claim", claim.statement.clone());
        log.append(
            "market_price_claim:asset",
            format!("{} ({})", claim.asset, claim.asset_label),
        );
        log.append("market_price_claim:period", claim.period.clone());
        log.append(
            "market_price_claim:claimed_price",
            format!("{} {:.2}", claim.currency, claim.claimed_price),
        );
    }
    for assessment in &market_assessments {
        log.append(
            "market_price_claim:source",
            format!("{} {}", assessment.source_id, assessment.source_url),
        );
        log.append(
            "market_price_claim:range",
            format!(
                "asset={} period={} source={} min={:.2} min_date={} max={:.2} max_date={}",
                assessment.claim.asset,
                assessment.claim.period,
                assessment.source_id,
                assessment.observed_min_price,
                assessment.observed_min_date,
                assessment.observed_max_price,
                assessment.observed_max_date,
            ),
        );
        log.append("market_price_claim:assessment", assessment.trace_payload());
    }
    market_assessments
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
    market_assessments: &[MarketPriceAssessment],
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
    let mut body = template.replace(TARGET_PLACEHOLDER, &target);
    let contradicted = market_assessments
        .iter()
        .filter(|assessment| assessment.status == "contradicted")
        .collect::<Vec<_>>();
    if !contradicted.is_empty() {
        let heading = match language {
            "ru" => "Проверка ценовых утверждений",
            "hi" => "मूल्य दावों की जांच",
            "zh" => "价格声明核查",
            _ => "Price claim check",
        };
        let summaries = contradicted
            .into_iter()
            .map(|assessment| format!("- {}", assessment.summary_sentence()))
            .collect::<Vec<_>>()
            .join("\n");
        body = format!("{body}\n\n{heading}:\n{summaries}");
    }
    body
}
