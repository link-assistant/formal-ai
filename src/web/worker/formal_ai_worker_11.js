// Worker module 12 of 21. Loaded by ../formal_ai_worker.js.
const GAME_TRACKER_TYPESCRIPT = `type Cooldown = {
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
}`;

const GENERIC_PROJECT_TYPESCRIPT = `type ProjectRecord = {
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
}`;

const GENERIC_PROJECT_JAVASCRIPT = `export function applyCommand(records, command) {
  switch (command.type) {
    case "add":
      return [...records, { id: command.id, title: command.title, status: "open", notes: [] }];
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
    default:
      throw new Error("Unknown command: " + command.type);
  }
}`;

const GENERIC_PROJECT_PYTHON = `from dataclasses import dataclass, field


@dataclass(frozen=True)
class ProjectRecord:
    id: str
    title: str
    status: str = "open"
    notes: tuple[str, ...] = field(default_factory=tuple)


def apply_command(records: tuple[ProjectRecord, ...], command: dict) -> tuple[ProjectRecord, ...]:
    if command["type"] == "add":
        return (*records, ProjectRecord(id=command["id"], title=command["title"]))
    if command["type"] == "note":
        return tuple(
            ProjectRecord(r.id, r.title, r.status, (*r.notes, command["note"]))
            if r.id == command["id"] else r
            for r in records
        )
    if command["type"] == "complete":
        return tuple(
            ProjectRecord(r.id, r.title, "done", r.notes)
            if r.id == command["id"] else r
            for r in records
        )
    raise ValueError(f"Unknown command: {command['type']}")
`;

const GENERIC_PROJECT_RUST = `#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: String,
    pub title: String,
    pub status: ProjectStatus,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Open,
    Done,
}

pub enum ProjectCommand {
    Add { id: String, title: String },
    Note { id: String, note: String },
    Complete { id: String },
}

pub fn apply_command(mut records: Vec<ProjectRecord>, command: ProjectCommand) -> Vec<ProjectRecord> {
    match command {
        ProjectCommand::Add { id, title } => records.push(ProjectRecord {
            id,
            title,
            status: ProjectStatus::Open,
            notes: Vec::new(),
        }),
        ProjectCommand::Note { id, note } => {
            for record in &mut records {
                if record.id == id {
                    record.notes.push(note.clone());
                }
            }
        }
        ProjectCommand::Complete { id } => {
            for record in &mut records {
                if record.id == id {
                    record.status = ProjectStatus::Done;
                }
            }
        }
    }
    records
}`;

const PLAYWRIGHT_DOCS_URL = "https://playwright.dev/docs/writing-tests";
const PLAYWRIGHT_STARTER_TYPESCRIPT = `import { test, expect } from '@playwright/test';

test('opens the Playwright docs', async ({ page }) => {
  await page.goto('https://playwright.dev/');
  await expect(page).toHaveTitle(/Playwright/);

  await page.getByRole('link', { name: 'Docs' }).click();
  await expect(page.getByRole('heading', { name: /Playwright/ })).toBeVisible();
});`;

function containsAnySubstring(value, needles) {
  return needles.some((needle) => value.includes(needle));
}

function containsToken(normalized, token) {
  return String(normalized || "").split(/\s+/).includes(token);
}

// Issue #386 Playwright roles — mirror ROLE_PLAYWRIGHT_TOOL_NAME and
// ROLE_PLAYWRIGHT_SCRIPT_CUE in src/seed/roles.rs. The tool name (with its
// 'playright' misspelling, whose `action` names the canonical spelling) and the
// script-authoring cues live in data/seed/meanings-playwright.lino (embedded in
// MEANINGS_LINO). isPlaywrightScriptRequest matches both roles as raw
// substrings across every language, exactly like the Rust recogniser.
const ROLE_PLAYWRIGHT_TOOL_NAME = "playwright_tool_name";
const ROLE_PLAYWRIGHT_SCRIPT_CUE = "playwright_script_cue";

function isPlaywrightScriptRequest(normalized) {
  return (
    lexiconMentionsRoleSubstring(ROLE_PLAYWRIGHT_TOOL_NAME, normalized) &&
    lexiconMentionsRoleSubstring(ROLE_PLAYWRIGHT_SCRIPT_CUE, normalized)
  );
}

// True when the prompt contains a misspelled form of the Playwright name — any
// playwright_tool_name form whose `action` names the canonical spelling. The
// misspelling and its correction live in the seed data, so the handler reports
// the fix without naming either form here. Mirrors
// mentions_playwright_misspelling in src/solver_handlers/playwright_script.rs.
function mentionsPlaywrightMisspelling(normalized) {
  return roleWordForms(ROLE_PLAYWRIGHT_TOOL_NAME)
    .filter((form) => form.action)
    .some((form) => normalized.includes(form.text));
}

