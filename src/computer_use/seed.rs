use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde_json::Value;

use super::{ComputerPlanStep, ComputerUsePlan, ComputerUsePrimitive};

const TASKS_LINO: &str = crate::seed::COMPUTER_USE_TASKS_LINO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkTask {
    pub id: String,
    pub prompts: BTreeMap<String, String>,
    pub steps: Vec<ComputerPlanStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityGap {
    pub capability: String,
    pub locale: String,
    pub response: String,
}

#[derive(Default)]
struct ParsedSeed {
    tasks: Vec<BenchmarkTask>,
    completions: BTreeMap<String, String>,
    missing: BTreeMap<String, String>,
    verification_failures: BTreeMap<String, String>,
    preconditions: BTreeMap<String, String>,
    postconditions: BTreeMap<String, String>,
    gap_cues: Vec<(String, String, String)>,
    gap_responses: BTreeMap<(String, String), String>,
}

#[must_use]
pub fn benchmark_tasks() -> &'static [BenchmarkTask] {
    &parsed().tasks
}

#[must_use]
pub fn plan_for_prompt(prompt: &str) -> Option<ComputerUsePlan> {
    let normalized = normalize(prompt);
    for task in benchmark_tasks() {
        for (locale, candidate) in &task.prompts {
            if normalize(candidate) == normalized {
                return Some(ComputerUsePlan {
                    id: task.id.clone(),
                    locale: locale.clone(),
                    prompt: candidate.clone(),
                    steps: task.steps.clone(),
                });
            }
        }
    }
    None
}

#[must_use]
pub fn capability_gap_for_prompt(prompt: &str) -> Option<CapabilityGap> {
    let normalized = normalize(prompt);
    let seed = parsed();
    let (capability, locale, _) = seed
        .gap_cues
        .iter()
        .find(|(_, _, cue)| normalize(cue) == normalized)?;
    let response = seed
        .gap_responses
        .get(&(capability.clone(), locale.clone()))?
        .clone();
    Some(CapabilityGap {
        capability: capability.clone(),
        locale: locale.clone(),
        response,
    })
}

#[must_use]
pub fn completion_message(locale: &str, plan_id: &str, steps: usize) -> String {
    let steps = steps.to_string();
    render_template(
        localized_template(&parsed().completions, locale),
        &[("plan_id", plan_id), ("steps", &steps)],
    )
}

#[must_use]
pub fn missing_primitive_message(
    locale: &str,
    plan_id: &str,
    step_id: &str,
    primitive: ComputerUsePrimitive,
) -> String {
    render_template(
        localized_template(&parsed().missing, locale),
        &[
            ("plan_id", plan_id),
            ("step_id", step_id),
            ("primitive", primitive.name()),
        ],
    )
}

#[must_use]
pub fn verification_failed_message(
    locale: &str,
    plan_id: &str,
    step_id: &str,
    primitive: &str,
) -> String {
    render_template(
        localized_template(&parsed().verification_failures, locale),
        &[
            ("plan_id", plan_id),
            ("step_id", step_id),
            ("primitive", primitive),
        ],
    )
}

#[must_use]
pub fn tool_description(primitive: ComputerUsePrimitive) -> String {
    let lines = crate::seed::TOOLS_LINO.lines().collect::<Vec<_>>();
    let name = format!("    name {}", primitive.name());
    lines
        .iter()
        .position(|line| *line == name)
        .and_then(|index| lines.get(index + 1))
        .and_then(|line| line.trim().strip_prefix("note "))
        .and_then(|quoted| serde_json::from_str::<String>(quoted).ok())
        .unwrap_or_else(|| primitive.name().to_owned())
}

fn parsed() -> &'static ParsedSeed {
    static PARSED: OnceLock<ParsedSeed> = OnceLock::new();
    PARSED.get_or_init(parse_seed)
}

