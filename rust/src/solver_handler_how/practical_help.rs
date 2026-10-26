//! Data-owned need/goal matching and provenance-gated practical guidance.
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::source_fetch::{CachedSourceClient, SourceTransport};
fn policy(key: &str) -> String {
    crate::rule_interpreter::handler_policy("practical-help", key).unwrap_or_default()
}
fn forms(key: &str) -> Vec<String> {
    policy(key)
        .split('|')
        .map(normalize_prompt)
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
pub(crate) fn goal(prompt: &str, _normalized: &str) -> Option<String> {
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
    policy("goals")
        .split('|')
        .find(|goal| {
            forms(&format!("{goal}-forms"))
                .iter()
                .any(|word| rest.starts_with(word.as_str()) && whole(rest, word))
        })
        .map(str::to_owned)
}
fn trusted(goal: &str) -> bool {
    url::Url::parse(&policy(&format!("{goal}-source"))).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str().is_some_and(|host| {
                policy("trusted-hosts")
                    .split('|')
                    .any(|entry| entry.trim().eq_ignore_ascii_case(host))
            })
    })
}
fn render(
    prompt: &str,
    goal: &str,
    evidence: &str,
    mode: &str,
    log: &mut EventLog,
) -> SymbolicAnswer {
    let source = policy(&format!("{goal}-source"));
    let proof = policy(&format!("{goal}-proof"));
    let supported = trusted(goal)
        && !proof.is_empty()
        && evidence.len() <= 2_097_152
        && normalize_prompt(&crate::how_to_guide::extract::strip_html(evidence))
            .contains(&normalize_prompt(&proof));
    let key = policy(&format!(
        "{goal}-{}",
        if supported { "response" } else { "unavailable" }
    ));
    let text = crate::seed::render_localized_once(
        &key,
        crate::language::detect(prompt).slug(),
        &[("source", &source)],
    );
    log.append("practical_help:goal", goal.to_owned());
    log.append(
        "practical_help:source_mode",
        if supported { mode } else { "unavailable" }.to_owned(),
    );
    if supported {
        log.append("practical_help:source", source);
    }
    crate::solver_handlers::finalize_simple(
        prompt,
        log,
        "procedural_how_to",
        "response:practical_help",
        &text,
        if supported { 0.8 } else { 0.0 },
    )
}
pub(super) fn cached(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let goal = goal(prompt, normalized)?;
    Some(render(
        prompt,
        &goal,
        &policy(&format!("{goal}-cached-proof")),
        "curated-excerpt",
        log,
    ))
}
pub(crate) fn with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let goal = goal(prompt, normalized)?;
    let cached = policy(&format!("{goal}-cached-proof"));
    if !cached.is_empty() {
        return Some(render(prompt, &goal, &cached, "curated-excerpt", log));
    }
    Some(from_source(prompt, &goal, log, client))
}
fn from_source<T: SourceTransport>(
    prompt: &str,
    goal: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> SymbolicAnswer {
    let evidence = if trusted(goal) {
        client
            .fetch(&policy(&format!("{goal}-source")))
            .ok()
            .and_then(|capture| {
                (capture.bytes().len() <= 2_097_152)
                    .then(|| String::from_utf8(capture.bytes().to_vec()).ok())
                    .flatten()
            })
    } else {
        None
    };
    render(
        prompt,
        goal,
        evidence.as_deref().unwrap_or(""),
        "fetched-source",
        log,
    )
}
pub(super) fn followup(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !forms("followups")
        .iter()
        .any(|form| form == &normalize_prompt(prompt))
        && !super::is_procedural_elaboration_request(&normalize_prompt(prompt))
    {
        return None;
    }
    let prior = crate::solver_helpers::last_user_turn(log)?;
    let goal = goal(prior, &normalize_prompt(prior))?;
    Some(render(
        prompt,
        &goal,
        &policy(&format!("{goal}-cached-proof")),
        "curated-excerpt",
        log,
    ))
}

#[cfg(test)]
#[path = "../../tests/fixtures/practical-help-tests.rs"]
mod tests;