function renderPlaywrightClarification(language) {
  if (language === "ru") {
    return [
      "Я могу написать Playwright-скрипт. Уточните URL страницы, действия и ожидаемую проверку.",
      "Если нужен пример по умолчанию, я могу взять стартовый сценарий из документации Playwright.",
    ].join(" ");
  }
  return [
    "I can write a Playwright script. Please provide the page URL, the actions to perform, and the expected assertion.",
    "If you want a default example, I can use the starter scenario from the Playwright docs.",
  ].join(" ");
}

function renderPlaywrightStarter(language, correctedSpelling) {
  const lines = [];
  if (language === "ru" && correctedSpelling) {
    lines.push(
      "Я трактую `Playright` как `Playwright` и даю стартовый TypeScript-пример по документации Playwright.",
    );
  } else if (language === "ru") {
    lines.push("Даю стартовый TypeScript-пример по документации Playwright.");
  } else if (correctedSpelling) {
    lines.push(
      "I interpret `Playright` as `Playwright` and will use a starter TypeScript example based on the Playwright docs.",
    );
  } else {
    lines.push(
      "I will use a starter TypeScript example based on the Playwright docs.",
    );
  }
  lines.push("");
  lines.push(`Source: ${PLAYWRIGHT_DOCS_URL}`);
  lines.push("");
  lines.push("```typescript");
  lines.push(PLAYWRIGHT_STARTER_TYPESCRIPT);
  lines.push("```");
  lines.push("");
  if (language === "ru") {
    lines.push("Проверка:");
    lines.push("1. `npm init playwright@latest`");
    lines.push("2. `npx playwright test`");
    lines.push("");
    lines.push("Уточните URL, действия и ожидаемый результат, если нужен сценарий под конкретный сайт.");
  } else {
    lines.push("Check it with:");
    lines.push("1. `npm init playwright@latest`");
    lines.push("2. `npx playwright test`");
    lines.push("");
    lines.push("Provide the URL, actions, and expected result if you want a site-specific script.");
  }
  return lines.join("\n");
}

function tryPlaywrightScript(prompt, preferences = {}, language = "en") {
  const normalized = normalizePrompt(prompt);
  if (!isPlaywrightScriptRequest(normalized)) return null;
  const guessProbability = numericPreference(
    preferences && preferences.guessProbability,
    0.8,
    0,
    1,
  );
  const evidence = [
    "script_framework:playwright",
    `source:${PLAYWRIGHT_DOCS_URL}`,
    `guess_probability:${guessProbability.toFixed(2)}`,
  ];
  const correctedSpelling = mentionsPlaywrightMisspelling(normalized);
  if (correctedSpelling) {
    evidence.push("spelling_correction:Playright->Playwright");
  }
  if (guessProbability < 0.5) {
    return {
      intent: "playwright_script_clarification",
      content: renderPlaywrightClarification(language),
      confidence: 0.64,
      evidence,
    };
  }
  return {
    intent: "playwright_script",
    content: renderPlaywrightStarter(language, correctedSpelling),
    confidence: 0.82,
    evidence,
  };
}

function detectSoftwareAction(normalized) {
  const match = scanSoftwareSurface(normalized, softwareActionTable());
  return match ? match.payload : null;
}

function detectSoftwareArtifact(normalized) {
  const match = scanSoftwareSurface(normalized, softwareArtifactTable());
  return match ? { surface: match.surface, label: match.payload } : null;
}

function extractSoftwareTarget(prompt, artifact) {
  const markers = [
    `${artifact.surface} for `,
    `${artifact.surface} to `,
    `${artifact.label} for `,
    `${artifact.label} to `,
    " for ",
    " to ",
  ];
  for (const marker of markers) {
    const target = extractAfterMarker(prompt, marker);
    if (target) return target;
  }
  return "the requested environment";
}

function extractAfterMarker(prompt, marker) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  const lowerMarker = marker.toLowerCase();
  const start = lower.indexOf(lowerMarker);
  if (start < 0) return null;
  const tail = source.slice(start + lowerMarker.length);
  const stopMatch = /[?.,;\n]/.exec(tail);
  const stop = stopMatch ? stopMatch.index : tail.length;
  const raw = tail
    .slice(0, stop)
    .split(" with ")[0]
    .split(" that ")[0]
    .split(" and ")[0]
    .trim();
  if (!raw) return null;
  return capitalizeShortTarget(raw);
}

function capitalizeShortTarget(raw) {
  const compact = String(raw || "").trim().split(/\s+/).slice(0, 5).join(" ");
  if (!compact) return compact;
  if (/[A-ZА-Я]/.test(compact)) return compact;
  return compact.charAt(0).toUpperCase() + compact.slice(1);
}

function sentenceCase(raw) {
  const trimmed = String(raw || "").trim().replace(/^[-* ]+|[-* ]+$/g, "");
  if (!trimmed) return "";
  return trimmed.charAt(0).toUpperCase() + trimmed.slice(1);
}

