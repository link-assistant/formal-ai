//! Handlers for user-intent clarification, capability queries, follow-up
//! elaboration, ill-formed input, and shell-refusal policy. Extracted from
//! `solver_handlers/mod.rs` to keep individual files under 1000 lines.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::proof_engine::{
    ProofOutcome, ProofRenderConfig, attempt_proof_with_config, render_outcome_with_config,
};
use crate::seed::{self, Slot, WordForm};
use crate::solver_handlers::finalize_simple;

/// The literal lead-in (text before the `…` slot) of every prefix-slot form of
/// a role, in lexicon declaration order. Lets the proof and who-is recognisers
/// reason about a role's opening surfaces without baking the words into code.
fn prefix_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Prefix)
        .map(WordForm::before_slot)
        .collect()
}

/// The surface text of every bare-slot form of a role, in lexicon declaration
/// order. A meaning's roles apply to all its forms, so we keep only the bare
/// detection tokens and drop any prefix/suffix surfaces the meaning also owns.
fn bare_literals(role: &str) -> Vec<&'static str> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .filter(|form| form.slot() == Slot::Bare)
        .map(|form| form.text.as_str())
        .collect()
}

/// Issue #185: catch "prove …" / "show that …" / "доказать …" / "साबित कर
/// …" / "证明 …" prompts and route them through the universal proof
/// engine (`crate::proof_engine`).
///
/// Every branch of the engine returns a real outcome — `Proven` for a
/// theorem we can discharge by direct calculation or by quoting the
/// classical proof, `Disproven` with a worked counterexample,
/// `PartialPlan` that walks the user through the proof plan and asks
/// for the missing axiom set or definitions, or `Inconclusive` with a
/// concrete reason. The handler never falls back to the generic
/// "unknown intent" opener.
pub fn try_proof_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_proof_request_with_config(prompt, normalized, log, ProofRenderConfig::default())
}

/// Configuration-aware variant of [`try_proof_request`].
///
/// The two sliders in [`ProofRenderConfig`] control how the proof is
/// surfaced to the user:
///
/// * High `guess_probability` → the engine explains how it interpreted the
///   prompt (an "Interpretation" header), commits to a formal translation,
///   and runs the proof through to a conclusion.
/// * High `follow_up_probability` → the engine appends a "Clarifying
///   questions" footer listing every input it still needs before the final
///   research execution.
///
/// The two sliders are independent so all four combinations work.
pub fn try_proof_request_with_config(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    config: ProofRenderConfig,
) -> Option<SymbolicAnswer> {
    // A proof verb may be followed by whitespace or punctuation (",", ":",
    // "!", "."). Avoid false positives on longer words that just happen to
    // start with the verb (e.g. "prover" or "proven") by checking the
    // following character is non-alphabetic. End-of-string is treated as a
    // boundary (so `normalized == verb` still matches).
    let starts_with_verb = |verb: &str| -> bool {
        normalized
            .strip_prefix(verb)
            .is_some_and(|tail| !tail.chars().next().unwrap_or(' ').is_alphabetic())
    };
    // A proof request is recognised structurally from the meaning lexicon, not
    // from words baked into this file: a clause-initial bare directive verb
    // (`proof_directive`, with the verb-boundary check above), a request-frame
    // lead in any language that needs no `that` clause (`proof_request_lead`),
    // or a mid-prompt proof assertion marker in any language (`proof_marker`).
    let is_proof_request = bare_literals(seed::ROLE_PROOF_DIRECTIVE)
        .iter()
        .any(|&verb| starts_with_verb(verb))
        || prefix_literals(seed::ROLE_PROOF_REQUEST_LEAD)
            .iter()
            .any(|&lead| normalized.starts_with(lead))
        || seed::lexicon().mentions_role_raw(seed::ROLE_PROOF_MARKER, normalized);
    if !is_proof_request {
        return None;
    }
    if is_known_unsolved_bounded_proof_request(normalized) {
        return None;
    }
    let language = detect_language(prompt).slug();
    let mentions_godel =
        seed::lexicon().mentions_role_raw(seed::ROLE_PROOF_CONCEPT_GODEL, normalized);
    let mentions_determinism =
        seed::lexicon().mentions_role_raw(seed::ROLE_PROOF_CONCEPT_DETERMINISM, normalized);
    log.append("policy:proof_request", prompt.to_owned());
    log.append(
        "policy:proof_guess_probability",
        format!("{:.2}", config.guess_probability),
    );
    log.append(
        "policy:proof_follow_up_probability",
        format!("{:.2}", config.follow_up_probability),
    );
    if mentions_godel {
        log.append("concept", "godel_incompleteness".to_owned());
    }
    if mentions_determinism {
        log.append("concept", "determinism".to_owned());
    }
    log.append("pipeline:planned", "relative-meta-logic".to_owned());
    let claim = extract_claim_from_prompt(normalized);
    let outcome = attempt_proof_with_config(
        prompt,
        &claim,
        language,
        mentions_godel,
        mentions_determinism,
        config,
    );
    log.append("proof_outcome", outcome.status_slug().to_owned());
    if let Some(method) = outcome.method() {
        log.append("proof_method", method.slug().to_owned());
    }
    if config.show_interpretation() {
        log.append("proof_render:interpretation", "shown".to_owned());
    }
    if config.ask_follow_ups() {
        log.append("proof_render:follow_ups", "shown".to_owned());
    }
    let mut body = render_outcome_with_config(&outcome, language, config);
    if matches!(outcome, ProofOutcome::PartialPlan { .. }) {
        body.push_str(&pipeline_footer(language));
    }
    let confidence = match &outcome {
        ProofOutcome::Proven { .. } | ProofOutcome::Disproven { .. } => 0.85,
        ProofOutcome::PartialPlan { .. } => 0.6,
        ProofOutcome::Inconclusive { .. } => 0.4,
    };
    Some(finalize_simple(
        prompt,
        log,
        "proof_request",
        "response:proof_request",
        &body,
        confidence,
    ))
}

