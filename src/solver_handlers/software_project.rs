//! Generic software-project request handler.
//!
//! The handler covers open-ended "build/write/create an extension/app/bot/tool"
//! prompts. It first projects the surface text into a small Links Notation
//! meaning record, then derives reasoning and plan steps from that meaning.
//! Code is returned only after the user approves the plan in a later turn.

use std::fmt::Write as _;

use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult, Parser};

use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::{last_assistant_turn, last_user_turn};

const FEATURE_MARKERS: &[&str] = &[
    "track",
    "tracking",
    "reduce",
    "damage",
    "cooldown",
    "hp",
    "protection",
    "resistance",
    "stack",
    "status",
    "effect",
    "export",
    "csv",
    "reminder",
    "schedule",
    "invoice",
    "payment",
    "rename",
    "date",
    "report",
    "notification",
    "import",
    "backup",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArtifactMatch {
    surface: &'static str,
    label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SoftwareProjectMeaning {
    action: &'static str,
    artifact_surface: &'static str,
    artifact: &'static str,
    target: String,
    requirements: Vec<String>,
    game_tracker: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalState {
    Proposed,
    Approved,
}

impl ApprovalState {
    const fn label(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
        }
    }
}

impl SoftwareProjectMeaning {
    fn from_prompt(prompt: &str, normalized: &str) -> Option<Self> {
        if normalized.contains("hello") && normalized.contains("world") {
            return None;
        }

        let action = scan_parse(normalized, parse_action_word)?;
        let artifact = scan_parse(normalized, parse_artifact_phrase)?;
        let target = extract_target(prompt, artifact);
        let requirements = extract_requirements(prompt);
        let game_tracker = is_game_unit_tracker(normalized);

        Some(Self {
            action,
            artifact_surface: artifact.surface,
            artifact: artifact.label,
            target,
            requirements,
            game_tracker,
        })
    }

    fn canonical_key(&self) -> String {
        let mut key = format!(
            "action={};artifact={};target={};game_tracker={}",
            self.action, self.artifact, self.target, self.game_tracker
        );
        for requirement in &self.requirements {
            let _ = write!(key, ";requirement={requirement}");
        }
        key
    }

    fn meaning_id(&self) -> String {
        stable_id("software_project_request", &self.canonical_key())
    }

    const fn domain_label(&self) -> &'static str {
        if self.game_tracker {
            "tabletop_game_unit_tracker"
        } else {
            "software_project"
        }
    }

    fn meaning_lino(&self, approval_state: ApprovalState) -> String {
        let mut buffer = String::from("software_project_request");
        let _ = writeln!(buffer, "\n  action {}", lino_string(self.action));
        let _ = writeln!(buffer, "  artifact {}", lino_string(self.artifact));
        let _ = writeln!(
            buffer,
            "  artifact_surface {}",
            lino_string(self.artifact_surface)
        );
        let _ = writeln!(buffer, "  target {}", lino_string(&self.target));
        let _ = writeln!(buffer, "  domain {}", lino_string(self.domain_label()));
        let _ = writeln!(buffer, "  approval_state {}", approval_state.label());
        let _ = writeln!(buffer, "  approval_required true");
        for requirement in &self.requirements {
            let _ = writeln!(buffer, "  requirement {}", lino_string(requirement));
        }
        if self.game_tracker {
            buffer.push_str("  state_model \"unit_state\"\n");
            buffer.push_str("  command \"apply_damage\"\n");
            buffer.push_str("  command \"set_stacks\"\n");
            buffer.push_str("  command \"tick_cooldowns\"\n");
            buffer.push_str("  validation \"damage_mitigation_floor_at_zero\"\n");
            buffer.push_str("  validation \"cooldowns_decrement_without_negative_rounds\"\n");
        } else {
            buffer.push_str("  state_model \"project_records\"\n");
            buffer.push_str("  command \"create_record\"\n");
            buffer.push_str("  command \"update_record\"\n");
            buffer.push_str("  command \"export_state\"\n");
            buffer.push_str("  validation \"pure_state_transitions_before_host_api\"\n");
        }
        buffer
    }
}

pub fn try_software_project_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if is_approval_prompt(normalized) {
        if let Some(meaning) = prior_software_project_meaning(log) {
            record_meaning(log, &meaning, ApprovalState::Approved);
            let body = render_implementation_response(&meaning);
            return Some(finalize_simple(
                prompt,
                log,
                "software_project_implementation",
                "response:software_project_implementation",
                &body,
                0.82,
            ));
        }
    }

    let meaning = SoftwareProjectMeaning::from_prompt(prompt, normalized)?;
    record_meaning(log, &meaning, ApprovalState::Proposed);
    let body = render_plan_response(&meaning);
    Some(finalize_simple(
        prompt,
        log,
        "software_project_plan",
        "response:software_project_plan",
        &body,
        0.78,
    ))
}

