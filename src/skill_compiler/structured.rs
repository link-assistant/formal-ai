use std::{collections::BTreeSet, fmt::Write as _};

use crate::engine::normalize_prompt;
use crate::seed;

use super::{
    code_spans, extract_trigger_response, looks_like_skill_description, CompiledSkillInput,
    CompiledSkillStep, SkillCompileError,
};

#[derive(Debug, Clone, Default)]
pub(super) struct StructuredSkillSpec {
    pub(super) name: String,
    pub(super) inputs: Vec<StructuredInput>,
    pub(super) preconditions: Vec<String>,
    pub(super) steps: Vec<String>,
    pub(super) effects: Vec<String>,
    pub(super) expected_tests: Vec<StructuredExpectedTest>,
    pub(super) permissions: Vec<StructuredPermission>,
    pub(super) targets: Vec<String>,
    pub(super) trigger_response: Option<ReplayPair>,
}

#[derive(Debug, Clone)]
pub(super) struct StructuredInput {
    pub(super) name: String,
    pub(super) value_type: String,
}

#[derive(Debug, Clone)]
pub(super) struct StructuredExpectedTest {
    pub(super) input: String,
    pub(super) expected_output: String,
}

#[derive(Debug, Clone)]
pub(super) struct StructuredPermission {
    pub(super) capability: String,
    pub(super) description: String,
}

#[derive(Debug, Clone)]
pub(super) struct ReplayPair {
    pub(super) trigger: String,
    pub(super) response: String,
}

impl StructuredSkillSpec {
    pub(super) fn primary_replay(&self) -> Option<ReplayPair> {
        self.trigger_response.clone().or_else(|| {
            self.expected_tests.first().map(|test| ReplayPair {
                trigger: test.input.clone(),
                response: test.expected_output.clone(),
            })
        })
    }
}

pub(super) fn parse_structured_skill(
    description: &str,
) -> Result<Option<StructuredSkillSpec>, SkillCompileError> {
    let mut spec = StructuredSkillSpec::default();
    let mut saw_structured_line = false;
    let mut required_capabilities = BTreeSet::new();

    for line in description
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if line_after_label(line, "Skill").is_some() {
            saw_structured_line = true;
            spec.name = required_code_span(line, "Skill")?;
        } else if line_after_label(line, "Input").is_some() {
            saw_structured_line = true;
            spec.inputs.push(parse_input_line(line)?);
        } else if line_after_label(line, "Precondition").is_some() {
            saw_structured_line = true;
            spec.preconditions
                .push(required_labeled_text(line, "Precondition")?);
        } else if line_after_label(line, "Step").is_some() {
            saw_structured_line = true;
            spec.steps.push(required_labeled_text(line, "Step")?);
        } else if line_after_label(line, "Effect").is_some() {
            saw_structured_line = true;
            spec.effects.push(required_labeled_text(line, "Effect")?);
        } else if line_after_label(line, "Expected test").is_some() {
            saw_structured_line = true;
            spec.expected_tests.push(parse_expected_test_line(line)?);
        } else if line_after_label(line, "Target").is_some() {
            saw_structured_line = true;
            spec.targets.push(parse_target_line(line)?);
        } else if line_after_label(line, "Tool").is_some() {
            saw_structured_line = true;
            required_capabilities.insert(tool_capability(&required_code_span(line, "Tool")?));
        } else if line_after_label(line, "Permission").is_some() {
            saw_structured_line = true;
            spec.permissions.push(parse_permission_line(line)?);
        } else if saw_structured_line && looks_like_skill_description(line) {
            saw_structured_line = true;
            if let Some((trigger, response)) = extract_trigger_response(line) {
                spec.trigger_response = Some(ReplayPair { trigger, response });
            }
        } else if saw_structured_line || looks_like_structured_skill_line(line) {
            return Err(SkillCompileError::UnsupportedInstruction {
                reason: format!("unsupported structured skill line `{line}`"),
            });
        }
    }

    if !saw_structured_line {
        return Ok(None);
    }
    if spec.name.is_empty() {
        return Err(SkillCompileError::UnsupportedInstruction {
            reason: String::from("structured skills must start with Skill `name`"),
        });
    }
    if spec.trigger_response.is_none() && spec.expected_tests.is_empty() {
        return Err(SkillCompileError::UnsupportedShape);
    }

    for instruction in spec
        .preconditions
        .iter()
        .chain(spec.steps.iter())
        .chain(spec.effects.iter())
    {
        if let Some(reason) = unsupported_instruction_reason(instruction) {
            return Err(SkillCompileError::UnsupportedInstruction { reason });
        }
        if let Some(capability) = inferred_permissioned_capability(instruction) {
            required_capabilities.insert(capability);
        }
    }

    let granted_capabilities = spec
        .permissions
        .iter()
        .map(|permission| permission.capability.as_str())
        .collect::<Vec<_>>();
    for capability in required_capabilities {
        if !capability_is_granted(&capability, &granted_capabilities) {
            return Err(SkillCompileError::PermissionRequired { capability });
        }
    }
    deduplicate_permissions(&mut spec.permissions);
    spec.targets.sort();
    spec.targets.dedup();

    Ok(Some(spec))
}

