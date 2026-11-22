//! Data-owned need/goal matching and provenance-gated practical guidance.
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::source_fetch::{
    CachedSourceClient, CurlSourceTransport, SourceCapture, SourceTransport,
};
use crate::web_formalize::{PageBlock, formalize_page_with_context};

fn policy(key: &str) -> String {
    crate::rule_interpreter::handler_policy("practical-help", key).unwrap_or_default()
}
fn values(key: &str) -> Vec<String> {
    let scalar = policy(key);
    if !scalar.is_empty() {
        return vec![scalar];
    }
    crate::rule_interpreter::handler_table_rows("practical-help")
        .iter()
        .filter(|(name, _)| name == key)
        .map(|(_, value)| value.clone())
        .collect()
}
fn forms(key: &str) -> Vec<String> {
    values(key)
        .iter()
        .map(|value| normalize_prompt(value))
        .filter(|value| !value.is_empty())
        .collect()
}
fn whole(text: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let cjk = word
        .chars()
        .any(|character| ('\u{3400}'..='\u{9fff}').contains(&character));
    text.match_indices(word).any(|(at, _)| {
        cjk || (!text[..at]
            .chars()
            .next_back()
            .is_some_and(|character| character.is_alphanumeric() || character == '_')
            && !text[at + word.len()..]
                .chars()
                .next()
                .is_some_and(|character| character.is_alphanumeric() || character == '_'))
    })
}
pub fn goal(prompt: &str, _normalized: &str) -> Option<String> {
    let canonical = normalize_prompt(prompt);
    let normalized = canonical.as_str();
    if prompt
        .trim_start()
        .starts_with(['"', '\'', '\u{0060}', '\u{00ab}', '\u{201c}'])
    {
        return None;
    }
    let lead = forms("leads")
        .into_iter()
        .filter(|lead| {
            normalized.starts_with(lead.as_str())
                && (lead
                    .chars()
                    .any(|character| ('\u{3400}'..='\u{9fff}').contains(&character))
                    || !normalized[lead.len()..]
                        .chars()
                        .next()
                        .is_some_and(|character| character.is_alphanumeric() || character == '_'))
        })
        .max_by_key(String::len)?;
    let rest = normalized[lead.len()..].trim();
    if forms("negations").iter().any(|word| whole(rest, word)) {
        return None;
    }
    values("goals")
        .iter()
        .find(|goal| {
            forms(&format!("{goal}-forms"))
                .iter()
                .any(|word| rest.starts_with(word.as_str()) && whole(rest, word))
        })
        .cloned()
}
const MAXIMUM_CAPTURE_BYTES: usize = 2 * 1024 * 1024;
const MAXIMUM_GUIDANCE_CHARACTERS: usize = 480;

fn source_is_trusted(source: &str) -> bool {
    url::Url::parse(source).is_ok_and(|url| {
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.host_str().is_some_and(|host| {
                values("trusted-hosts")
                    .iter()
                    .any(|entry| entry.trim().eq_ignore_ascii_case(host))
            })
    })
}

fn source_guidance(capture: &SourceCapture, goal: &str) -> Option<String> {
    section_excerpt(
        capture.bytes(),
        &policy(&format!("{goal}-heading")),
        &forms(&format!("{goal}-excerpt-terms")),
    )
}

pub(crate) fn section_excerpt(bytes: &[u8], heading: &str, terms: &[String]) -> Option<String> {
    let heading = normalize_prompt(heading);
    let cues: Vec<_> = terms.iter().map(|term| normalize_prompt(term)).collect();
    if heading.is_empty()
        || cues.is_empty()
        || cues.iter().any(String::is_empty)
        || bytes.len() > MAXIMUM_CAPTURE_BYTES
        || std::str::from_utf8(bytes).is_err()
    {
        return None;
    }
    let (_, blocks) = formalize_page_with_context(bytes, Some("text/html"), None);
    let mut blocks = blocks.into_iter().map(|(_, block)| block);
    let level = blocks.find_map(|block| match block {
        PageBlock::Heading { level, text } if normalize_prompt(&text) == heading => Some(level),
        _ => None,
    })?;
    blocks
        .take_while(
            |block| !matches!(block, PageBlock::Heading { level: next, .. } if *next <= level),
        )
        .find_map(|block| match block {
            PageBlock::Paragraph { text, .. } | PageBlock::ListItem { text } => {
                let excerpt = text.split_whitespace().collect::<Vec<_>>().join(" ");
                if excerpt.is_empty() || excerpt.chars().count() > MAXIMUM_GUIDANCE_CHARACTERS {
                    return None;
                }
                let normalized = normalize_prompt(&excerpt);
                cues.iter()
                    .all(|word| whole(&normalized, word))
                    .then_some(excerpt)
            }
            _ => None,
        })
}

pub(super) fn cached(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let goal = goal(prompt, normalized)?;
    let client = CachedSourceClient::new(
        crate::coding::synthesis_runtime::source_cache_root(),
        CurlSourceTransport,
    );
    Some(from_source(prompt, &goal, log, &client))
}

pub fn with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let goal = goal(prompt, normalized)?;
    Some(from_source(prompt, &goal, log, client))
}

fn from_source<T: SourceTransport>(
    prompt: &str,
    goal: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> SymbolicAnswer {
    let source = policy(&format!("{goal}-source"));
    let capture = if source_is_trusted(&source) {
        match client.fetch(&source) {
            Ok(capture) => Some(capture),
            Err(error) => {
                log.append("practical_help:source_failure", error.to_string());
                None
            }
        }
    } else {
        log.append("practical_help:source_failure", "untrusted_source");
        None
    };
    let language = crate::language::detect(prompt).slug();
    log.append("practical_help:goal", goal.to_owned());
    let (answer, mode) = match capture {
        Some(capture) => {
            if let Some(excerpt) = source_guidance(&capture, goal) {
                capture.record(log);
                log.append("practical_help:source", source.clone());
                let text = crate::seed::render_localized_once(
                    "practical-help-source-guidance",
                    language,
                    &[("source", &source), ("excerpt", &excerpt)],
                );
                let mode = if capture.cached() {
                    "cached-source"
                } else {
                    "fetched-source"
                };
                (text, mode)
            } else {
                log.append("practical_help:source_failure", "no_matching_excerpt");
                (unavailable_response(goal, language), "unavailable")
            }
        }
        None => (unavailable_response(goal, language), "unavailable"),
    };
    log.append("practical_help:source_mode", mode);
    crate::solver_handlers::finalize_simple(
        prompt,
        log,
        "procedural_how_to",
        "response:practical_help",
        &answer,
        if mode == "unavailable" { 0.0 } else { 0.8 },
    )
}

fn unavailable_response(goal: &str, language: &str) -> String {
    crate::seed::render_localized_once(&policy(&format!("{goal}-unavailable")), language, &[])
}

pub(crate) fn followup_goal(normalized: &str, prior: &str) -> Option<String> {
    if !forms("followups").iter().any(|form| form == normalized)
        && !super::is_procedural_elaboration_request(normalized)
    {
        return None;
    }
    goal(prior, &normalize_prompt(prior))
}

#[cfg(test)]
#[path = "../../tests/fixtures/practical-help-tests.rs"]
mod tests;