fn prior_software_project_meaning(log: &EventLog) -> Option<SoftwareProjectMeaning> {
    let assistant = last_assistant_turn(log)?;
    if !assistant.contains("software_project_request") || !assistant.contains("approve plan") {
        return None;
    }
    let prior_prompt = last_user_turn(log)?;
    SoftwareProjectMeaning::from_prompt(prior_prompt, &prior_prompt.to_lowercase())
}

fn record_meaning(
    log: &mut EventLog,
    meaning: &SoftwareProjectMeaning,
    approval_state: ApprovalState,
) {
    log.append("formalization", "text_to_links_notation".to_owned());
    log.append("meaning", meaning.meaning_id());
    log.append("software_project:action", meaning.action.to_owned());
    log.append("software_project:artifact", meaning.artifact.to_owned());
    log.append("software_project:target", meaning.target.clone());
    log.append("software_project:domain", meaning.domain_label().to_owned());
    log.append("approval_state", approval_state.label().to_owned());
    log.append(
        "software_project:strategy",
        if meaning.game_tracker {
            "game_unit_tracker"
        } else {
            "bounded_project_plan"
        }
        .to_owned(),
    );
    for requirement in &meaning.requirements {
        log.append("requirement", requirement.clone());
    }
    for step in reasoning_steps(meaning) {
        log.append("reasoning_step", step);
    }
    for step in plan_steps(meaning) {
        log.append("plan_step", step);
    }
}

fn parse_action_word(input: &str) -> IResult<&str, &'static str> {
    alt((
        value("write", tag("write")),
        value("build", tag("build")),
        value("create", tag("create")),
        value("implement", tag("implement")),
        value("make", tag("make")),
        value("develop", tag("develop")),
        value("generate", tag("generate")),
        value("design", tag("design")),
        value("scaffold", tag("scaffold")),
    ))
    .parse(input)
}

fn parse_artifact_phrase(input: &str) -> IResult<&str, ArtifactMatch> {
    alt((
        value(
            ArtifactMatch {
                surface: "browser extension",
                label: "browser extension",
            },
            tag("browser extension"),
        ),
        value(
            ArtifactMatch {
                surface: "command line tool",
                label: "command-line tool",
            },
            tag("command line tool"),
        ),
        value(
            ArtifactMatch {
                surface: "cli tool",
                label: "command-line tool",
            },
            tag("cli tool"),
        ),
        value(
            ArtifactMatch {
                surface: "web app",
                label: "web app",
            },
            tag("web app"),
        ),
        value(
            ArtifactMatch {
                surface: "mobile app",
                label: "mobile app",
            },
            tag("mobile app"),
        ),
        value(
            ArtifactMatch {
                surface: "extension",
                label: "extension",
            },
            tag("extension"),
        ),
        value(
            ArtifactMatch {
                surface: "plugin",
                label: "plugin",
            },
            tag("plugin"),
        ),
        value(
            ArtifactMatch {
                surface: "add-on",
                label: "extension",
            },
            tag("add-on"),
        ),
        value(
            ArtifactMatch {
                surface: "addon",
                label: "extension",
            },
            tag("addon"),
        ),
        value(
            ArtifactMatch {
                surface: "bot",
                label: "bot",
            },
            tag("bot"),
        ),
        value(
            ArtifactMatch {
                surface: "application",
                label: "application",
            },
            tag("application"),
        ),
        value(
            ArtifactMatch {
                surface: "app",
                label: "app",
            },
            tag("app"),
        ),
        value(
            ArtifactMatch {
                surface: "service",
                label: "service",
            },
            tag("service"),
        ),
        value(
            ArtifactMatch {
                surface: "api",
                label: "API",
            },
            tag("api"),
        ),
        value(
            ArtifactMatch {
                surface: "website",
                label: "website",
            },
            tag("website"),
        ),
        value(
            ArtifactMatch {
                surface: "tool",
                label: "tool",
            },
            tag("tool"),
        ),
        value(
            ArtifactMatch {
                surface: "mod",
                label: "mod",
            },
            tag("mod"),
        ),
    ))
    .parse(input)
}

fn scan_parse<T: Copy>(normalized: &str, parser: fn(&str) -> IResult<&str, T>) -> Option<T> {
    for (index, _) in normalized.char_indices() {
        if !is_start_boundary(normalized, index) {
            continue;
        }
        let input = &normalized[index..];
        if let Ok((remaining, value)) = parser(input) {
            let consumed = input.len().saturating_sub(remaining.len());
            if consumed > 0 && is_end_boundary(normalized, index + consumed) {
                return Some(value);
            }
        }
    }
    None
}

