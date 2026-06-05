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
use crate::seed::{self, Slot};

mod structured;

use structured::{
    handler_signature, handler_stub_source, parse_structured_skill, structured_canonical,
    StructuredSkillSpec,
};

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
    /// Human-readable skill name from the structured definition, when present.
    pub skill_name: String,
    /// Typed arguments accepted by the compiled skill subset.
    pub inputs: Vec<CompiledSkillInput>,
    /// Preconditions checked before a generated handler is considered valid.
    pub preconditions: Vec<CompiledSkillPrecondition>,
    /// Ordered procedure steps lowered from the skill definition.
    pub steps: Vec<CompiledSkillStep>,
    /// Declared deterministic effects of the procedure.
    pub effects: Vec<CompiledSkillEffect>,
    /// Generated replay tests used both as examples and deterministic fixtures.
    pub expected_tests: Vec<CompiledSkillExpectedTest>,
    /// Explicit package/tool permissions required by the compiled skill.
    pub required_permissions: Vec<CompiledSkillPermission>,
    /// Target-specific handler stubs that can be reviewed before implementation.
    pub handler_stubs: Vec<CompiledSkillHandlerStub>,
}

/// Typed argument accepted by the structured skill subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillInput {
    pub id: String,
    pub name: String,
    pub value_type: String,
}

/// Deterministic precondition from a structured skill definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillPrecondition {
    pub id: String,
    pub description: String,
}

/// Ordered procedure step from a structured skill definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillStep {
    pub id: String,
    pub order: usize,
    pub description: String,
}

/// Declared effect of a structured skill definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillEffect {
    pub id: String,
    pub description: String,
}

/// Generated expected test for deterministic replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillExpectedTest {
    pub id: String,
    pub input: String,
    pub normalized_input: String,
    pub expected_output: String,
    pub trigger_id: String,
    pub handler_id: String,
}

/// Explicit package/tool permission required by a structured skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillPermission {
    pub id: String,
    pub capability: String,
    pub description: String,
}

/// Reviewable generated handler placeholder for a target runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledSkillHandlerStub {
    pub id: String,
    pub target: String,
    pub signature: String,
    pub source: String,
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
    /// The skill asks for nondeterministic or otherwise unsupported behavior.
    UnsupportedInstruction { reason: String },
    /// The skill names a permissioned tool/action without an explicit grant.
    PermissionRequired { capability: String },
}