fn is_known_unsolved_bounded_proof_request(normalized: &str) -> bool {
    let asks_for_terse_final_proof = normalized.contains("in two sentences")
        || normalized.contains("in 2 sentences")
        || normalized.contains("briefly prove")
        || normalized.contains("short proof");
    let names_open_problem = normalized.contains("p=np")
        || normalized.contains("p = np")
        || normalized.contains("p versus np")
        || normalized.contains("p vs np");
    asks_for_terse_final_proof && names_open_problem
}

/// Strip the surrounding "prove that …" / "докажи, что …" / "证明 …"
/// scaffolding so the proof engine receives the bare claim. We err on
/// the side of returning the full normalized prompt — the engine
/// tolerates extra text in its keyword search and arithmetic split.
fn extract_claim_from_prompt(normalized: &str) -> String {
    let trimmed = normalized.trim();
    // The claim scaffolds (each ending in the `…` slot) are sourced from the
    // `proof_claim_scaffold` role in declaration order, so the first matching
    // prefix wins exactly as before — every `that`/`что`/`कि` variant is listed
    // ahead of its shorter sibling in the lexicon. Comma variants such as
    // `докажи, что` are intentionally absent: `normalize_prompt` rewrites the
    // comma to a space, so they are unreachable here.
    let prefixes = prefix_literals(seed::ROLE_PROOF_CLAIM_SCAFFOLD);
    for prefix in &prefixes {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return strip_claim_prefix_noise(rest).to_owned();
        }
    }
    for prefix in &prefixes {
        if let Some(index) = trimmed.find(prefix) {
            let before = &trimmed[..index];
            if before.chars().last().is_some_and(is_claim_intro_boundary) {
                let rest = &trimmed[index + prefix.len()..];
                return strip_claim_prefix_noise(rest).to_owned();
            }
        }
    }
    trimmed.to_owned()
}

const fn is_claim_intro_boundary(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '.' | ',' | ':' | ';' | '!' | '?' | '…' | '。' | '，' | '：' | '；' | '！' | '？'
        )
}

fn strip_claim_prefix_noise(text: &str) -> &str {
    text.trim_start_matches(|c: char| c == ',' || c == ':' || c.is_whitespace())
}

fn pipeline_footer(language: &str) -> String {
    match language {
        "ru" => String::from(
            "\n\nПоддерживаемый конвейер: impulse → formalize (Викиданные) → context → \
             план доказательства → проверка в relative-meta-logic → deformalize → finalize.",
        ),
        "hi" => String::from(
            "\n\nसमर्थित पाइपलाइन: impulse → formalize (Wikidata) → context → प्रमाण योजना \
             → relative-meta-logic में सत्यापन → deformalize → finalize।",
        ),
        "zh" => String::from(
            "\n\n所支持的流程:impulse → formalize(Wikidata)→ context → 证明计划 → \
             relative-meta-logic 校验 → deformalize → finalize。",
        ),
        _ => String::from(
            "\n\nSupported pipeline: impulse → formalize (Wikidata-backed) → context → \
             proof plan → verification in relative-meta-logic → deformalize → finalize.",
        ),
    }
}