fn is_start_boundary(value: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    value[..index]
        .chars()
        .next_back()
        .is_some_and(|character| !is_word_character(character))
}

fn is_end_boundary(value: &str, index: usize) -> bool {
    if index >= value.len() {
        return true;
    }
    value[index..]
        .chars()
        .next()
        .is_some_and(|character| !is_word_character(character))
}

const fn is_word_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
}

fn extract_target(prompt: &str, artifact: ArtifactMatch) -> String {
    let markers = [
        format!("{} for ", artifact.surface),
        format!("{} to ", artifact.surface),
        format!("{} for ", artifact.label),
        format!("{} to ", artifact.label),
        String::from(" for "),
        String::from(" to "),
    ];
    for marker in markers {
        if let Some(target) = extract_after_marker(prompt, &marker) {
            return target;
        }
    }
    String::from("the requested environment")
}

fn extract_after_marker(prompt: &str, marker: &str) -> Option<String> {
    let lower_prompt = prompt.to_lowercase();
    let lower_marker = marker.to_lowercase();
    let start = lower_prompt.find(&lower_marker)? + lower_marker.len();
    let tail = &prompt[start..];
    let stop = tail.find(['?', '.', ',', ';', '\n']).unwrap_or(tail.len());
    let raw = tail[..stop]
        .split(" with ")
        .next()
        .unwrap_or("")
        .split(" that ")
        .next()
        .unwrap_or("")
        .split(" and ")
        .next()
        .unwrap_or("")
        .trim();
    if raw.is_empty() {
        return None;
    }
    Some(capitalize_short_target(raw))
}

fn capitalize_short_target(raw: &str) -> String {
    let compact = raw.split_whitespace().take(5).collect::<Vec<_>>().join(" ");
    if compact.chars().any(char::is_uppercase) {
        return compact;
    }
    let mut chars = compact.chars();
    let Some(first) = chars.next() else {
        return compact;
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

fn extract_requirements(prompt: &str) -> Vec<String> {
    let mut requirements = Vec::new();
    for segment in prompt.split(['.', ';', '\n']) {
        for clause in segment.split(',') {
            let cleaned = clause.trim();
            if cleaned.is_empty() {
                continue;
            }
            let lower = cleaned.to_lowercase();
            if contains_any(&lower, FEATURE_MARKERS) {
                push_unique(&mut requirements, sentence_case(cleaned));
            }
        }
    }
    if requirements.is_empty() {
        requirements.push(String::from(
            "Capture state, user commands, persistence, validation, and tests.",
        ));
    }
    requirements
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn push_unique(items: &mut Vec<String>, value: String) {
    if !items.iter().any(|item| item == &value) {
        items.push(value);
    }
}

fn sentence_case(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches(['-', '*', ' ']);
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

fn is_game_unit_tracker(normalized: &str) -> bool {
    let domain = normalized.contains("dnd")
        || normalized.contains("d&d")
        || normalized.contains("wargame")
        || normalized.contains("tabletop")
        || normalized.contains("unit")
        || normalized.contains("token")
        || normalized.contains("owlbear");
    let mechanics = normalized.contains("hp")
        || normalized.contains("damage")
        || normalized.contains("protection")
        || normalized.contains("resistance")
        || normalized.contains("cooldown");
    domain && mechanics
}

fn is_approval_prompt(normalized: &str) -> bool {
    let compact = normalized
        .trim()
        .trim_matches(|character: char| !character.is_ascii_alphanumeric())
        .replace(['.', '!', ','], "");
    matches!(
        compact.as_str(),
        "approve"
            | "approved"
            | "approve plan"
            | "yes"
            | "yes proceed"
            | "proceed"
            | "go ahead"
            | "looks good"
            | "do it"
            | "start implementation"
            | "generate code"
            | "convert to code"
    )
}

fn reasoning_steps(meaning: &SoftwareProjectMeaning) -> Vec<String> {
    let mut steps = vec![
        format!(
            "Classify the impulse as a request to {} a {} instead of a fact lookup.",
            meaning.action, meaning.artifact
        ),
        format!(
            "Bind the target environment to {} and keep the first response reviewable.",
            meaning.target
        ),
        format!(
            "Extract {} requirement(s) into the meaning record before planning.",
            meaning.requirements.len()
        ),
    ];
    if meaning.game_tracker {
        steps.push(String::from(
            "Map HP, Protection, Resistance, damage, and cooldown phrases to a unit-state domain model.",
        ));
    }
    steps.push(String::from(
        "Ask for approval before producing code or execution steps.",
    ));
    steps
}

fn plan_steps(meaning: &SoftwareProjectMeaning) -> Vec<String> {
    if meaning.game_tracker {
        return vec![
            format!(
                "Confirm the {} storage and selected-token API boundaries.",
                meaning.target
            ),
            String::from(
                "Define `UnitState` with HP, max HP, Protection, Resistance, and cooldowns.",
            ),
            String::from(
                "Write pure transition functions for damage mitigation, stack edits, and round ticks.",
            ),
            String::from(
                "Add tests for zero damage, overkill damage, stack changes, and cooldown expiry.",
            ),
            String::from("Wire the tested core into the extension panel and host persistence."),
        ];
    }

    vec![
        format!(
            "Confirm the host API and data boundaries for {}.",
            meaning.target
        ),
        String::from("Define the smallest serializable state records for the requirements."),
        String::from("Write one pure update function per user command."),
        String::from("Add tests for each state transition before host integration."),
        String::from("Add import/export so users can inspect and back up their data."),
    ]
}

fn render_plan_response(meaning: &SoftwareProjectMeaning) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Implementation plan pending approval for a {} targeting {}.",
        meaning.artifact, meaning.target
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str(&meaning.meaning_lino(ApprovalState::Proposed));
    body.push_str("```\n\nReasoning steps:\n");
    for (index, step) in reasoning_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str("\nProposed plan:\n");
    for (index, step) in plan_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str(
        "\nReply `approve plan` to generate the starter implementation, or describe what to change.",
    );
    body
}

fn render_implementation_response(meaning: &SoftwareProjectMeaning) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Approved implementation starter for a {} targeting {}.",
        meaning.artifact, meaning.target
    );
    body.push('\n');
    body.push_str("Formalized meaning:\n```lino\n");
    body.push_str(&meaning.meaning_lino(ApprovalState::Approved));
    body.push_str("```\n\nImplementation steps:\n");
    for (index, step) in plan_steps(meaning).iter().enumerate() {
        let _ = writeln!(body, "{}. {step}", index + 1);
    }
    body.push_str("\nStarter TypeScript core:\n\n```typescript\n");
    if meaning.game_tracker {
        body.push_str(GAME_TRACKER_TYPESCRIPT);
    } else {
        body.push_str(GENERIC_PROJECT_TYPESCRIPT);
    }
    body.push_str("\n```\n");
    body
}

fn lino_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

const GAME_TRACKER_TYPESCRIPT: &str = r"type Cooldown = {
  name: string;
  remainingRounds: number;
};

