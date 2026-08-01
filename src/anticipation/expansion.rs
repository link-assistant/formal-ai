//! Data-driven request-class expansion for anticipatory dreaming.

use std::collections::BTreeSet;

use crate::engine::normalize_prompt;
use crate::seed::{self, Slot, WordForm};

use super::{AnticipationConfig, IntentClass, Observation, PromptVariant};

pub(super) fn expand_class(
    class: &IntentClass,
    observations: &[Observation],
    config: &AnticipationConfig,
) -> Vec<PromptVariant> {
    let mut variants = Vec::new();
    let mut seen = BTreeSet::new();
    for observation in observations
        .iter()
        .filter(|observation| observation.class.id == class.id)
    {
        push_variant(
            &mut variants,
            &mut seen,
            &observation.prompt,
            format!("parameter:{}", observation.event_id),
            &observation.event_id,
            config.max_variations_per_prediction,
        );
        expand_operations(observation, &mut variants, &mut seen, config);
        expand_meanings(observation, &mut variants, &mut seen, config);
        if variants.len() >= config.max_variations_per_prediction {
            break;
        }
    }
    variants
}

fn expand_meanings(
    observation: &Observation,
    variants: &mut Vec<PromptVariant>,
    seen: &mut BTreeSet<String>,
    config: &AnticipationConfig,
) {
    let normalized = normalize_prompt(&observation.prompt);
    let language = crate::language::detect(&observation.prompt);
    for meaning in &seed::lexicon().meanings {
        let forms = meaning
            .lexemes
            .iter()
            .filter(|lexeme| lexeme.language == language.slug())
            .flat_map(|lexeme| lexeme.words.iter())
            .collect::<Vec<_>>();
        for matched in &forms {
            let Some(subject) = template_subject(matched, &normalized) else {
                continue;
            };
            for replacement in &forms {
                if replacement.text == matched.text {
                    continue;
                }
                let prompt = instantiate_variant(matched, replacement, &normalized, &subject);
                push_variant(
                    variants,
                    seen,
                    &prompt,
                    format!("meaning:{}", meaning.slug),
                    &observation.event_id,
                    config.max_variations_per_prediction,
                );
                if variants.len() >= config.max_variations_per_prediction {
                    return;
                }
            }
        }
    }
}

fn expand_operations(
    observation: &Observation,
    variants: &mut Vec<PromptVariant>,
    seen: &mut BTreeSet<String>,
    config: &AnticipationConfig,
) {
    let normalized = normalize_prompt(&observation.prompt);
    let language = crate::language::detect(&observation.prompt);
    let vocabulary = seed::operation_vocabulary();
    for operation in vocabulary
        .operations
        .iter()
        .filter(|operation| operation.matches(&normalized))
    {
        let Some(forms) = operation.languages.get(language.slug()) else {
            continue;
        };
        for phrase in &forms.phrases {
            let needle = normalize_prompt(phrase);
            if !surface_present(&normalized, &needle) {
                continue;
            }
            for replacement in &forms.phrases {
                let prompt = replace_surface(&normalized, &needle, &normalize_prompt(replacement));
                push_variant(
                    variants,
                    seen,
                    &prompt,
                    format!("operation:{}", operation.canonical),
                    &observation.event_id,
                    config.max_variations_per_prediction,
                );
                if variants.len() >= config.max_variations_per_prediction {
                    return;
                }
            }
        }
    }
}

fn push_variant(
    variants: &mut Vec<PromptVariant>,
    seen: &mut BTreeSet<String>,
    prompt: &str,
    source: String,
    base_event_id: &str,
    limit: usize,
) {
    let normalized = normalize_prompt(prompt);
    if normalized.is_empty() || variants.len() >= limit || !seen.insert(normalized) {
        return;
    }
    variants.push(PromptVariant {
        prompt: prompt.trim().to_owned(),
        source,
        base_event_id: base_event_id.to_owned(),
    });
}

enum TemplateSubject {
    Slot(String),
    Bare(String),
}

fn template_subject(form: &WordForm, prompt: &str) -> Option<TemplateSubject> {
    let before = normalize_prompt(form.before_slot());
    let after = normalize_prompt(form.after_slot());
    match form.slot() {
        Slot::Prefix => prompt
            .strip_prefix(&before)
            .map(str::trim)
            .filter(|subject| !subject.is_empty())
            .map(|subject| TemplateSubject::Slot(subject.to_owned())),
        Slot::Suffix => prompt
            .strip_suffix(&after)
            .map(str::trim)
            .filter(|subject| !subject.is_empty())
            .map(|subject| TemplateSubject::Slot(subject.to_owned())),
        Slot::Circumfix => prompt
            .strip_prefix(&before)
            .and_then(|rest| rest.strip_suffix(&after))
            .map(str::trim)
            .filter(|subject| !subject.is_empty())
            .map(|subject| TemplateSubject::Slot(subject.to_owned())),
        Slot::Bare => surface_present(prompt, &before).then_some(TemplateSubject::Bare(before)),
    }
}

fn instantiate_variant(
    matched: &WordForm,
    replacement: &WordForm,
    prompt: &str,
    subject: &TemplateSubject,
) -> String {
    match subject {
        TemplateSubject::Bare(needle) => {
            replace_surface(prompt, needle, &normalize_prompt(&replacement.text))
        }
        TemplateSubject::Slot(subject) => {
            let before = normalize_prompt(replacement.before_slot());
            let after = normalize_prompt(replacement.after_slot());
            match replacement.slot() {
                Slot::Prefix => format!("{before} {subject}"),
                Slot::Suffix => format!("{subject} {after}"),
                Slot::Circumfix => format!("{before} {subject} {after}"),
                Slot::Bare => replace_surface(
                    prompt,
                    &normalize_prompt(&matched.text.replace('…', subject)),
                    &normalize_prompt(&replacement.text),
                ),
            }
        }
    }
}

fn surface_present(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    haystack.match_indices(needle).any(|(start, _)| {
        let end = start + needle.len();
        let left = haystack[..start]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_alphanumeric());
        let right = haystack[end..]
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_alphanumeric());
        left && right
    })
}

fn replace_surface(haystack: &str, needle: &str, replacement: &str) -> String {
    haystack.find(needle).map_or_else(
        || haystack.to_owned(),
        |start| {
            format!(
                "{}{}{}",
                &haystack[..start],
                replacement,
                &haystack[start + needle.len()..]
            )
        },
    )
}
