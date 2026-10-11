// The computer-use benchmark seed: a port of rust/src/computer_use/seed.rs.
//
// Reads data/seed/computer-use-tasks.lino (`crate::seed::COMPUTER_USE_TASKS_LINO`)
// line by line exactly as the Rust parser does, and data/seed/tools.lino
// (`crate::seed::TOOLS_LINO`) for `tool_description`.

import { cached, readText } from '../host.mjs';
import { rustLines } from '../content.mjs';
import { byteOrder, splitWhitespace, stripPrefix, trim } from './rust_str.mjs';
import { computerPrimitiveFromToolName } from '../protocol_policy.mjs';

const TASKS_PATH = 'data/seed/computer-use-tasks.lino';
const TOOLS_PATH = 'data/seed/tools.lino';

/** `serde_json::from_str::<String>`: the decoded string or null. */
function jsonString(text) {
  try {
    const value = JSON.parse(text);
    return typeof value === 'string' ? value : null;
  } catch {
    return null;
  }
}

/** `serde_json::from_str::<Value>`: the decoded value or undefined. */
function jsonValue(text) {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

/** `str::split_once(' ')`. */
function splitOnceSpace(text) {
  const at = text.indexOf(' ');
  return at < 0 ? null : [text.slice(0, at), text.slice(at + 1)];
}

/** Mirrors `fn parse_localized`. */
function parseLocalized(rest) {
  const split = splitOnceSpace(trim(rest));
  if (!split) return null;
  const value = jsonString(split[1]);
  return value === null ? null : [split[0], value];
}

/** Mirrors `fn parse_primitive_template`. */
function parsePrimitiveTemplate(rest) {
  const split = splitOnceSpace(trim(rest));
  if (!split) return null;
  if (computerPrimitiveFromToolName(split[0]) === null) return null;
  const value = jsonString(split[1]);
  return value === null ? null : [split[0], value];
}

/** Mirrors `fn parse_step_arguments`. */
function parseStepArguments(encoded) {
  const trimmed = trim(encoded);
  const json = trimmed.startsWith("'") && trimmed.length >= 2 && trimmed.endsWith("'")
    ? trimmed.slice(1, -1)
    : trimmed;
  return jsonValue(json);
}

/** Mirrors `fn render_template`. @param {Array<[string, string]>} values */
export function renderTemplate(template, values) {
  return values.reduce((rendered, [name, value]) => rendered.split(`{${name}}`).join(value), template);
}

/** Mirrors `fn localized_template`. @param {Map<string, string>} templates */
function localizedTemplate(templates, locale) {
  return templates.get(locale) ?? templates.get('en') ?? '';
}

/** Mirrors `fn primitive_template`. @param {Map<string, string>} templates */
function primitiveTemplate(templates, primitive) {
  return templates.get(primitive) ?? primitive;
}

/** Mirrors `fn normalize` in rust/src/computer_use/seed.rs. */
function normalize(value) {
  return splitWhitespace(value).join(' ').toLowerCase();
}

/** Insert into a sorted `[[key, value]]` list (the `BTreeMap::insert` twin). */
function sortedInsert(entries, key, value) {
  const existing = entries.find((entry) => entry[0] === key);
  if (existing) {
    existing[1] = value;
    return;
  }
  entries.push([key, value]);
  entries.sort((left, right) => byteOrder(left[0], right[0]));
}

/** Mirrors `fn parse_seed` (cached like the Rust `OnceLock`). */
function parsed() {
  return cached('computer-use-seed', () => {
    const seed = {
      tasks: [],
      completions: new Map(),
      missing: new Map(),
      verification_failures: new Map(),
      preconditions: new Map(),
      postconditions: new Map(),
      gap_cues: [],
      gap_responses: new Map(),
    };
    let currentTask = null;
    let currentGap = null;
    const localizedInto = (map, rest) => {
      const pair = parseLocalized(rest);
      if (pair) map.set(pair[0], pair[1]);
    };
    const templateInto = (map, rest) => {
      const pair = parsePrimitiveTemplate(rest);
      if (pair) map.set(pair[0], pair[1]);
    };
    for (const line of rustLines(readText(TASKS_PATH))) {
      let rest;
      if ((rest = stripPrefix(line, '  completion ')) !== null) {
        localizedInto(seed.completions, rest);
      } else if ((rest = stripPrefix(line, '  missing ')) !== null) {
        localizedInto(seed.missing, rest);
      } else if ((rest = stripPrefix(line, '  verification_failed ')) !== null) {
        localizedInto(seed.verification_failures, rest);
      } else if ((rest = stripPrefix(line, '  precondition ')) !== null) {
        templateInto(seed.preconditions, rest);
      } else if ((rest = stripPrefix(line, '  postcondition ')) !== null) {
        templateInto(seed.postconditions, rest);
      } else if ((rest = stripPrefix(line, '  task ')) !== null) {
        if (currentTask) seed.tasks.push(currentTask);
        currentGap = null;
        currentTask = { id: trim(rest), prompts: [], steps: [] };
      } else if ((rest = stripPrefix(line, '  capability_gap ')) !== null) {
        if (currentTask) seed.tasks.push(currentTask);
        currentTask = null;
        currentGap = trim(rest);
      } else if ((rest = stripPrefix(line, '    prompt ')) !== null) {
        if (!currentTask) continue;
        const pair = parseLocalized(rest);
        if (pair) sortedInsert(currentTask.prompts, pair[0], pair[1]);
      } else if ((rest = stripPrefix(line, '    step ')) !== null) {
        if (!currentTask) continue;
        const split = splitOnceSpace(trim(rest));
        if (!split) continue;
        const primitive = computerPrimitiveFromToolName(split[0]);
        const args = parseStepArguments(split[1]);
        if (primitive === null || args === undefined) continue;
        const number = String(currentTask.steps.length + 1).padStart(2, '0');
        currentTask.steps.push({
          id: `${currentTask.id}-${number}`,
          primitive,
          arguments: args,
          precondition: renderTemplate(primitiveTemplate(seed.preconditions, primitive),
            [['permission', `tool:computer:${primitive}`]]),
          postcondition: primitiveTemplate(seed.postconditions, primitive),
        });
      } else if ((rest = stripPrefix(line, '    cue ')) !== null) {
        const pair = parseLocalized(rest);
        if (currentGap !== null && pair) seed.gap_cues.push([currentGap, pair[0], pair[1]]);
      } else if ((rest = stripPrefix(line, '    response ')) !== null) {
        const pair = parseLocalized(rest);
        if (currentGap !== null && pair) seed.gap_responses.set(gapKey(currentGap, pair[0]), pair[1]);
      }
    }
    if (currentTask) seed.tasks.push(currentTask);
    return seed;
  });
}

const gapKey = (capability, locale) => JSON.stringify([capability, locale]);

/** A deep copy of a plan step: Rust built-in `#[derive(Clone)]` on `ComputerPlanStep`. */
export function cloneStep(step) {
  return { ...step, arguments: structuredClone(step.arguments) };
}

/**
 * Mirrors `fn benchmark_tasks`: `{id, prompts: [[locale, prompt]] (sorted by
 * locale), steps}` records in seed order.
 */
export function benchmarkTasks() {
  return parsed().tasks;
}

/** Mirrors `fn plan_for_prompt`: the recorded plan for a verbatim benchmark prompt, or null. */
export function planForPrompt(prompt) {
  const normalized = normalize(prompt);
  for (const task of benchmarkTasks()) {
    for (const [locale, candidate] of task.prompts) {
      if (normalize(candidate) === normalized) {
        return { id: task.id, locale, prompt: candidate, steps: task.steps.map(cloneStep) };
      }
    }
  }
  return null;
}

/** Mirrors `fn capability_gap_for_prompt`. */
export function capabilityGapForPrompt(prompt) {
  const normalized = normalize(prompt);
  const seed = parsed();
  const cue = seed.gap_cues.find(([, , text]) => normalize(text) === normalized);
  if (!cue) return null;
  const response = seed.gap_responses.get(gapKey(cue[0], cue[1]));
  return response === undefined ? null : { capability: cue[0], locale: cue[1], response };
}

/** Mirrors `fn completion_message`. */
export function completionMessage(locale, planId, steps) {
  return renderTemplate(localizedTemplate(parsed().completions, locale),
    [['plan_id', planId], ['steps', String(steps)]]);
}

/** Mirrors `fn missing_primitive_message`. */
export function missingPrimitiveMessage(locale, planId, stepId, primitive) {
  return renderTemplate(localizedTemplate(parsed().missing, locale),
    [['plan_id', planId], ['step_id', stepId], ['primitive', primitive]]);
}

/** Mirrors `fn verification_failed_message`. */
export function verificationFailedMessage(locale, planId, stepId, primitive) {
  return renderTemplate(localizedTemplate(parsed().verification_failures, locale),
    [['plan_id', planId], ['step_id', stepId], ['primitive', primitive]]);
}

/** Mirrors `fn step_conditions`: `[precondition, postcondition]` for a primitive. */
export function stepConditions(primitive) {
  const seed = parsed();
  return [
    renderTemplate(primitiveTemplate(seed.preconditions, primitive), [['permission', `tool:computer:${primitive}`]]),
    primitiveTemplate(seed.postconditions, primitive),
  ];
}

/** Mirrors `fn capability_gap_response` (English fallback). */
export function capabilityGapResponse(capability, locale) {
  const responses = parsed().gap_responses;
  const response = responses.get(gapKey(capability, locale)) ?? responses.get(gapKey(capability, 'en'));
  return response === undefined ? null : { capability, locale, response };
}

/** Mirrors `fn tool_description`: the `note` under `    name <primitive>` in tools.lino. */
export function toolDescription(primitive) {
  const lines = rustLines(readText(TOOLS_PATH));
  const index = lines.indexOf(`    name ${primitive}`);
  const next = index < 0 ? undefined : lines[index + 1];
  const quoted = next === undefined ? null : stripPrefix(trim(next), 'note ');
  return (quoted === null ? null : jsonString(quoted)) ?? primitive;
}
