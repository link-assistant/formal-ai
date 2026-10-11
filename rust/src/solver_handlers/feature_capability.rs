//! Feature-specific capability questions and runtime availability.
//!
//! Issue #918: the features, their runtime switches, localized labels and
//! examples are `data/seed/feature-capabilities.lino`, and every answer
//! sentence is a seeded `feature_capability_*` response
//! (`data/seed/multilingual-responses-capabilities.lino`). The browser twin
//! (`tryFeatureCapabilityStatus` in `js/worker/formal_ai_worker_capabilities_and_runtime_rules.js`) reads
//! the same records.

use std::sync::OnceLock;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::solver_handlers::finalize_simple;
use crate::web_search_core::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};

const FEATURES_PATH: &str = "data/seed/feature-capabilities.lino";
/// The prefix of the alias meanings that name the seeded features.
const ALIAS_PREFIX: &str = "feature_capability_";

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct CapabilityRuntime {
    pub offline: bool,
    pub agent_mode: bool,
    pub diagnostic_mode: bool,
    pub definition_fusion_by_default: bool,
}

impl CapabilityRuntime {
    #[allow(clippy::fn_params_excessive_bools)]
    pub const fn new(
        offline: bool,
        agent_mode: bool,
        diagnostic_mode: bool,
        definition_fusion_by_default: bool,
    ) -> Self {
        Self {
            offline,
            agent_mode,
            diagnostic_mode,
            definition_fusion_by_default,
        }
    }
}

pub fn try_feature_capability(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: CapabilityRuntime,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt);
    if !is_feature_capability_question(normalized, language.slug()) {
        return None;
    }

    let feature = detect_feature_capability(normalized, language.slug())?;
    if is_feature_action_request(normalized, feature) {
        return None;
    }
    let availability = feature.availability(runtime);
    log.append("feature:question", feature.slug.clone());
    if availability.available {
        log.append("feature:available", feature.slug.clone());
    } else {
        log.append(
            "feature:unavailable",
            format!("{}:{}", feature.slug, availability.reason),
        );
    }

    let body = if feature.slug == "web_search" {
        if availability.available {
            for provider in WEB_SEARCH_PROVIDERS {
                log.append("web_search:provider_planned", (*provider).to_owned());
            }
            log.append(
                "web_search:fusion_planned",
                format!("rrf:k={WEB_SEARCH_RRF_K}"),
            );
        }
        web_search_capability_body(
            language.slug(),
            availability.available,
            &WEB_SEARCH_PROVIDERS.join(", "),
        )
    } else {
        feature_capability_body(feature, language.slug(), availability)
    };

    Some(finalize_simple(
        prompt,
        log,
        "capabilities",
        "response:capabilities",
        &body,
        if availability.available { 0.95 } else { 0.6 },
    ))
}

/// One seeded feature: its slug, the runtime switch that gates it, and its
/// localized label and example.
#[derive(Debug, Clone)]
struct FeatureCapability {
    slug: String,
    state: String,
    node: LinoNode,
}

#[derive(Debug, Clone, Copy)]
struct FeatureAvailability {
    available: bool,
    reason: &'static str,
}

impl FeatureCapability {
    /// The `kind` (`label` or `example`) text in `language`, or English.
    fn localized(&self, kind: &str, language: &str) -> String {
        let Some(group) = self.node.children.iter().find(|child| child.name == kind) else {
            return String::new();
        };
        let text = group.find_child_value(language);
        if text.is_empty() {
            group.find_child_value("en").to_owned()
        } else {
            text.to_owned()
        }
    }

    fn availability(&self, runtime: CapabilityRuntime) -> FeatureAvailability {
        let (available, reason) = match self.state.as_str() {
            "web_search" => (
                !runtime.offline && !WEB_SEARCH_PROVIDERS.is_empty(),
                "offline_or_no_providers",
            ),
            "agent_mode" => (runtime.agent_mode, "agent_mode_off"),
            "diagnostic_mode" => (runtime.diagnostic_mode, "diagnostic_mode_off"),
            "definition_fusion" => (
                runtime.definition_fusion_by_default,
                "definition_fusion_explicit",
            ),
            _ => (true, "none"),
        };
        FeatureAvailability {
            available,
            reason: if available { "none" } else { reason },
        }
    }
}

/// The seeded features, in seed order.
fn feature_capabilities() -> &'static [FeatureCapability] {
    static FEATURES: OnceLock<Vec<FeatureCapability>> = OnceLock::new();
    FEATURES.get_or_init(|| {
        let Some((_, text)) = seed::seed_files()
            .into_iter()
            .find(|(registered, _)| *registered == FEATURES_PATH)
        else {
            return Vec::new();
        };
        parse_lino(text)
            .children
            .iter()
            .flat_map(|root| root.children.iter())
            .filter(|node| node.name == "feature")
            .map(|node| FeatureCapability {
                slug: node
                    .id
                    .strip_prefix(ALIAS_PREFIX)
                    .unwrap_or(&node.id)
                    .to_owned(),
                state: node.find_child_value("state").to_owned(),
                node: node.clone(),
            })
            .collect()
    })
}