impl fmt::Display for SkillCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedShape => {
                write!(formatter, "unsupported natural-language skill shape")
            }
            Self::UnsupportedInstruction { reason } => {
                write!(
                    formatter,
                    "unsupported natural-language instruction: {reason}"
                )
            }
            Self::PermissionRequired { capability } => {
                write!(formatter, "skill requires explicit permission {capability}")
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
    if let Some(spec) = parse_structured_skill(description)? {
        return CompiledSkillPackage::from_structured(description, spec);
    }
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
            skill_name: String::from("trigger_response_skill"),
            inputs: Vec::new(),
            preconditions: Vec::new(),
            steps: Vec::new(),
            effects: Vec::new(),
            expected_tests: Vec::new(),
            required_permissions: Vec::new(),
            handler_stubs: Vec::new(),
        }
    }

    fn from_structured(
        source_description: &str,
        spec: StructuredSkillSpec,
    ) -> Result<Self, SkillCompileError> {
        let Some(primary) = spec.primary_replay() else {
            return Err(SkillCompileError::UnsupportedShape);
        };
        let normalized_trigger = normalize_prompt(&primary.trigger);
        let canonical = structured_canonical(source_description, &spec, &primary);
        let id = stable_id("compiled_skill", &canonical);
        let rule_id = stable_id(
            "compiled_skill_rule",
            &format!("{id}:rule:{normalized_trigger}"),
        );
        let handler_id = stable_id(
            "compiled_skill_handler",
            &format!("{id}:handler:{}", primary.response),
        );
        let legacy_behavior_rule_id = stable_id(
            "behavior_rule_runtime",
            &format!("{}\n{}", primary.trigger, primary.response),
        );
        let inputs = spec
            .inputs
            .into_iter()
            .map(|input| {
                let record_id = stable_id("compiled_skill_input", &format!("{id}:{}", input.name));
                CompiledSkillInput {
                    id: record_id,
                    name: input.name,
                    value_type: input.value_type,
                }
            })
            .collect::<Vec<_>>();
        let preconditions = spec
            .preconditions
            .into_iter()
            .map(|description| CompiledSkillPrecondition {
                id: stable_id(
                    "compiled_skill_precondition",
                    &format!("{id}:{description}"),
                ),
                description,
            })
            .collect();
        let steps = spec
            .steps
            .into_iter()
            .enumerate()
            .map(|(index, description)| CompiledSkillStep {
                id: stable_id(
                    "compiled_skill_step",
                    &format!("{id}:{}:{description}", index + 1),
                ),
                order: index + 1,
                description,
            })
            .collect::<Vec<_>>();
        let effects = spec
            .effects
            .into_iter()
            .map(|description| CompiledSkillEffect {
                id: stable_id("compiled_skill_effect", &format!("{id}:{description}")),
                description,
            })
            .collect();
        let expected_tests = spec
            .expected_tests
            .into_iter()
            .map(|test| {
                let normalized_input = normalize_prompt(&test.input);
                let test_id = stable_id(
                    "compiled_skill_test",
                    &format!("{id}:{normalized_input}:{}", test.expected_output),
                );
                CompiledSkillExpectedTest {
                    id: test_id,
                    trigger_id: stable_id(
                        "compiled_skill_rule",
                        &format!("{id}:test_rule:{normalized_input}"),
                    ),
                    handler_id: stable_id(
                        "compiled_skill_handler",
                        &format!(
                            "{id}:test_handler:{normalized_input}:{}",
                            test.expected_output
                        ),
                    ),
                    input: test.input,
                    normalized_input,
                    expected_output: test.expected_output,
                }
            })
            .collect::<Vec<_>>();
        let required_permissions = spec
            .permissions
            .into_iter()
            .map(|permission| CompiledSkillPermission {
                id: stable_id(
                    "compiled_skill_permission",
                    &format!("{id}:{}:{}", permission.capability, permission.description),
                ),
                capability: permission.capability,
                description: permission.description,
            })
            .collect::<Vec<_>>();
        let skill_name = spec.name.clone();
        let handler_stubs = spec
            .targets
            .into_iter()
            .map(|target| {
                let signature = handler_signature(&skill_name, &inputs);
                let source = handler_stub_source(&target, &skill_name, &inputs, &steps, &primary);
                CompiledSkillHandlerStub {
                    id: stable_id("compiled_skill_handler_stub", &format!("{id}:{target}")),
                    target,
                    signature,
                    source,
                }
            })
            .collect();
        Ok(Self {
            id,
            legacy_behavior_rule_id,
            source_description: source_description.to_owned(),
            trigger: primary.trigger,
            normalized_trigger,
            response: primary.response,
            rule_id,
            handler_id,
            skill_name,
            inputs,
            preconditions,
            steps,
            effects,
            expected_tests,
            required_permissions,
            handler_stubs,
        })
    }

    /// Replay the package when `prompt` matches the compiled trigger.
    #[must_use]
    pub fn replay(&self, prompt: &str) -> Option<CompiledSkillReplay> {
        let normalized = normalize_prompt(prompt);
        if let Some(test) = self
            .expected_tests
            .iter()
            .find(|test| test.normalized_input == normalized)
        {
            return Some(CompiledSkillReplay {
                package_id: self.id.clone(),
                rule_id: test.trigger_id.clone(),
                handler_id: test.handler_id.clone(),
                answer: test.expected_output.clone(),
                cache_hit: self.id.clone(),
            });
        }
        if normalized != self.normalized_trigger {
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
        let mut out = String::new();
        push_lino_node(&mut out, 0, &self.id, None);
        push_lino_node(&mut out, 2, "type", Some("compiled_skill_package"));
        push_lino_node(
            &mut out,
            2,
            "schema_version",
            Some(KNOWLEDGE_SCHEMA_VERSION),
        );
        push_lino_node(&mut out, 2, "package_kind", Some("associative_package"));
        push_lino_node(&mut out, 2, "source", Some("natural_language_skill"));
        push_lino_node(
            &mut out,
            2,
            "source_description",
            Some(&self.source_description),
        );
        push_lino_node(&mut out, 2, "skill_name", Some(&self.skill_name));
        push_lino_node(&mut out, 2, "trigger_rule", Some(&self.rule_id));
        push_lino_node(&mut out, 2, "trigger", Some(&self.trigger));
        push_lino_node(
            &mut out,
            2,
            "normalized_trigger",
            Some(&self.normalized_trigger),
        );
        push_lino_node(&mut out, 2, "compiled_handler", Some(&self.handler_id));
        push_lino_node(&mut out, 2, "handler_kind", Some("deterministic_response"));
        push_lino_node(&mut out, 2, "response", Some(&self.response));
        push_lino_node(&mut out, 2, "replay_mode", Some("exact_normalized_prompt"));
        push_lino_node(
            &mut out,
            2,
            "legacy_behavior_rule_id",
            Some(&self.legacy_behavior_rule_id),
        );
        for input in &self.inputs {
            push_lino_node(&mut out, 2, "input", Some(&input.name));
            push_lino_node(&mut out, 4, "id", Some(&input.id));
            push_lino_node(&mut out, 4, "type", Some(&input.value_type));
        }
        for precondition in &self.preconditions {
            push_lino_node(&mut out, 2, "precondition", Some(&precondition.id));
            push_lino_node(&mut out, 4, "description", Some(&precondition.description));
        }
        for step in &self.steps {
            let order = step.order.to_string();
            push_lino_node(&mut out, 2, "step", Some(&step.id));
            push_lino_node(&mut out, 4, "order", Some(&order));
            push_lino_node(&mut out, 4, "description", Some(&step.description));
        }
        for effect in &self.effects {
            push_lino_node(&mut out, 2, "effect", Some(&effect.id));
            push_lino_node(&mut out, 4, "description", Some(&effect.description));
        }
        for test in &self.expected_tests {
            push_lino_node(&mut out, 2, "expected_test", Some(&test.id));
            push_lino_node(&mut out, 4, "input", Some(&test.input));
            push_lino_node(
                &mut out,
                4,
                "normalized_input",
                Some(&test.normalized_input),
            );
            push_lino_node(&mut out, 4, "expected_output", Some(&test.expected_output));
            push_lino_node(&mut out, 4, "trigger_rule", Some(&test.trigger_id));
            push_lino_node(&mut out, 4, "compiled_handler", Some(&test.handler_id));
        }
        for permission in &self.required_permissions {
            push_lino_node(&mut out, 2, "permission", Some(&permission.id));
            push_lino_node(&mut out, 4, "capability", Some(&permission.capability));
            push_lino_node(&mut out, 4, "description", Some(&permission.description));
        }
        for stub in &self.handler_stubs {
            push_lino_node(&mut out, 2, "handler_stub", Some(&stub.id));
            push_lino_node(&mut out, 4, "target", Some(&stub.target));
            push_lino_node(&mut out, 4, "signature", Some(&stub.signature));
            push_lino_node(&mut out, 4, "source_code", Some(&stub.source));
        }
        out.trim_end().to_owned()
    }

    /// Project the package, trigger rule, and handler to E1-style link records.
    #[must_use]
    pub fn link_records(&self) -> Vec<LinkRecord> {
        let mut records = vec![
            link_record(
                &self.id,
                "CompiledSkillPackage",
                "associative_package",
                &stable_id("natural_language_skill", &self.source_description),
                &[
                    ("source_description", self.source_description.as_str()),
                    ("skill_name", self.skill_name.as_str()),
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
        ];
        for input in &self.inputs {
            records.push(link_record(
                &input.id,
                "CompiledSkillInput",
                "typed_input",
                &self.id,
                &[
                    ("name", input.name.as_str()),
                    ("type", input.value_type.as_str()),
                ],
            ));
        }
        for precondition in &self.preconditions {
            records.push(link_record(
                &precondition.id,
                "CompiledSkillPrecondition",
                "precondition",
                &self.id,
                &[("description", precondition.description.as_str())],
            ));
        }
        for step in &self.steps {
            let order = step.order.to_string();
            records.push(link_record(
                &step.id,
                "CompiledSkillStep",
                "procedure_step",
                &self.id,
                &[
                    ("order", order.as_str()),
                    ("description", step.description.as_str()),
                ],
            ));
        }
        for effect in &self.effects {
            records.push(link_record(
                &effect.id,
                "CompiledSkillEffect",
                "declared_effect",
                &self.id,
                &[("description", effect.description.as_str())],
            ));
        }
        for test in &self.expected_tests {
            records.push(link_record(
                &test.id,
                "CompiledSkillExpectedTest",
                "generated_test",
                &self.id,
                &[
                    ("input", test.input.as_str()),
                    ("expected_output", test.expected_output.as_str()),
                    ("trigger_rule", test.trigger_id.as_str()),
                    ("compiled_handler", test.handler_id.as_str()),
                ],
            ));
        }
        for permission in &self.required_permissions {
            records.push(link_record(
                &permission.id,
                "CompiledSkillPermission",
                "permission_grant",
                &self.id,
                &[
                    ("capability", permission.capability.as_str()),
                    ("description", permission.description.as_str()),
                ],
            ));
        }
        for stub in &self.handler_stubs {
            records.push(link_record(
                &stub.id,
                "CompiledSkillHandlerStub",
                "generated_handler_stub",
                &self.id,
                &[
                    ("target", stub.target.as_str()),
                    ("signature", stub.signature.as_str()),
                    ("source_code", stub.source.as_str()),
                ],
            ));
        }
        records
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

/// True when `description` reads as a teachable skill — either an explicit
/// teaching form or a conditional when-then frame that quotes a trigger and reply.
///
/// No keyword is named here. The explicit teaching form is read from the
/// [`skill_teaching_trigger_lead`](seed::ROLE_SKILL_TEACHING_TRIGGER_LEAD),
/// [`skill_teaching_response_verb`](seed::ROLE_SKILL_TEACHING_RESPONSE_VERB), and
/// [`behavior_rule_edit_directive`](seed::ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE) roles;
/// the when-then frames are read from the
/// [`skill_when_then_pair`](seed::ROLE_SKILL_WHEN_THEN_PAIR) role. Each frame is a
/// [`Slot::Circumfix`] surface whose literal before the ellipsis … (U+2026) is the
/// head clause and whose literal after it is the link clause. A description teaches
/// a skill when it contains a head, a link following that head, a backtick-quoted
/// trigger between them, and a backtick-quoted reply after the link — the same
/// byte test that once ran against a hardcoded keyword-pair table, now covering
/// every supported language from the data.
fn looks_like_skill_description(description: &str) -> bool {
    let lower = description.to_lowercase();
    if explicit_teaching_form(&lower) {
        return true;
    }
    seed::lexicon()
        .role_word_forms(seed::ROLE_SKILL_WHEN_THEN_PAIR)
        .into_iter()
        .filter(|form| form.slot() == Slot::Circumfix)
        .any(|form| {
            let head = form.before_slot();
            let link = form.after_slot();
            let Some(head_pos) = lower.find(head) else {
                return false;
            };
            let Some(link_pos) = lower[head_pos + head.len()..].find(link) else {
                return false;
            };
            let absolute_link_pos = head_pos + head.len() + link_pos;
            let before_link = &description[head_pos..absolute_link_pos];
            let after_link = &description[absolute_link_pos + link.len()..];
            before_link.contains('`') && after_link.contains('`')
        })
}

/// True when `lower` (an already-lower-cased description) is an explicit teaching
/// instruction — a trigger lead paired with a response verb, or a standalone
/// behaviour-rule edit directive.
///
/// Every surface is read from the lexicon by meaning rather than named here. A
/// trigger lead ([`ROLE_SKILL_TEACHING_TRIGGER_LEAD`](seed::ROLE_SKILL_TEACHING_TRIGGER_LEAD))
/// teaches a skill only when it co-occurs with a response verb
/// ([`ROLE_SKILL_TEACHING_RESPONSE_VERB`](seed::ROLE_SKILL_TEACHING_RESPONSE_VERB));
/// an edit directive
/// ([`ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE`](seed::ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE))
/// is recognised on its own. Each role is matched as a raw substring through
/// [`Lexicon::mentions_role_raw`](seed::Lexicon::mentions_role_raw) — the surfaces
/// are stored lower-cased and the caller has already lower-cased the description,
/// so an inflectable stem (the Russian "ответ") still folds its endings.
fn explicit_teaching_form(lower: &str) -> bool {
    let lexicon = seed::lexicon();
    (lexicon.mentions_role_raw(seed::ROLE_SKILL_TEACHING_TRIGGER_LEAD, lower)
        && lexicon.mentions_role_raw(seed::ROLE_SKILL_TEACHING_RESPONSE_VERB, lower))
        || lexicon.mentions_role_raw(seed::ROLE_BEHAVIOR_RULE_EDIT_DIRECTIVE, lower)
}

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

fn push_lino_node(out: &mut String, indent: usize, name: &str, value: Option<&str>) {
    out.push_str(&" ".repeat(indent));
    out.push_str(name);
    if let Some(value) = value {
        out.push_str(" \"");
        out.push_str(&escape_lino_value(value));
        out.push('"');
    }
    out.push('\n');
}

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}

#[path = "source_tests/skill_compiler/tests.rs"]
mod tests;
