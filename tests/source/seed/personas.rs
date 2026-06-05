//! Roleplay-persona records loaded from `data/seed/personas.lino`.
//!
//! Each `persona "<Name>"` entry lists multilingual aliases (the surface
//! forms a user might type), an optional Wikidata Q-ID, and the canonical
//! display name used in the response body. Each `topic` entry lists
//! multilingual detection keywords and the factual sentence the roleplay
//! frame should ground in — so adding a new persona or explanation topic in
//! any of the four supported languages does not require touching Rust code.
//!
//! The matcher contract is deliberately tiny:
//!
//! - `PersonaSeeds::matches_trigger` opts the prompt into roleplay mode when
//!   any universal trigger phrase appears as a substring.
//! - `PersonaSeeds::pick_persona` returns the first persona whose aliases
//!   match. When nothing matches, the caller uses `default_persona`.
//! - `PersonaSeeds::pick_topic` returns the first topic whose keywords match
//!   (callers fall back to `fallback_body` when nothing matches).
//!
//! The body template wraps the chosen topic body, keeping the
//! "Roleplay frame recorded for <persona>" preface uniform regardless of
//! which persona/topic combination fires.

use super::parser::{parse_lino, split_pipe_list};
use super::PERSONAS_LINO;

/// One persona surface form — the canonical display name, optional Wikidata
/// Q-ID anchor, and multilingual aliases that route prompts to this entry.
#[derive(Debug, Clone, Default)]
pub struct Persona {
    pub display_name: String,
    pub wikidata: String,
    pub aliases: Vec<String>,
}

/// One explanation topic — detection keywords (in every supported language)
/// plus the factual body the roleplay response grounds in.
#[derive(Debug, Clone, Default)]
pub struct PersonaTopic {
    pub slug: String,
    pub detection_keywords: Vec<String>,
    pub body: String,
}

/// Top-level persona-seed bundle.
///
/// Holds the universal trigger phrases that opt prompts into roleplay mode,
/// the persona catalogue, the topic catalogue, and the body template the
/// handler uses to wrap each persona/topic pair.
#[derive(Debug, Clone, Default)]
pub struct PersonaSeeds {
    pub triggers: Vec<String>,
    pub default_persona: String,
    pub body_template: String,
    pub fallback_body: String,
    pub personas: Vec<Persona>,
    pub topics: Vec<PersonaTopic>,
}

impl PersonaSeeds {
    /// Return `true` when any trigger phrase appears in the normalized
    /// prompt (already lowercased by the caller).
    #[must_use]
    pub fn matches_trigger(&self, normalized: &str) -> bool {
        self.triggers
            .iter()
            .any(|trigger| !trigger.is_empty() && normalized.contains(trigger.as_str()))
    }

    /// Pick the first persona whose alias appears in the normalized prompt.
    #[must_use]
    pub fn pick_persona(&self, normalized: &str) -> Option<&Persona> {
        self.personas.iter().find(|persona| {
            persona
                .aliases
                .iter()
                .any(|alias| !alias.is_empty() && normalized.contains(alias.as_str()))
        })
    }

    /// Pick the first topic whose detection keywords match the prompt.
    #[must_use]
    pub fn pick_topic(&self, normalized: &str) -> Option<&PersonaTopic> {
        self.topics.iter().find(|topic| {
            topic
                .detection_keywords
                .iter()
                .any(|keyword| !keyword.is_empty() && normalized.contains(keyword.as_str()))
        })
    }

    /// Render the full response body for a given persona + topic.
    ///
    /// Substitutes `<persona>` and `<body>` in the configured `body_template`.
    /// When the seed does not declare a template, falls back to the canonical
    /// "Roleplay frame recorded for X. ... Y" shape used by the original
    /// hardcoded handler so the response stays deterministic for tests.
    #[must_use]
    pub fn render_body(&self, persona: &str, body: &str) -> String {
        let template = if self.body_template.is_empty() {
            "Roleplay frame recorded for <persona>. I will keep the persona explicit and factual: <body>"
        } else {
            self.body_template.as_str()
        };
        template
            .replace("<persona>", persona)
            .replace("<body>", body)
    }
}

#[must_use]
pub fn persona_seeds() -> PersonaSeeds {
    let tree = parse_lino(PERSONAS_LINO);
    let mut seeds = PersonaSeeds::default();
    let Some(root) = tree.children.iter().find(|c| c.name == "personas") else {
        return seeds;
    };
    seeds.triggers = split_pipe_list(root.find_child_value("trigger"))
        .into_iter()
        .map(|t| t.to_lowercase())
        .collect();
    root.find_child_value("default_persona")
        .clone_into(&mut seeds.default_persona);
    root.find_child_value("body_template")
        .clone_into(&mut seeds.body_template);
    root.find_child_value("fallback_body")
        .clone_into(&mut seeds.fallback_body);
    for entry in &root.children {
        match entry.name.as_str() {
            "persona" => {
                if entry.id.is_empty() {
                    continue;
                }
                seeds.personas.push(Persona {
                    display_name: entry.id.clone(),
                    wikidata: entry.find_child_value("wikidata").to_owned(),
                    aliases: split_pipe_list(entry.find_child_value("aliases"))
                        .into_iter()
                        .map(|a| a.to_lowercase())
                        .collect(),
                });
            }
            "topic" => {
                if entry.id.is_empty() {
                    continue;
                }
                let body = entry.find_child_value("body").to_owned();
                if body.is_empty() {
                    continue;
                }
                seeds.topics.push(PersonaTopic {
                    slug: entry.id.clone(),
                    detection_keywords: split_pipe_list(
                        entry.find_child_value("detection_keywords"),
                    )
                    .into_iter()
                    .map(|k| k.to_lowercase())
                    .collect(),
                    body,
                });
            }
            _ => {}
        }
    }
    seeds
}