fn parse_input_line(line: &str) -> Result<StructuredInput, SkillCompileError> {
    let name = required_code_span(line, "Input")?;
    let Some((_, value_type)) = line.split_once(':') else {
        return Err(SkillCompileError::UnsupportedInstruction {
            reason: format!("typed input `{name}` must declare a type"),
        });
    };
    let value_type = value_type.trim();
    if !supported_input_type(value_type) {
        return Err(SkillCompileError::UnsupportedInstruction {
            reason: format!("unsupported input type `{value_type}`"),
        });
    }
    Ok(StructuredInput {
        name,
        value_type: value_type.to_owned(),
    })
}

fn parse_expected_test_line(line: &str) -> Result<StructuredExpectedTest, SkillCompileError> {
    let spans = code_spans(line);
    if spans.len() >= 2 {
        return Ok(StructuredExpectedTest {
            input: spans[0].clone(),
            expected_output: spans[1].clone(),
        });
    }
    let rest = line_after_label(line, "Expected test").unwrap_or_default();
    if let Some((input, expected_output)) = rest.split_once("->") {
        let input = trim_code_or_text(input);
        let expected_output = trim_code_or_text(expected_output);
        if !input.is_empty() && !expected_output.is_empty() {
            return Ok(StructuredExpectedTest {
                input,
                expected_output,
            });
        }
    }
    Err(SkillCompileError::UnsupportedInstruction {
        reason: String::from("expected tests must use `input` -> `output`"),
    })
}

fn parse_target_line(line: &str) -> Result<String, SkillCompileError> {
    let target = required_code_span(line, "Target")?.to_ascii_lowercase();
    match target.as_str() {
        "rust" | "native" => Ok(target),
        "javascript" | "js" => Ok(String::from("javascript")),
        _ => Err(SkillCompileError::UnsupportedInstruction {
            reason: format!("unsupported handler target `{target}`"),
        }),
    }
}

fn parse_permission_line(line: &str) -> Result<StructuredPermission, SkillCompileError> {
    let capability = required_code_span(line, "Permission")?;
    if !capability.contains(':') {
        return Err(SkillCompileError::UnsupportedInstruction {
            reason: format!("permission `{capability}` must include a capability namespace"),
        });
    }
    let description =
        line.rsplit_once('`')
            .map_or("explicit package permission grant", |(_, rest)| {
                rest.trim()
                    .strip_prefix(':')
                    .map_or("explicit package permission grant", str::trim)
            });
    Ok(StructuredPermission {
        capability,
        description: if description.is_empty() {
            String::from("explicit package permission grant")
        } else {
            description.to_owned()
        },
    })
}