// Walk the `feature_capability_alias` meanings in their seed declaration order —
// which mirrors the historical hand-ordered priority of `FEATURE_CAPABILITIES` —
// and return the first whose multilingual forms occur as a raw substring of the
// normalized prompt, checked in the prompt's own language plus English. The
// matched meaning's slug, minus its `feature_capability_` prefix, keys the
// runtime table below, so no surface alias is named in the code.
fn detect_feature_capability(
    normalized: &str,
    language: &str,
) -> Option<&'static FeatureCapability> {
    let lexicon = seed::lexicon();
    let languages: Vec<&str> = if language == "en" {
        vec!["en"]
    } else {
        vec![language, "en"]
    };
    let meaning = lexicon.first_role_match_in_languages_raw(
        seed::ROLE_FEATURE_CAPABILITY_ALIAS,
        normalized,
        &languages,
    )?;
    let slug = meaning.slug.strip_prefix(ALIAS_PREFIX)?;
    feature_capabilities()
        .iter()
        .find(|feature| feature.slug == slug)
}

// A prompt is a capability question when one of the `feature_capability_question`
// interrogative cues occurs as a raw substring, checked in the prompt's own
// detected language only. A language the seed gives no cue reads as English,
// which also accepts a grammatical "is/are ... enabled/available" frame
// computed in code (R1188-U1: no per-language branch).
fn is_feature_capability_question(normalized: &str, language: &str) -> bool {
    let (lexicon, role) = (seed::lexicon(), seed::ROLE_FEATURE_CAPABILITY_QUESTION);
    let mentions = |lang: &str| lexicon.mentions_role_in_languages_raw(role, normalized, &[lang]);
    if language != "en"
        && !lexicon
            .words_for_role_in_languages(role, &[language])
            .is_empty()
    {
        return mentions(language);
    }
    mentions("en") || is_english_availability_question(normalized)
}

fn is_english_availability_question(normalized: &str) -> bool {
    let trimmed = normalized.trim_end_matches('?').trim();
    if !(trimmed.contains(" enabled") || trimmed.contains(" available")) {
        return false;
    }
    trimmed.starts_with("is ")
        || trimmed.starts_with("are ")
        || trimmed.contains(" is ")
        || trimmed.contains(" are ")
}

// Some capability questions are really action requests ("can you calculate 2 +
// 2?"): the user wants the task done, not a yes/no about the capability. Those
// English action frames live in the seed as `feature_action_arithmetic` /
// `feature_action_planning` forms; each is reconstructed with a trailing space
// so it matches as a word boundary. Only the English frames drive the gate; the
// other-language forms are kept in the seed purely for self-description.
fn is_feature_action_request(normalized: &str, feature: &FeatureCapability) -> bool {
    let lexicon = seed::lexicon();
    match feature.slug.as_str() {
        "arithmetic" => lexicon
            .words_for_role_in_languages(seed::ROLE_FEATURE_ACTION_ARITHMETIC, &["en"])
            .iter()
            .any(|frame| normalized.starts_with(format!("{frame} ").as_str())),
        "planning" => lexicon
            .words_for_role_in_languages(seed::ROLE_FEATURE_ACTION_PLANNING, &["en"])
            .iter()
            .any(|frame| normalized.contains(format!("{frame} ").as_str())),
        _ => false,
    }
}

fn feature_capability_body(
    feature: &FeatureCapability,
    language: &str,
    availability: FeatureAvailability,
) -> String {
    let label = feature.localized("label", language);
    let example = feature.localized("example", language);
    if availability.available {
        return capability_response(
            "feature_capability_available",
            language,
            &[("label", &label), ("example", &example)],
        );
    }
    let reason = seed::localized_response(
        &format!("feature_capability_reason_{}", availability.reason),
        language,
    )
    .unwrap_or_default();
    capability_response(
        "feature_capability_unavailable",
        language,
        &[
            ("reason", &reason),
            ("label", &label),
            ("example", &example),
        ],
    )
}

fn web_search_capability_body(language: &str, available: bool, providers: &str) -> String {
    if available {
        capability_response(
            "feature_capability_web_search_available",
            language,
            &[
                ("k", &WEB_SEARCH_RRF_K.to_string()),
                ("providers", providers),
            ],
        )
    } else {
        capability_response("feature_capability_web_search_unavailable", language, &[])
    }
}

fn capability_response(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut text = seed::localized_response(intent, language).unwrap_or_default();
    for (name, value) in values {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}