function extractSoftwareFeatures(prompt) {
  const markers = requirementMarkerWords();
  const features = [];
  const segments = String(prompt || "").split(/[.;\n]/);
  for (const segment of segments) {
    for (const clause of segment.split(",")) {
      const cleaned = clause.trim();
      if (!cleaned) continue;
      const lower = cleaned.toLowerCase();
      if (!containsAnySubstring(lower, markers)) continue;
      const feature = sentenceCase(cleaned);
      if (feature && !features.includes(feature)) features.push(feature);
    }
  }
  if (features.length === 0) {
    features.push("Capture state, user commands, persistence, validation, and tests.");
  }
  return features;
}

// A request is a game-unit tracker only when it pairs a game domain with a
// combat mechanic — both the game_tracker_domain and game_tracker_mechanic
// roles must be evidenced. Mirrors is_game_unit_tracker in
// src/solver_handlers/software_project.rs; the decomposition lives in the
// lexicon, so the code knows only "a tracker needs both a domain and a mechanic".
function isGameUnitTracker(normalized) {
  return (
    lexiconMentionsRole(ROLE_GAME_TRACKER_DOMAIN, normalized) &&
    lexiconMentionsRole(ROLE_GAME_TRACKER_MECHANIC, normalized)
  );
}

function classifySoftwareRequirement(requirement, gameTracker) {
  const lower = String(requirement || "").toLowerCase();
  // A game unit tracker is state by construction, regardless of wording.
  if (gameTracker) {
    return "state_tracking";
  }
  // Walk the requirement-category meanings in declaration order; the first
  // whose surface word appears classifies the clause. Mirrors
  // classify_requirement in src/solver_handlers/software_project.rs. The
  // catch-all project_behavior comes last, so it acts as the default.
  for (const meaning of meaningsWithRole(ROLE_SOFTWARE_REQUIREMENT_CATEGORY)) {
    const label = SOFTWARE_REQUIREMENT_CATEGORY_LABELS[meaning.slug];
    if (!label) continue;
    if (meaning.words.some((word) => lower.includes(word.toLowerCase()))) {
      return label;
    }
  }
  return "project_behavior";
}

function softwareSubtaskTitle(category, requirement) {
  switch (category) {
    case "state_tracking":
      return `Model state fields and pure transitions for ${requirement}`;
    case "data_exchange":
      return `Define parsers, serializers, and backup flow for ${requirement}`;
    case "automation":
      return `Schedule deterministic jobs and delivery checks for ${requirement}`;
    case "validation":
      return `Encode validation rules and failure messages for ${requirement}`;
    case "integration":
      return `Isolate host API boundaries and mocks for ${requirement}`;
    case "user_interface":
      return `Design focused views and state updates for ${requirement}`;
    default:
      return `Implement and test the smallest behavior for ${requirement}`;
  }
}

function deriveSoftwareSubtasks(requirements, gameTracker) {
  return requirements.map((requirement, index) => {
    const category = classifySoftwareRequirement(requirement, gameTracker);
    return {
      requirementId: `R${index + 1}`,
      category,
      title: softwareSubtaskTitle(category, requirement),
    };
  });
}

// Pick the delivery mode by walking the software_delivery_mode meanings in
// declaration order (manual instructions → immediate execution → script
// generation — the order encodes priority) and taking the first one evidenced
// in the request; the default is generated code. Mirrors detect_delivery_mode
// in src/solver_handlers/software_project.rs — the surface words live in the
// lexicon, so the code knows only "a request can ask for a delivery mode".
function detectSoftwareDeliveryMode(normalized) {
  const meaning = firstRoleMatch(ROLE_SOFTWARE_DELIVERY_MODE, normalized);
  return (meaning && SOFTWARE_DELIVERY_MODE_LABELS[meaning.slug]) || "code_generation";
}

// Resolve the target language by walking the software_implementation_language
// meanings in declaration order (python → rust → javascript) and taking the
// first one named in the request; the default is TypeScript. Mirrors
// detect_implementation_language in src/solver_handlers/software_project.rs.
function detectSoftwareImplementationLanguage(normalized) {
  const meaning = firstRoleMatch(ROLE_SOFTWARE_IMPLEMENTATION_LANGUAGE, normalized);
  return (meaning && SOFTWARE_IMPLEMENTATION_LANGUAGE_LABELS[meaning.slug]) || "typescript";
}

