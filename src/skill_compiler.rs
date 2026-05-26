//! Natural-language skill compiler.
//!
//! The first compiler target is intentionally narrow and deterministic:
//! trigger/response prose such as `When I say "X", answer "Y"` lowers to a
//! reviewable associative package containing a trigger rule and compiled
//! response handler. The solver can replay that package from dialog history
//! and record the reuse as a `cache_hit`.

use std::error::Error;
use std::fmt;

use crate::engine::{normalize_prompt, stable_id, KNOWLEDGE_SCHEMA_VERSION};
use crate::link_store::{DoubletLink, LinkRecord};
use crate::links_format::format_lino_record;

/// A reusable, reviewable package compiled from one natural-language skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillPackage {
    /// Stable package id used as the cache key.
    pub id: String,
    /// Legacy dialog-local behavior-rule id kept for existing UI wording.
    pub legacy_behavior_rule_id: String,
    /// Original prose instruction supplied by the user or seed data.
    pub source_description: String,
    /// Surface form that triggers the compiled package.
    pub trigger: String,
    /// Normalized trigger used by the deterministic replay matcher.
    pub normalized_trigger: String,
    /// Response emitted by the compiled handler.
    pub response: String,
    /// Stable id of the generated trigger/substitution rule.
    pub rule_id: String,
    /// Stable id of the generated compiled handler.
    pub handler_id: String,
}

/// Deterministic replay result from a compiled package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillReplay {
    /// Package that replayed.
    pub package_id: String,
    /// Trigger rule selected for this replay.
    pub rule_id: String,
    /// Compiled handler selected for this replay.
    pub handler_id: String,
    /// User-facing answer projected by the compiled handler.
    pub answer: String,
    /// Cache-hit payload that should be appended to the event log.
    pub cache_hit: String,
}

/// Error returned when a skill description cannot be lowered safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillCompileError {
    /// The text did not match a supported deterministic skill shape.
    UnsupportedShape,
}

impl fmt::Display for SkillCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedShape => {
                write!(formatter, "unsupported natural-language skill shape")
            }
        }
    }
}

impl Error for SkillCompileError {}

/// Compile a natural-language skill into a reusable associative package.
///
/// Supported shapes include `When I say "X", answer "Y"`, `If I ask "X",
/// reply "Y"`, and the multilingual `When "X" then "Y"` forms already used by
/// behavior-rule teaching.
pub fn compile_natural_language_skill(
    description: &str,
) -> Result<CompiledSkillPackage, SkillCompileError> {
    let Some((trigger, response)) = extract_trigger_response(description) else {
        return Err(SkillCompileError::UnsupportedShape);
    };
    Ok(CompiledSkillPackage::new(description, &trigger, &response))
}

/// Knowledge-export record for the natural-language skill compiler.
pub(crate) fn natural_language_skill_compiler_record() -> String {
    format_lino_record(
        "natural_language_skill_compiler",
        &[
            ("type", String::from("compiled_handler")),
            ("rule_shape", String::from("natural_language_skill")),
            ("package_type", String::from("CompiledSkillPackage")),
            ("module", String::from("src/skill_compiler.rs")),
            ("exports", String::from("compile_natural_language_skill")),
            ("trigger_record", String::from("CompiledSkillTriggerRule")),
            ("handler_record", String::from("CompiledSkillHandler")),
            ("cache_event", String::from("cache_hit")),
            ("source", String::from("ARCHITECTURE.md section 9 #5")),
        ],
    )
}

impl CompiledSkillPackage {
    /// Create a compiled package from a verified trigger/response pair.
    #[must_use]
    pub fn new(source_description: &str, trigger: &str, response: &str) -> Self {
        let normalized_trigger = normalize_prompt(trigger);
        let canonical = format!("{normalized_trigger}\n{response}");
        let id = stable_id("compiled_skill", &canonical);
        let rule_id = stable_id(
            "compiled_skill_rule",
            &format!("{id}:rule:{normalized_trigger}"),
        );
        let handler_id = stable_id(
            "compiled_skill_handler",
            &format!("{id}:handler:{response}"),
        );
        let legacy_behavior_rule_id =
            stable_id("behavior_rule_runtime", &format!("{trigger}\n{response}"));
        Self {
            id,
            legacy_behavior_rule_id,
            source_description: source_description.to_owned(),
            trigger: trigger.to_owned(),
            normalized_trigger,
            response: response.to_owned(),
            rule_id,
            handler_id,
        }
    }

    /// Replay the package when `prompt` matches the compiled trigger.
    #[must_use]
    pub fn replay(&self, prompt: &str) -> Option<CompiledSkillReplay> {
        if normalize_prompt(prompt) != self.normalized_trigger {
            return None;
        }
        Some(CompiledSkillReplay {
            package_id: self.id.clone(),
            rule_id: self.rule_id.clone(),
            handler_id: self.handler_id.clone(),
            answer: self.response.clone(),
            cache_hit: self.id.clone(),
        })
    }

