//! Playwright script requests.
//!
//! Issue #918: the handler holds no prose and no example. The tool name, its
//! misspelling and the script cues are the `playwright_tool_name` and
//! `playwright_script_cue` roles of the seed lexicon; the docs URL is the
//! `policy playwright_script` block of `data/seed/handler-rules.lino`; the
//! clarification, the two leads, the starter layout and the starter
//! TypeScript are seeded `playwright_*` responses. The browser twin
//! (`tryPlaywrightScript` in `js/worker/formal_ai_worker_software_project_plans.js`) renders the
//! same records.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::solver_handlers::finalize_simple;

fn docs_url() -> String {
    crate::rule_interpreter::handler_policy("playwright_script", "docs_url").unwrap_or_default()
}

pub fn try_playwright_script(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    guess_probability: f32,
) -> Option<SymbolicAnswer> {
    if !is_playwright_script_request(normalized) {
        return None;
    }

    let docs = docs_url();
    log.append("script_framework", "playwright".to_owned());
    log.append("source", docs.clone());
    let corrected_spelling = mentions_playwright_misspelling(normalized);
    if corrected_spelling {
        log.append("spelling_correction", "Playright -> Playwright".to_owned());
    }
    log.append(
        "guess_probability",
        format!("{:.2}", guess_probability.clamp(0.0, 1.0)),
    );

    let language = detect_language(prompt).slug();
    if guess_probability < 0.5 {
        let body = seed::localized_response("playwright_script_clarification", language)
            .unwrap_or_default();
        return Some(finalize_simple(
            prompt,
            log,
            "playwright_script_clarification",
            "response:playwright_script_clarification",
            &body,
            0.64,
        ));
    }

    let body = render_starter(language, corrected_spelling, &docs);
    Some(finalize_simple(
        prompt,
        log,
        "playwright_script",
        "response:playwright_script",
        &body,
        0.82,
    ))
}

/// True when the prompt both names the Playwright tool and carries a
/// script-authoring cue, each recognized as a raw substring across every
/// supported language via the `playwright_tool_name` and
/// `playwright_script_cue` roles in the seed lexicon.
fn is_playwright_script_request(normalized: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role_raw(seed::ROLE_PLAYWRIGHT_TOOL_NAME, normalized)
        && lexicon.mentions_role_raw(seed::ROLE_PLAYWRIGHT_SCRIPT_CUE, normalized)
}

/// True when the prompt contains a misspelled form of the Playwright name —
/// any `playwright_tool_name` word form whose `action` names the canonical
/// spelling. The misspelling and its correction live in the seed data, so the
/// handler reports the fix without naming either form in the code.
fn mentions_playwright_misspelling(normalized: &str) -> bool {
    seed::lexicon()
        .role_word_forms(seed::ROLE_PLAYWRIGHT_TOOL_NAME)
        .iter()
        .filter(|form| !form.action.is_empty())
        .any(|form| normalized.contains(form.text.as_str()))
}

/// The starter answer: the seeded lead (with the spelling correction when the
/// prompt misspelled the tool), the docs source and the seeded TypeScript
/// starter, laid out by the seeded `playwright_script_starter` record.
fn render_starter(language: &str, corrected_spelling: bool, docs: &str) -> String {
    let lead_intent = if corrected_spelling {
        "playwright_script_lead_corrected"
    } else {
        "playwright_script_lead"
    };
    let opening = seed::localized_response(lead_intent, language).unwrap_or_default();
    let starter =
        seed::localized_response("playwright_starter_typescript", "en").unwrap_or_default();
    seed::localized_response("playwright_script_starter", language)
        .unwrap_or_default()
        .replace("{lead}", &opening)
        .replace("{source}", docs)
        .replace("{code}", &starter)
}