// Mirrors approval_gates in src/solver_handlers/software_project.rs: the feature,
// step-granularity, and bash-command gates are added when the lexicon evidences
// the matching role, so the gate vocabulary lives once in data.
function softwareApprovalGates(normalized, deliveryMode) {
  const gates = ["task_formalization", "implementation_plan"];
  if (lexiconMentionsRole(ROLE_SOFTWARE_FEATURE, normalized)) gates.push("requirements");
  if (lexiconMentionsRole(ROLE_SOFTWARE_STEP_GRANULARITY, normalized)) gates.push("each_step");
  if (deliveryMode === "code_generation") {
    gates.push("generated_code");
  } else if (deliveryMode === "manual_instructions") {
    gates.push("manual_instructions");
  } else {
    gates.push("generated_script");
    gates.push("bash_command");
  }
  if (lexiconMentionsRole(ROLE_SOFTWARE_BASH_COMMAND, normalized)) {
    gates.push("bash_command");
  }
  return [...new Set(gates)].sort();
}

function softwareImplementationCode(meaning) {
  if (meaning.gameTracker) {
    return {
      label: "TypeScript",
      fence: "typescript",
      body: GAME_TRACKER_TYPESCRIPT,
    };
  }
  if (meaning.implementationLanguage === "python") {
    return { label: "Python", fence: "python", body: GENERIC_PROJECT_PYTHON };
  }
  if (meaning.implementationLanguage === "rust") {
    return { label: "Rust", fence: "rust", body: GENERIC_PROJECT_RUST };
  }
  if (meaning.implementationLanguage === "javascript") {
    return { label: "JavaScript", fence: "javascript", body: GENERIC_PROJECT_JAVASCRIPT };
  }
  return { label: "TypeScript", fence: "typescript", body: GENERIC_PROJECT_TYPESCRIPT };
}

function softwareDomainLabel(meaning) {
  return meaning.gameTracker ? "tabletop_game_unit_tracker" : "software_project";
}

function softwareApprovalLabel(approved) {
  return approved ? "approved" : "proposed";
}

function linoString(value) {
  return `"${String(value || "")
    .replace(/\\/g, "\\\\")
    .replace(/"/g, '\\"')
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "\\r")}"`;
}

function softwareMeaningLino(meaning, approved) {
  const lines = ["software_project_request"];
  lines.push(`  action ${linoString(meaning.action)}`);
  lines.push(`  artifact ${linoString(meaning.artifact)}`);
  lines.push(`  artifact_surface ${linoString(meaning.artifactSurface)}`);
  lines.push(`  target ${linoString(meaning.target)}`);
  lines.push(`  domain ${linoString(softwareDomainLabel(meaning))}`);
  lines.push(`  delivery_mode ${meaning.deliveryMode}`);
  lines.push(`  implementation_language ${linoString(meaning.implementationLanguage)}`);
  lines.push(`  approval_state ${softwareApprovalLabel(approved)}`);
  lines.push("  approval_required true");
  for (const gate of meaning.approvalGates) {
    lines.push(`  approval_gate ${linoString(gate)}`);
  }
  for (const requirement of meaning.requirements) {
    lines.push(`  requirement ${linoString(requirement)}`);
    lines.push(
      `  requirement_category ${linoString(
        classifySoftwareRequirement(requirement, meaning.gameTracker),
      )}`,
    );
  }
  for (const subtask of meaning.subtasks) {
    lines.push(
      `  subtask ${linoString(
        `${subtask.requirementId} [${subtask.category}] ${subtask.title}`,
      )}`,
    );
  }
  if (meaning.gameTracker) {
    lines.push('  state_model "unit_state"');
    lines.push('  command "apply_damage"');
    lines.push('  command "set_stacks"');
    lines.push('  command "tick_cooldowns"');
    lines.push('  validation "damage_mitigation_floor_at_zero"');
    lines.push('  validation "cooldowns_decrement_without_negative_rounds"');
  } else {
    lines.push('  state_model "project_records"');
    lines.push('  command "create_record"');
    lines.push('  command "update_record"');
    lines.push('  command "export_state"');
    lines.push('  validation "pure_state_transitions_before_host_api"');
  }
  return lines.join("\n") + "\n";
}

function softwareMeaningKey(meaning) {
  return [
    `action=${meaning.action}`,
    `artifact=${meaning.artifact}`,
    `target=${meaning.target}`,
    `game_tracker=${meaning.gameTracker}`,
    `delivery_mode=${meaning.deliveryMode}`,
    `implementation_language=${meaning.implementationLanguage}`,
    ...meaning.requirements.map((requirement) => `requirement=${requirement}`),
    ...meaning.subtasks.map((subtask) => `subtask=${subtask.category}:${subtask.title}`),
  ].join(";");
}