type UnitState = {
  id: string;
  name: string;
  hp: number;
  maxHp: number;
  protection: number;
  resistance: number;
  cooldowns: Cooldown[];
};

type DamageResult = {
  damageTaken: number;
  prevented: number;
  unit: UnitState;
};

export function mitigateDamage(unit: UnitState, rawDamage: number): DamageResult {
  const prevented = Math.max(0, unit.protection) + Math.max(0, unit.resistance);
  const damageTaken = Math.max(0, rawDamage - prevented);
  return {
    damageTaken,
    prevented,
    unit: { ...unit, hp: Math.max(0, unit.hp - damageTaken) },
  };
}

export function setStacks(
  unit: UnitState,
  protection: number,
  resistance: number,
): UnitState {
  return {
    ...unit,
    protection: Math.max(0, protection),
    resistance: Math.max(0, resistance),
  };
}

export function tickCooldowns(unit: UnitState): UnitState {
  return {
    ...unit,
    cooldowns: unit.cooldowns
      .map((cooldown) => ({
        ...cooldown,
        remainingRounds: Math.max(0, cooldown.remainingRounds - 1),
      }))
      .filter((cooldown) => cooldown.remainingRounds > 0),
  };
}";

const GENERIC_PROJECT_TYPESCRIPT: &str = r#"type ProjectRecord = {
  id: string;
  title: string;
  status: "open" | "done";
  notes: string[];
};

type ProjectCommand =
  | { type: "add"; id: string; title: string }
  | { type: "note"; id: string; note: string }
  | { type: "complete"; id: string };

export function applyCommand(
  records: ProjectRecord[],
  command: ProjectCommand,
): ProjectRecord[] {
  switch (command.type) {
    case "add":
      return [
        ...records,
        { id: command.id, title: command.title, status: "open", notes: [] },
      ];
    case "note":
      return records.map((record) =>
        record.id === command.id
          ? { ...record, notes: [...record.notes, command.note] }
          : record,
      );
    case "complete":
      return records.map((record) =>
        record.id === command.id ? { ...record, status: "done" } : record,
      );
  }
}"#;