fn line_after_label<'a>(line: &'a str, label: &str) -> Option<&'a str> {
    let trimmed = line.trim_start();
    let rest = trimmed.get(label.len()..)?;
    if !trimmed[..label.len()].eq_ignore_ascii_case(label) {
        return None;
    }
    if !rest.is_empty()
        && !rest.starts_with(char::is_whitespace)
        && !rest.starts_with(':')
        && !rest.starts_with('`')
    {
        return None;
    }
    Some(rest.trim_start().strip_prefix(':').unwrap_or(rest).trim())
}

fn required_code_span(line: &str, label: &str) -> Result<String, SkillCompileError> {
    code_spans(line)
        .into_iter()
        .next()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SkillCompileError::UnsupportedInstruction {
            reason: format!("{label} must use a backtick-quoted value"),
        })
}

fn required_labeled_text(line: &str, label: &str) -> Result<String, SkillCompileError> {
    if let Some(value) = code_spans(line).into_iter().next() {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    let value = line_after_label(line, label)
        .map(trim_code_or_text)
        .unwrap_or_default();
    if value.is_empty() {
        Err(SkillCompileError::UnsupportedInstruction {
            reason: format!("{label} must include text"),
        })
    } else {
        Ok(value)
    }
}

fn trim_code_or_text(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim()
        .to_owned()
}

fn supported_input_type(value_type: &str) -> bool {
    let lower = value_type.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "text"
            | "string"
            | "integer"
            | "number"
            | "decimal"
            | "boolean"
            | "url"
            | "path"
            | "semver"
            | "date"
    ) || (lower.starts_with("enum(") && lower.ends_with(')') && lower.len() > "enum()".len())
}

fn looks_like_structured_skill_line(line: &str) -> bool {
    [
        "Skill",
        "Input",
        "Precondition",
        "Step",
        "Effect",
        "Expected test",
        "Target",
        "Tool",
        "Permission",
    ]
    .iter()
    .any(|label| line_after_label(line, label).is_some())
}

/// Reject an instruction whose surface marks it as nondeterministic.
///
/// The cues are not named here: they are the surfaces of the
/// [`nondeterministic_marker`](seed::ROLE_NONDETERMINISTIC_MARKER) meaning,
/// matched as raw substrings through
/// [`Lexicon::mentions_role_raw`](seed::Lexicon::mentions_role_raw). The
/// instruction is lower-cased with [`str::to_lowercase`] (full Unicode folding,
/// so non-ASCII surfaces fold too) before matching against the lower-cased
/// stored surfaces. A structured skill must be deterministic and reviewable, so
/// any such cue is a hard error.
fn unsupported_instruction_reason(instruction: &str) -> Option<String> {
    let lower = instruction.to_lowercase();
    if seed::lexicon().mentions_role_raw(seed::ROLE_NONDETERMINISTIC_MARKER, &lower) {
        return Some(String::from(
            "structured skills must be deterministic and reviewable",
        ));
    }
    None
}

/// Infer the package capability an instruction silently requires from its surface.
///
/// The cues are read from the lexicon by meaning, never named here. A
/// [`shell_capability_need`](seed::ROLE_SHELL_CAPABILITY_CUE) surface implies
/// `tool:local_shell`; a [`network_capability_need`](seed::ROLE_NETWORK_CAPABILITY_CUE)
/// surface implies `tool:web_fetch`. The roles are checked in that order — shell
/// before network — so an instruction that mentions both resolves to the shell
/// grant, preserving the original precedence. Each role is matched as a raw
/// substring through [`Lexicon::mentions_role_raw`](seed::Lexicon::mentions_role_raw)
/// against the [`str::to_lowercase`]-folded instruction.
fn inferred_permissioned_capability(instruction: &str) -> Option<String> {
    let lower = instruction.to_lowercase();
    let lexicon = seed::lexicon();
    for (role, capability) in [
        (seed::ROLE_SHELL_CAPABILITY_CUE, "tool:local_shell"),
        (seed::ROLE_NETWORK_CAPABILITY_CUE, "tool:web_fetch"),
    ] {
        if lexicon.mentions_role_raw(role, &lower) {
            return Some(String::from(capability));
        }
    }
    None
}

