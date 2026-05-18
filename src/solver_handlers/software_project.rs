//! Generic software-project request handler.
//!
//! The handler covers open-ended "build/write/create an extension/app/bot/tool"
//! prompts. It deliberately produces a bounded, reviewable implementation plan
//! and pure-domain starter code where the request is concrete enough, instead
//! of falling through to the unknown fallback or pretending to run a full
//! project build inside chat mode.

use std::fmt::Write as _;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::solver_handlers::finalize_simple;

const ACTION_WORDS: &[&str] = &[
    "write",
    "build",
    "create",
    "implement",
    "make",
    "develop",
    "generate",
    "design",
    "scaffold",
];

const ARTIFACTS: &[(&str, &str)] = &[
    ("browser extension", "browser extension"),
    ("command line tool", "command-line tool"),
    ("cli tool", "command-line tool"),
    ("web app", "web app"),
    ("mobile app", "mobile app"),
    ("extension", "extension"),
    ("plugin", "plugin"),
    ("add-on", "extension"),
    ("addon", "extension"),
    ("bot", "bot"),
    ("application", "application"),
    ("app", "app"),
    ("service", "service"),
    ("api", "API"),
    ("website", "website"),
    ("tool", "tool"),
    ("mod", "mod"),
];

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
];

pub fn try_software_project_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let artifact = detect_artifact(normalized)?;
    if !contains_any(normalized, ACTION_WORDS) {
        return None;
    }
    if normalized.contains("hello") && normalized.contains("world") {
        return None;
    }

    let target = extract_target(prompt, artifact);
    let features = extract_features(prompt);
    let game_tracker = is_game_unit_tracker(normalized);

    log.append("software_project:artifact", artifact.to_owned());
    log.append("software_project:target", target.clone());
    log.append(
        "software_project:strategy",
        if game_tracker {
            "game_unit_tracker"
        } else {
            "bounded_project_plan"
        }
        .to_owned(),
    );
    for feature in &features {
        log.append("requirement", feature.clone());
    }
    if game_tracker {
        log.append("domain_model", "unit_state".to_owned());
        log.append("validation", "damage_mitigation_floor_at_zero".to_owned());
        log.append(
            "validation",
            "cooldowns_decrement_without_negative_rounds".to_owned(),
        );
    }

    let body = render_plan(artifact, &target, &features, game_tracker);
    Some(finalize_simple(
        prompt,
        log,
        "software_project_plan",
        "response:software_project_plan",
        &body,
        0.75,
    ))
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn detect_artifact(normalized: &str) -> Option<&'static str> {
    ARTIFACTS
        .iter()
        .find(|(needle, _)| normalized.contains(*needle))
        .map(|(_, artifact)| *artifact)
}

fn extract_target(prompt: &str, artifact: &str) -> String {
    for marker in [
        format!("{artifact} for "),
        format!("{artifact} to "),
        String::from(" for "),
        String::from(" to "),
    ] {
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

fn extract_features(prompt: &str) -> Vec<String> {
    let mut features = Vec::new();
    for segment in prompt.split(['.', ';', '\n']) {
        for clause in segment.split(',') {
            let cleaned = clause.trim();
            if cleaned.is_empty() {
                continue;
            }
            let lower = cleaned.to_lowercase();
            if contains_any(&lower, FEATURE_MARKERS) {
                push_unique(&mut features, sentence_case(cleaned));
            }
        }
    }
    if features.is_empty() {
        features.push(String::from(
            "Capture state, user commands, persistence, validation, and tests.",
        ));
    }
    features
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

fn render_plan(artifact: &str, target: &str, features: &[String], game_tracker: bool) -> String {
    let mut body = String::new();
    let _ = writeln!(
        body,
        "Implementation plan for a {artifact} targeting {target}:"
    );
    body.push('\n');
    body.push_str("Requirements extracted:\n");
    for feature in features {
        let _ = writeln!(body, "- {feature}");
    }
    body.push('\n');
    body.push_str(
        "Architecture:\n\
         - Domain state: keep the smallest serializable records for each tracked object.\n\
         - Commands: add, edit, apply update, advance time, export/import state.\n\
         - Persistence: save state through the host extension storage boundary.\n\
         - Tests: cover state transitions before wiring UI or host APIs.\n",
    );

    if game_tracker {
        body.push_str("\nStarter TypeScript core:\n\n");
        body.push_str("```typescript\n");
        body.push_str(GAME_TRACKER_TYPESCRIPT);
        body.push_str("\n```\n\n");
        body.push_str(
            "Owlbear integration steps:\n\
             1. Map each selected token/item to a `UnitState` record in extension storage.\n\
             2. Render a compact panel with HP, Protection, Resistance, and cooldown controls.\n\
             3. Route damage buttons through `mitigateDamage` so stacks reduce damage before HP.\n\
             4. Call `tickCooldowns` at round end and persist the returned state.\n\
             5. Add tests for zero damage, overkill damage, stack edits, and cooldown expiry.",
        );
    } else {
        body.push_str(
            "\nNext implementation steps:\n\
             1. Define the state schema and one pure update function per user action.\n\
             2. Write unit tests for those update functions.\n\
             3. Add the host-specific adapter only after the core state transitions pass.\n\
             4. Add an export path so users can inspect and back up their data.",
        );
    }

    body
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