function stableSoftwareMeaningId(meaning) {
  let hash = 0xcbf29ce484222325n;
  const source = softwareMeaningKey(meaning);
  for (let index = 0; index < source.length; index += 1) {
    hash ^= BigInt(source.charCodeAt(index));
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return `software_project_request_${hash.toString(16)}`;
}

function formalizeSoftwareProjectRequest(prompt) {
  const normalized = normalizePrompt(prompt);
  if (normalized.includes("hello") && normalized.includes("world")) return null;
  const action = detectSoftwareAction(normalized);
  const artifact = detectSoftwareArtifact(normalized);
  if (!action || !artifact) return null;
  const requirements = extractSoftwareFeatures(prompt);
  const gameTracker = isGameUnitTracker(normalized);
  const deliveryMode = detectSoftwareDeliveryMode(normalized);
  return {
    action,
    artifactSurface: artifact.surface,
    artifact: artifact.label,
    target: extractSoftwareTarget(prompt, artifact),
    requirements,
    subtasks: deriveSoftwareSubtasks(requirements, gameTracker),
    deliveryMode,
    implementationLanguage: detectSoftwareImplementationLanguage(normalized),
    approvalGates: softwareApprovalGates(normalized, deliveryMode),
    gameTracker,
  };
}

function softwareReasoningSteps(meaning) {
  const steps = [
    `Classify the impulse as a request to ${meaning.action} a ${meaning.artifact} instead of a fact lookup.`,
    `Bind the target environment to ${meaning.target} and keep the first response reviewable.`,
    `Extract ${meaning.requirements.length} requirement(s) into the meaning record before planning.`,
    `Decompose the requirement graph into ${meaning.subtasks.length} implementation subtask(s) with category labels.`,
    `Select delivery mode ${meaning.deliveryMode} and approval gates: ${meaning.approvalGates.join(", ")}.`,
  ];
  if (meaning.gameTracker) {
    steps.push(
      "Map HP, Protection, Resistance, damage, and cooldown phrases to a unit-state domain model.",
    );
  }
  steps.push("Ask for approval before producing code, scripts, manual instructions, or execution steps.");
  return steps;
}

function softwarePlanSteps(meaning) {
  const steps = [
    "Review the formalized task, requirement graph, approval gates, and delivery mode with the user.",
  ];
  if (meaning.gameTracker) {
    steps.push(
      `Confirm the ${meaning.target} storage and selected-token API boundaries.`,
      "Define `UnitState` with HP, max HP, Protection, Resistance, and cooldowns.",
      "Write pure transition functions for damage mitigation, stack edits, and round ticks.",
      "Add tests for zero damage, overkill damage, stack changes, and cooldown expiry.",
      "Wire the tested core into the extension panel and host persistence.",
    );
    return steps;
  }
  steps.push(
    `Confirm the host API and data boundaries for ${meaning.target}.`,
    "Define the smallest serializable state records for the requirements.",
  );
  for (const subtask of meaning.subtasks) {
    steps.push(`Implement ${subtask.category}: ${subtask.title}.`);
  }
  steps.push(
    `Generate a ${meaning.implementationLanguage} starter core plus language-appropriate repository initialization and checks.`,
  );
  steps.push("Keep shell, Docker, or WebVM commands behind the configured approval gates.");
  return steps;
}

function softwareEvidence(meaning, approved) {
  const evidence = [
    "formalization:text_to_links_notation",
    `meaning:${stableSoftwareMeaningId(meaning)}`,
    `software_project:action:${meaning.action}`,
    `software_project:artifact:${meaning.artifact}`,
    `software_project:target:${meaning.target}`,
    `software_project:domain:${softwareDomainLabel(meaning)}`,
    `software_project:delivery_mode:${meaning.deliveryMode}`,
    `software_project:implementation_language:${meaning.implementationLanguage}`,
    `approval_state:${softwareApprovalLabel(approved)}`,
    `software_project:strategy:${meaning.gameTracker ? "game_unit_tracker" : "bounded_project_plan"}`,
  ];
  for (const gate of meaning.approvalGates) {
    evidence.push(`approval_gate:${gate}`);
  }
  for (const requirement of meaning.requirements) {
    evidence.push(`requirement:${requirement}`);
    evidence.push(`requirement_category:${classifySoftwareRequirement(requirement, meaning.gameTracker)}`);
  }
  for (const subtask of meaning.subtasks) {
    evidence.push(`software_project:subtask:${subtask.requirementId}:${subtask.category}:${subtask.title}`);
  }
  return evidence;
}

function renderSoftwareProjectPlan(meaning) {
  const lines = [];
  lines.push(
    `Implementation plan pending approval for a ${meaning.artifact} targeting ${meaning.target}.`,
  );
  lines.push("");
  lines.push("Formalized meaning:");
  lines.push("```lino");
  lines.push(softwareMeaningLino(meaning, false).trimEnd());
  lines.push("```");
  lines.push("");
  lines.push("Reasoning steps:");
  softwareReasoningSteps(meaning).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  lines.push("Requirement model:");
  meaning.requirements.forEach((requirement, index) => {
    const category = classifySoftwareRequirement(requirement, meaning.gameTracker);
    lines.push(`${index + 1}. [${category}] ${requirement}`);
  });
  lines.push("");
  lines.push("Subtasks:");
  meaning.subtasks.forEach((subtask, index) => {
    lines.push(`${index + 1}. ${subtask.requirementId} -> ${subtask.title}`);
  });
  lines.push("");
  lines.push("Approval gates:");
  meaning.approvalGates.forEach((gate) => {
    lines.push(`- ${gate}`);
  });
  lines.push("");
  lines.push("Proposed plan:");
  softwarePlanSteps(meaning).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  lines.push(
    "Reply `approve plan` to generate the starter implementation, or describe what to change.",
  );
  return lines.join("\n");
}

function renderSoftwareProjectImplementation(meaning) {
  const lines = [];
  lines.push(
    `Approved implementation starter for a ${meaning.artifact} targeting ${meaning.target}.`,
  );
  lines.push("");
  lines.push("Formalized meaning:");
  lines.push("```lino");
  lines.push(softwareMeaningLino(meaning, true).trimEnd());
  lines.push("```");
  lines.push("");
  lines.push("Implementation steps:");
  softwarePlanSteps(meaning).forEach((step, index) => {
    lines.push(`${index + 1}. ${step}`);
  });
  lines.push("");
  const code = softwareImplementationCode(meaning);
  lines.push(`Starter ${code.label} core:`);
  lines.push("");
  lines.push(`\`\`\`${code.fence}`);
  lines.push(code.body);
  lines.push("```");
  lines.push("");
  lines.push("Generated code checks:");
  lines.push(`1. Initialize a ${code.label} project in an isolated workspace.`);
  lines.push("2. Run the language-native syntax/type check before host integration.");
  return lines.join("\n");
}

// Mirror is_approval_prompt in src/solver_handlers/software_project.rs: strip
// leading/trailing non-alphanumerics (Unicode-aware, so a non-Latin go-ahead
// compacts the same way), drop interior sentence punctuation, then match the
// whole compacted prompt against any approval-trigger surface word. The words
// live in data/seed/meanings-software-project.lino, not in code.
function isSoftwareApprovalPrompt(normalized) {
  const compact = String(normalized || "")
    .trim()
    .replace(/^[^\p{L}\p{N}]+|[^\p{L}\p{N}]+$/gu, "")
    .replace(/[.!,]/g, "");
  return meaningsWithRole(ROLE_SOFTWARE_APPROVAL_TRIGGER).some((meaning) =>
    meaning.words.some((word) => compact === word),
  );
}

function lastHistoryTurn(history, role) {
  if (!Array.isArray(history)) return null;
  for (let index = history.length - 1; index >= 0; index -= 1) {
    const turn = history[index];
    if (turn && turn.role === role && turn.content) return String(turn.content);
  }
  return null;
}

function priorSoftwareProjectMeaning(history) {
  const assistant = lastHistoryTurn(history, "assistant");
  if (
    !assistant ||
    !assistant.includes("software_project_request") ||
    !assistant.includes("approve plan")
  ) {
    return null;
  }
  const user = lastHistoryTurn(history, "user");
  return user ? formalizeSoftwareProjectRequest(user) : null;
}

const INSTALL_FORMAT_MARKDOWN = "markdown";
const INSTALL_FORMAT_SHELL = "shell_script";
const INSTALL_FORMAT_POWERSHELL = "powershell_script";

const INSTALL_ALGORITHM_CONSTRUCTION_STAGES = [
  {
    id: "collect_corpus",
    output: "representative problem-class examples",
    verifier: "case-study corpus preserved",
  },
  {
    id: "derive_surfaces",
    output: "source and target surface ontology",
    verifier: "source/target format detection",
  },
  {
    id: "extract_ir",
    output: "shared intermediate representation",
    verifier: "ordered command preservation fixture",
  },
  {
    id: "synthesize_operations",
    output: "recognizers, extractors, renderers, and validators",
    verifier: "round-trip surface invariants",
  },
  {
    id: "project_targets",
    output: "target-specific Markdown, shell, and PowerShell renderers",
    verifier: "per-target rendering fixture",
  },
  {
    id: "mirror_runtimes",
    output: "Rust and browser-worker projections of the same algorithm",
    verifier: "cross-runtime parity checks",
  },
  {
    id: "promote_capability",
    output: "reusable coding-task construction pattern",
    verifier: "catalog, synthesis, blueprint, and rule-synthesis compatibility",
  },
];

const INSTALL_CODING_SURFACE_PROJECTIONS = [
  {
    slug: "coding_catalog",
    projection: "task spec -> parameterized template -> CST/compile check",
  },
  {
    slug: "program_synthesis",
    projection: "semantic function tree -> source program -> sandbox tests",
  },
  {
    slug: "program_blueprint",
    projection: "capability set -> blueprint recipe -> honest code projection",
  },
  {
    slug: "numeric_list",
    projection: "operation/data/language IR -> generated code plus evaluated result",
  },
  {
    slug: "rule_synthesis",
    projection: "operation/target binding -> candidate rule -> verification fixture",
  },
  {
    slug: "installation_conversion",
    projection: "installation surfaces -> install-step IR -> target renderers",
  },
];

function installationContainsAny(value, needles) {
  return needles.some((needle) => String(value || "").includes(needle));
}

function isInstallationConversionRequest(normalized) {
  const asksConversion = installationContainsAny(normalized, [
    "convert",
    "conversion",
    "transform",
    "turn",
    "translate",
    "back to",
    "конверт",
    "преобраз",
    "перевед",
    "बदल",
    "परिवर्त",
    "रूपांतर",
    "कन्वर्ट",
    "转换",
    "轉換",
    "转成",
    "轉成",
    "转为",
    "轉為",
    "翻译",
    "翻譯",
  ]);
  const namesInstallSurface = installationContainsAny(normalized, [
    "readme",
    "markdown",
    "installation guide",
    "install guide",
    "deployment guide",
    "deploy guide",
    "installation script",
    "install script",
    "deployment script",
    "deploy script",
    "руководство по установ",
    "инструкц",
    "установ",
    "स्थापना",
    "इंस्टॉल",
    "इंस्टॉलेशन",
    "安装",
    "安裝",
    "部署",
  ]);
  const namesScriptSurface = installationContainsAny(normalized, [
    " sh ",
    " bash",
    "shell",
    "powershell",
    "pwsh",
    "ps1",
    "script",
    "скрипт",
    "скрипта",
    "脚本",
    "腳本",
  ]);
  return asksConversion && namesInstallSurface && namesScriptSurface;
}

function installationFencedBlocks(text) {
  const blocks = [];
  let currentInfo = null;
  let currentBody = [];
  for (const line of String(text || "").split(/\r?\n/)) {
    const trimmed = line.trimStart();
    if (trimmed.startsWith("```")) {
      if (currentInfo !== null) {
        blocks.push({
          info: currentInfo,
          body: currentBody.join("\n").replace(/\n+$/g, ""),
        });
        currentInfo = null;
        currentBody = [];
      } else {
        currentInfo = trimmed.slice(3).trim().split(/\s+/, 1)[0].toLowerCase();
      }
      continue;
    }
    if (currentInfo !== null) currentBody.push(line);
  }
  return blocks;
}

function isInstallationShellFence(info) {
  return ["bash", "sh", "shell", "zsh"].includes(String(info || ""));
}

function isInstallationPowerShellFence(info) {
  return ["powershell", "pwsh", "ps1"].includes(String(info || ""));
}

function detectInstallationSourceFormat(prompt, normalized) {
  const fences = installationFencedBlocks(prompt);
  const explicitPowerShell = installationContainsAny(normalized, [
    "this powershell",
    "powershell installation script",
    "powershell script back",
    "ps1 script",
  ]);
  const explicitShell = installationContainsAny(normalized, [
    "this shell",
    "this bash",
    "shell installation script",
    "shell script back",
    "bash script back",
  ]);
  const explicitMarkdown = installationContainsAny(normalized, [
    "this readme",
    "readme.md installation guide",
    "readme installation guide",
    "this markdown",
    "markdown installation guide",
  ]);
  if (explicitPowerShell) {
    return INSTALL_FORMAT_POWERSHELL;
  }
  if (explicitShell) {
    return INSTALL_FORMAT_SHELL;
  }
  if (explicitMarkdown) {
    return INSTALL_FORMAT_MARKDOWN;
  }
  if (fences.some((block) => isInstallationPowerShellFence(block.info))) {
    return INSTALL_FORMAT_POWERSHELL;
  }
  if (fences.some((block) => isInstallationShellFence(block.info))) {
    return INSTALL_FORMAT_SHELL;
  }
  if (fences.some((block) => block.info === "markdown" || block.info === "md")) {
    return INSTALL_FORMAT_MARKDOWN;
  }
  return INSTALL_FORMAT_MARKDOWN;
}

function pushInstallationTarget(targets, target) {
  if (!targets.includes(target)) targets.push(target);
}

function detectInstallationTargetFormats(normalized, sourceFormat) {
  const targets = [];
  if (
    installationContainsAny(normalized, [
      "back to a readme",
      "back to readme",
      "to a readme",
      "to readme",
      "to markdown",
      "markdown guide",
    ])
  ) {
    pushInstallationTarget(targets, INSTALL_FORMAT_MARKDOWN);
  }
  if (
    installationContainsAny(normalized, [
      "both sh and powershell",
      "both bash and powershell",
      "sh and powershell",
      "bash and powershell",
    ])
  ) {
    pushInstallationTarget(targets, INSTALL_FORMAT_SHELL);
    pushInstallationTarget(targets, INSTALL_FORMAT_POWERSHELL);
  }
  if (
    installationContainsAny(normalized, [
      "into a sh script",
      "to a sh script",
      "into sh",
      "to sh",
      "into a shell script",
      "to a shell script",
      "into a bash script",
      "to a bash script",
    ])
  ) {
    pushInstallationTarget(targets, INSTALL_FORMAT_SHELL);
  }
  if (
    sourceFormat !== INSTALL_FORMAT_POWERSHELL &&
    installationContainsAny(normalized, [
      "into a powershell script",
      "to a powershell script",
      "into powershell",
      "to powershell",
      "to ps1",
      "into ps1",
    ])
  ) {
    pushInstallationTarget(targets, INSTALL_FORMAT_POWERSHELL);
  }
  if (targets.length === 0) {
    if (sourceFormat === INSTALL_FORMAT_MARKDOWN) {
      pushInstallationTarget(targets, INSTALL_FORMAT_SHELL);
    } else {
      pushInstallationTarget(targets, INSTALL_FORMAT_MARKDOWN);
    }
  }
  return targets;
}

function extractInstallationSourceText(prompt, sourceFormat) {
  const fences = installationFencedBlocks(prompt);
  const matching = fences.find((block) => {
    if (sourceFormat === INSTALL_FORMAT_MARKDOWN) {
      return block.info === "markdown" || block.info === "md";
    }
    if (sourceFormat === INSTALL_FORMAT_SHELL) return isInstallationShellFence(block.info);
    return isInstallationPowerShellFence(block.info);
  });
  if (matching) return matching.body;
  if (sourceFormat === INSTALL_FORMAT_MARKDOWN) return String(prompt || "");
  if (fences.length > 0) return fences[0].body;
  return String(prompt || "");
}

function normalizeInstallationScriptLine(line) {
  return String(line || "").trim().replace(/^\$ /, "").replace(/^PS> /, "").trim();
}

function shouldSkipInstallationScriptLine(line) {
  return (
    line === "" ||
    line.startsWith("#!") ||
    line.startsWith("#") ||
    line === "set -e" ||
    line === "set -eu" ||
    line === "set -euo pipefail" ||
    line === "$ErrorActionPreference = 'Stop'"
  );
}

// Provenance of a candidate line. Mirrors the Rust `Provenance` enum: code
// spans/fences are author-marked code (weak shape check), bare lines must prove
// themselves structurally.
const INSTALL_PROVENANCE_CODE_SPAN = "code_span";
const INSTALL_PROVENANCE_BARE_LINE = "bare_line";

const INSTALL_COMMAND_FUNCTION_WORDS = new Set([
  "the",
  "a",
  "an",
  "and",
  "or",
  "to",
  "with",
  "into",
  "from",
  "your",
  "you",
  "our",
  "this",
  "that",
  "these",
  "those",
  "then",
  "will",
  "should",
  "must",
  "please",
  "manually",
]);

// True when the token is shaped like an executable name or a path to one rather
// than a natural-language word. Commands are lowercase by convention, so an
// uppercase or non-ASCII lead immediately reads as prose.
function isInstallationExecutableHead(token) {
  if (!token) return false;
  const first = token[0];
  const startsOk = /[a-z0-9./]/.test(first);
  if (!startsOk) return false;
  return /^[a-z0-9./_+-]+$/.test(token);
}

function installationHasShellOperator(command) {
  return (
    command.includes(" | ") ||
    command.includes("&&") ||
    command.includes("||") ||
    command.includes(" ; ")
  );
}

function installationReadsAsProse(tokens) {
  return tokens.some((token) => {
    const word = token.replace(/^[^0-9a-z]+/i, "").replace(/[^0-9a-z]+$/i, "").toLowerCase();
    return INSTALL_COMMAND_FUNCTION_WORDS.has(word);
  });
}

// Decide whether `command` is an install/deploy command by reasoning about its
// structure and provenance instead of matching a fixed tool whitelist. Any
// well-formed command line is accepted regardless of which tool it invokes,
// while prose lines are rejected even when they mention a tool.
function looksLikeInstallationCommand(command, provenance = INSTALL_PROVENANCE_CODE_SPAN) {
  const trimmed = String(command || "").trim();
  if (!trimmed) return false;

  // A raw prose line that embeds a code span ("Run `npm install`.") is prose:
  // the inline/fence collectors already lifted the real command out.
  if (provenance === INSTALL_PROVENANCE_BARE_LINE && trimmed.includes("`")) return false;

  const tokens = trimmed.split(/\s+/);
  const head = tokens[0].toLowerCase();
  if (!isInstallationExecutableHead(head)) return false;

  // Shell composition is unambiguous command shape regardless of provenance.
  if (installationHasShellOperator(trimmed)) return true;

  // An executable-looking head can still front a wrapped prose note; English
  // function words betray it.
  if (installationReadsAsProse(tokens)) return false;

  if (provenance === INSTALL_PROVENANCE_BARE_LINE) {
    return tokens.length >= 2 || head.includes("/");
  }
  return true;
}