fn parse_seed() -> ParsedSeed {
    let mut parsed = ParsedSeed::default();
    let mut current_task: Option<BenchmarkTask> = None;
    let mut current_gap: Option<String> = None;

    for line in TASKS_LINO.lines() {
        if let Some(rest) = line.strip_prefix("  completion ") {
            if let Some((locale, value)) = parse_localized(rest) {
                parsed.completions.insert(locale, value);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("  missing ") {
            if let Some((locale, value)) = parse_localized(rest) {
                parsed.missing.insert(locale, value);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("  verification_failed ") {
            if let Some((locale, value)) = parse_localized(rest) {
                parsed.verification_failures.insert(locale, value);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("  precondition ") {
            if let Some((primitive, value)) = parse_primitive_template(rest) {
                parsed.preconditions.insert(primitive, value);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("  postcondition ") {
            if let Some((primitive, value)) = parse_primitive_template(rest) {
                parsed.postconditions.insert(primitive, value);
            }
            continue;
        }
        if let Some(id) = line.strip_prefix("  task ") {
            if let Some(task) = current_task.take() {
                parsed.tasks.push(task);
            }
            current_gap = None;
            current_task = Some(BenchmarkTask {
                id: id.trim().to_owned(),
                prompts: BTreeMap::new(),
                steps: Vec::new(),
            });
            continue;
        }
        if let Some(capability) = line.strip_prefix("  capability_gap ") {
            if let Some(task) = current_task.take() {
                parsed.tasks.push(task);
            }
            current_gap = Some(capability.trim().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("    prompt ") {
            let Some(task) = current_task.as_mut() else {
                continue;
            };
            if let Some((locale, value)) = parse_localized(rest) {
                task.prompts.insert(locale, value);
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("    step ") {
            let Some(task) = current_task.as_mut() else {
                continue;
            };
            if let Some((name, arguments)) = rest.trim().split_once(' ') {
                if let (Some(primitive), Some(arguments)) = (
                    ComputerUsePrimitive::from_tool_name(name),
                    parse_step_arguments(arguments),
                ) {
                    let number = task.steps.len() + 1;
                    task.steps.push(ComputerPlanStep {
                        id: format!("{}-{number:02}", task.id),
                        primitive,
                        arguments,
                        precondition: render_template(
                            primitive_template(&parsed.preconditions, primitive),
                            &[("permission", &primitive.permission_key())],
                        ),
                        postcondition: primitive_template(&parsed.postconditions, primitive)
                            .to_owned(),
                    });
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("    cue ") {
            if let (Some(capability), Some((locale, cue))) =
                (current_gap.as_ref(), parse_localized(rest))
            {
                parsed.gap_cues.push((capability.clone(), locale, cue));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("    response ") {
            if let (Some(capability), Some((locale, response))) =
                (current_gap.as_ref(), parse_localized(rest))
            {
                parsed
                    .gap_responses
                    .insert((capability.clone(), locale), response);
            }
        }
    }
    if let Some(task) = current_task {
        parsed.tasks.push(task);
    }
    parsed
}

fn localized_template<'a>(templates: &'a BTreeMap<String, String>, locale: &str) -> &'a str {
    templates
        .get(locale)
        .or_else(|| templates.get("en"))
        .map_or("", String::as_str)
}

fn parse_localized(rest: &str) -> Option<(String, String)> {
    let (locale, quoted) = rest.trim().split_once(' ')?;
    let value = serde_json::from_str::<String>(quoted).ok()?;
    Some((locale.to_owned(), value))
}

fn parse_primitive_template(rest: &str) -> Option<(String, String)> {
    let (primitive, quoted) = rest.trim().split_once(' ')?;
    ComputerUsePrimitive::from_tool_name(primitive)?;
    let value = serde_json::from_str::<String>(quoted).ok()?;
    Some((primitive.to_owned(), value))
}

fn parse_step_arguments(encoded: &str) -> Option<Value> {
    let encoded = encoded.trim();
    let json = encoded
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(encoded);
    serde_json::from_str(json).ok()
}

fn primitive_template(
    templates: &BTreeMap<String, String>,
    primitive: ComputerUsePrimitive,
) -> &str {
    templates
        .get(primitive.name())
        .map_or_else(|| primitive.name(), String::as_str)
}

fn render_template(template: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = template.to_owned();
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}

fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