    /// Export the compiled package as reviewable Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        format_lino_record(
            &self.id,
            &[
                ("type", String::from("compiled_skill_package")),
                ("schema_version", String::from(KNOWLEDGE_SCHEMA_VERSION)),
                ("package_kind", String::from("associative_package")),
                ("source", String::from("natural_language_skill")),
                ("source_description", self.source_description.clone()),
                ("trigger_rule", self.rule_id.clone()),
                ("trigger", self.trigger.clone()),
                ("normalized_trigger", self.normalized_trigger.clone()),
                ("compiled_handler", self.handler_id.clone()),
                ("handler_kind", String::from("deterministic_response")),
                ("response", self.response.clone()),
                ("replay_mode", String::from("exact_normalized_prompt")),
                (
                    "legacy_behavior_rule_id",
                    self.legacy_behavior_rule_id.clone(),
                ),
            ],
        )
    }

    /// Project the package, trigger rule, and handler to E1-style link records.
    #[must_use]
    pub fn link_records(&self) -> Vec<LinkRecord> {
        vec![
            link_record(
                &self.id,
                "CompiledSkillPackage",
                "associative_package",
                &stable_id("natural_language_skill", &self.source_description),
                &[
                    ("source_description", self.source_description.as_str()),
                    ("trigger_rule", self.rule_id.as_str()),
                    ("compiled_handler", self.handler_id.as_str()),
                    ("replay_mode", "exact_normalized_prompt"),
                ],
            ),
            link_record(
                &self.rule_id,
                "CompiledSkillTriggerRule",
                "substitution_rule",
                &self.id,
                &[
                    ("trigger", self.trigger.as_str()),
                    ("normalized_trigger", self.normalized_trigger.as_str()),
                    ("handler", self.handler_id.as_str()),
                ],
            ),
            link_record(
                &self.handler_id,
                "CompiledSkillHandler",
                "deterministic_response_handler",
                &self.id,
                &[
                    ("handler_kind", "deterministic_response"),
                    ("response", self.response.as_str()),
                ],
            ),
        ]
    }
}

fn extract_trigger_response(description: &str) -> Option<(String, String)> {
    if !looks_like_skill_description(description) {
        return None;
    }
    let spans = code_spans(description);
    if spans.len() < 2 {
        return None;
    }
    let trigger = spans[0].trim();
    let response = spans[1].trim();
    if trigger.is_empty() || response.is_empty() {
        return None;
    }
    Some((trigger.to_owned(), response.to_owned()))
}

fn looks_like_skill_description(description: &str) -> bool {
    let lower = description.to_lowercase();
    if explicit_teaching_form(&lower) {
        return true;
    }
    for (head, link) in WHEN_THEN_KEYWORD_PAIRS {
        if let Some(head_pos) = lower.find(head) {
            if let Some(link_pos) = lower[head_pos + head.len()..].find(link) {
                let absolute_link_pos = head_pos + head.len() + link_pos;
                let before_link = &description[head_pos..absolute_link_pos];
                let after_link = &description[absolute_link_pos + link.len()..];
                if before_link.contains('`') && after_link.contains('`') {
                    return true;
                }
            }
        }
    }
    false
}

fn explicit_teaching_form(lower: &str) -> bool {
    ((lower.contains("when i say")
        || lower.contains("when the user says")
        || lower.contains("when the user asks")
        || lower.contains("if i ask"))
        && (lower.contains("answer") || lower.contains("reply") || lower.contains("respond")))
        || lower.contains("add behavior rule")
        || lower.contains("update behavior rule")
        || (lower.contains("когда я скажу") && lower.contains("ответ"))
        || (lower.contains("если я спрошу") && lower.contains("ответ"))
        || lower.contains("добавь правило поведения")
        || lower.contains("обнови правило поведения")
}

const WHEN_THEN_KEYWORD_PAIRS: &[(&str, &str)] = &[
    ("when ", " then "),
    ("when ", " do "),
    ("когда ", " тогда "),
    ("когда ", " делай "),
    ("когда ", " сделай "),
    ("когда ", " отвечай "),
    ("когда ", " отвечать "),
    ("если ", " то "),
    ("जब ", " तब "),
    ("जब ", " तो "),
    ("当 ", " 时 "),
    ("当 ", " 则 "),
    ("当 ", " 回答 "),
    ("当 ", "时回答 "),
    ("当 ", "则回答 "),
];

fn code_spans(text: &str) -> Vec<String> {
    text.split('`')
        .enumerate()
        .filter_map(|(index, part)| {
            let trimmed = part.trim();
            if index % 2 == 1 && !trimmed.is_empty() {
                Some(trimmed.to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn link_record(
    record_id: &str,
    record_type: &str,
    subtype: &str,
    source_id: &str,
    fields: &[(&str, &str)],
) -> LinkRecord {
    let mut links = Vec::new();
    push_doublet(&mut links, record_id, "Type");
    push_doublet(&mut links, "Type", record_type);
    push_doublet(&mut links, record_type, "SubType");
    push_doublet(&mut links, "SubType", subtype);
    push_doublet(&mut links, subtype, "Value");
    push_doublet(&mut links, record_id, source_id);
    push_field(
        &mut links,
        record_id,
        "schema_version",
        KNOWLEDGE_SCHEMA_VERSION,
    );
    for (key, value) in fields {
        push_field(&mut links, record_id, key, value);
    }
    LinkRecord {
        stable_id: record_id.to_owned(),
        schema_version: String::from(KNOWLEDGE_SCHEMA_VERSION),
        record_type: record_type.to_owned(),
        source_id: source_id.to_owned(),
        links,
    }
}

fn push_field(links: &mut Vec<DoubletLink>, record_id: &str, key: &str, value: &str) {
    if value.is_empty() {
        return;
    }
    let field = format!("field:{key}");
    let field_value = format!("value:{value}");
    push_doublet(links, record_id, &field);
    push_doublet(links, &field, &field_value);
}

fn push_doublet(links: &mut Vec<DoubletLink>, from: &str, to: &str) {
    links.push(DoubletLink {
        index: stable_id("doublet", &format!("{from}->{to}")),
        from: from.to_owned(),
        to: to.to_owned(),
    });
}

#[cfg(test)]
mod tests {
    use super::{compile_natural_language_skill, SkillCompileError};

    #[test]
    fn unsupported_shape_is_rejected() {
        let err = compile_natural_language_skill("This is only a note.")
            .expect_err("free text should not compile");
        assert_eq!(err, SkillCompileError::UnsupportedShape);
    }
}
