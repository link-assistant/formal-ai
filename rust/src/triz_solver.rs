//! Derive a TRIZ worksheet from the two demands in the caller's prompt.

use regex::Regex;
use std::sync::OnceLock;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language;
use crate::seed::{self, parser::parse_lino};
use crate::solver_handlers::finalize_simple;

const SEED_PATH: &str = "data/seed/triz-principles.lino";
const INTENT: &str = "triz_resolution";

/// One reusable resolution family, independent of any benchmark problem.
#[derive(Clone, Debug)]
pub struct ResolutionFamily {
    pub method_id: String,
    pub name: String,
    pub applies_to: String,
    pub mechanism: String,
}

struct TrizRules {
    families: Vec<ResolutionFamily>,
    cues: Vec<String>,
    cue_pattern: Regex,
    pair_pattern: Regex,
    prefix_pattern: Regex,
    balanced_prefix: Regex,
    balanced_separator: Regex,
}

fn triz_rules() -> &'static TrizRules {
    static RULES: OnceLock<TrizRules> = OnceLock::new();
    RULES.get_or_init(|| {
        let source = seed::seed_files()
            .into_iter()
            .find(|(path, _)| *path == SEED_PATH)
            .map_or("", |(_, text)| text);
        let nodes = parse_lino(source).children;
        let families = nodes
            .iter()
            .filter(|record| record.find_child_value("record_type") == "triz_resolution_family")
            .map(|record| ResolutionFamily {
                method_id: record.find_child_value("method_id").to_owned(),
                name: record.find_child_value("name").to_owned(),
                applies_to: record.find_child_value("applies_to").to_owned(),
                mechanism: record.find_child_value("mechanism").to_owned(),
            })
            .filter(|family| !family.method_id.is_empty())
            .collect();
        let grammar = nodes.iter().find(|record| record.name == "triz_cues");
        let phrases = |name: &str| -> Vec<String> {
            grammar
                .into_iter()
                .flat_map(|record| &record.children)
                .filter(|child| child.name == name && !child.id.trim().is_empty())
                .map(|child| child.id.clone())
                .collect()
        };
        let alternation = |values: &[String]| {
            values
                .iter()
                .map(|phrase| regex::escape(phrase))
                .collect::<Vec<_>>()
                .join("|")
        };
        let cues = phrases("phrase");
        let cue_pattern = Regex::new(&format!("(?i){}", alternation(&cues))).expect("escaped cues");
        let pair_pattern = Regex::new(&format!("(?i){}", alternation(&phrases("separator"))))
            .expect("escaped separators");
        let prefix_pattern = Regex::new(&format!("(?i)^(?:{})", alternation(&phrases("prefix"))))
            .expect("escaped prefixes");
        let balanced_prefix = Regex::new(&format!(
            "(?i)^(?:{})",
            alternation(&phrases("balanced_prefix"))
        ))
        .expect("escaped balanced prefixes");
        let balanced_separator = Regex::new(&format!(
            "(?i){}",
            alternation(&phrases("balanced_separator"))
        ))
        .expect("escaped balanced separators");
        TrizRules {
            families,
            cues,
            cue_pattern,
            pair_pattern,
            prefix_pattern,
            balanced_prefix,
            balanced_separator,
        }
    })
}

/// The general resolution families, in seed order.
#[must_use]
pub fn triz_families() -> Vec<ResolutionFamily> {
    triz_rules().families.clone()
}

/// Phrases that mark a prompt as a contradiction question.
#[must_use]
pub fn triz_cues() -> Vec<String> {
    triz_rules().cues.clone()
}

fn demand(text: &str) -> Option<&str> {
    let trimmed = text.trim_matches(|character: char| {
        character.is_whitespace()
            || matches!(character, ':' | '：' | '?' | '？' | '.' | '。' | '!' | '！')
    });
    (trimmed.chars().any(char::is_alphanumeric)).then_some(trimmed)
}

/// Extract the original two sides of a stated trade-off for the claim router.
#[must_use]
pub fn triz_demands(prompt: &str) -> Option<(&str, &str)> {
    let rules = triz_rules();
    let cue = rules.cue_pattern.find(prompt)?;
    let tail = prompt[cue.end()..].trim_start();
    let frame = tail.strip_prefix([':', '：']).map_or(tail, str::trim_start);
    let frame = rules
        .prefix_pattern
        .find(frame)
        .map_or(frame, |prefix| &frame[prefix.end()..]);
    let (frame, separator) = if let Some(prefix) = rules.balanced_prefix.find(frame) {
        let frame = &frame[prefix.end()..];
        (frame, rules.balanced_separator.find(frame)?)
    } else {
        (frame, rules.pair_pattern.find(frame)?)
    };
    let left = demand(&frame[..separator.start()])?;
    let right = demand(&frame[separator.end()..])?;
    (left.to_lowercase() != right.to_lowercase()).then_some((left, right))
}

/// Return a worksheet for a stated trade-off, or request the missing demands.
pub fn handle_triz(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let rules = triz_rules();
    if rules.cues.is_empty()
        || !(rules.cue_pattern.is_match(normalized) || rules.cue_pattern.is_match(prompt))
    {
        return None;
    }
    let locale = language::detect(prompt).slug();
    log.append("triz_solver:cued", "contradiction question");
    let (body, response, confidence) = if let Some((left, right)) = triz_demands(prompt) {
        log.append("triz_solver:demands", format!("{left} / {right}"));
        let families = rules
            .families
            .iter()
            .map(|family| format!("- {}: {}", family.name, family.mechanism))
            .collect::<Vec<_>>()
            .join("\n");
        let response = "triz_resolution_map";
        let body = seed::fill_slots(
            &seed::localized_response(response, locale).unwrap_or_default(),
            &[("families", &families), ("left", left), ("right", right)],
        );
        (body, response, 0.7)
    } else {
        log.append("triz_resolution:refusal", "two demands required");
        let response = "triz_resolution_missing_demands";
        (
            seed::localized_response(response, locale).unwrap_or_default(),
            response,
            0.45,
        )
    };
    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        &format!("response:{response}"),
        &body,
        confidence,
    ))
}