fn tool_capability(tool: &str) -> String {
    if tool.contains(':') {
        tool.to_owned()
    } else {
        format!("tool:{tool}")
    }
}

fn capability_is_granted(capability: &str, granted_capabilities: &[&str]) -> bool {
    granted_capabilities
        .iter()
        .any(|granted| *granted == capability || *granted == "tool:*")
}

fn deduplicate_permissions(permissions: &mut Vec<StructuredPermission>) {
    let mut seen = BTreeSet::new();
    permissions.retain(|permission| seen.insert(permission.capability.clone()));
}

pub(super) fn structured_canonical(
    source_description: &str,
    spec: &StructuredSkillSpec,
    primary: &ReplayPair,
) -> String {
    let mut canonical = String::from("structured_skill\n");
    canonical.push_str(&spec.name);
    canonical.push('\n');
    for input in &spec.inputs {
        canonical.push_str(&input.name);
        canonical.push(':');
        canonical.push_str(&input.value_type);
        canonical.push('\n');
    }
    for step in &spec.steps {
        canonical.push_str("step:");
        canonical.push_str(step);
        canonical.push('\n');
    }
    canonical.push_str("trigger:");
    canonical.push_str(&normalize_prompt(&primary.trigger));
    canonical.push('\n');
    canonical.push_str("response:");
    canonical.push_str(&primary.response);
    canonical.push('\n');
    canonical.push_str(source_description.trim());
    canonical
}

pub(super) fn handler_signature(skill_name: &str, inputs: &[CompiledSkillInput]) -> String {
    let args = inputs
        .iter()
        .map(|input| format!("{}: {}", sanitize_identifier(&input.name), input.value_type))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{}({args}) -> deterministic_response",
        sanitize_identifier(skill_name)
    )
}

pub(super) fn handler_stub_source(
    target: &str,
    skill_name: &str,
    inputs: &[CompiledSkillInput],
    steps: &[CompiledSkillStep],
    primary: &ReplayPair,
) -> String {
    let name = sanitize_identifier(skill_name);
    match target {
        "rust" => {
            let args = inputs
                .iter()
                .map(|input| {
                    format!(
                        "{}: {}",
                        sanitize_identifier(&input.name),
                        rust_type_for(&input.value_type)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            let mut source = format!("pub fn {name}({args}) -> &'static str {{\n");
            for step in steps {
                let _ = writeln!(source, "    // step {}: {}", step.order, step.description);
            }
            let _ = writeln!(source, "    {:?}", primary.response);
            source.push_str("}\n");
            source
        }
        "javascript" => {
            let mut source = format!("export function {name}(input) {{\n");
            for step in steps {
                let _ = writeln!(source, "  // step {}: {}", step.order, step.description);
            }
            let _ = writeln!(source, "  return {:?};", primary.response);
            source.push_str("}\n");
            source
        }
        "native" => {
            let mut source = format!("native_handler {name}\n");
            for step in steps {
                let _ = writeln!(source, "  step {} {}", step.order, step.description);
            }
            let _ = writeln!(source, "  returns {:?}", primary.response);
            source
        }
        _ => String::new(),
    }
}

fn rust_type_for(value_type: &str) -> &'static str {
    let lower = value_type.to_ascii_lowercase();
    match lower.as_str() {
        "integer" => "i64",
        "number" | "decimal" => "f64",
        "boolean" => "bool",
        _ => "&str",
    }
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = String::new();
    let mut last_underscore = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            out.push(character.to_ascii_lowercase());
            last_underscore = false;
        } else if !last_underscore {
            out.push('_');
            last_underscore = true;
        }
    }
    let trimmed = out.trim_matches('_');
    let mut identifier = if trimmed.is_empty() {
        String::from("compiled_skill")
    } else {
        trimmed.to_owned()
    };
    if identifier
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_digit)
    {
        identifier.insert_str(0, "skill_");
    }
    identifier
}
