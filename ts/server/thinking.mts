// Thinking steps (rust/src/thinking.rs, rust/src/thinking_prose.rs): the
// `ThinkingStep` wire value, the seed-driven `(step, detail) -> sentence`
// naturalizer, the narrative headline, and the plain-text rendering the
// protocol surfaces carry as `reasoning`.
//
// Every sentence comes from data/seed/multilingual-responses-thinking*.lino,
// read the way the native index reads it: `(intent, language) -> text`, with
// English as the fallback language.

import { readdirSync } from 'node:fs';
import path from 'node:path';

import { stableId } from './ids.mjs';
import { REPO_ROOT, childValue, childrenNamed, parseLino, readRepoFile } from './lino.mjs';

const STEP_INTENT_PREFIX = 'thinking_step_';
const PLAIN_INTENT_SUFFIX = '_plain';
const NARRATIVE_INTENT_PREFIX = 'thinking_narrative_';
const LANGUAGE_NAME_INTENT_PREFIX = 'thinking_language_name_';
const FALLBACK_LANGUAGE = 'en';
const DETAIL_LIMIT = 600;

/** `(step kind, placeholder, humanize?)` - `DETAIL_STEPS`. */
const DETAIL_STEPS = [
  ['impulse', 'prompt', false],
  ['formalize', 'task', true],
  ['formalize_resolved', 'entity', true],
  ['clarify_formalization', 'options', false],
  ['dispatch_handler', 'route', true],
  ['route_attempt', 'route', true],
  ['match_rule', 'rule', true],
  ['compute', 'expression', false],
  ['compute_engine', 'engine', true],
  ['lookup_fact', 'fact', true],
  ['invoke_tool', 'tool', true],
  ['rule_verification', 'rule', true],
  ['policy_refusal', 'policy', true],
  ['program_plan', 'plan', true],
  ['scan_memory', 'term', false],
  ['user_context', 'context', false],
  ['deformalize', 'answer', false],
  ['agent_plan', 'task', true],
];

const ALWAYS_DETAIL_STEPS = [
  ['compute_expression', 'expression'],
  ['compute_steps', 'count'],
];

const PLAIN_STEPS = [
  'rule_construction',
  'coreference_binding',
  'modifier_detection',
  'http_chat',
  'memory',
  'extract_term',
  'group_by_conversation',
  'fallback',
];

/** `ROUTE_NARRATIVES`: route key -> narrative intent suffix. */
const ROUTE_NARRATIVES = new Map([
  ['greeting', 'greeting'],
  ['wellbeing', 'wellbeing'],
  ['assistant_free_time', 'assistant_free_time'],
  ['farewell', 'farewell'],
  ['gratitude', 'gratitude'],
  ['thanks', 'gratitude'],
  ['courtesy_response', 'gratitude'],
  ['courtesy', 'gratitude'],
  ['identity', 'identity'],
  ['assistant_name', 'identity'],
  ['recall_name', 'identity'],
  ['naming', 'identity'],
  ['assistant_naming', 'identity'],
  ['calculation', 'calculation'],
  ['arithmetic', 'calculation'],
  ['fact_lookup', 'fact_lookup'],
  ['concept_lookup', 'fact_lookup'],
  ['concept_lookup_in_context', 'fact_lookup'],
  ['translation', 'translation'],
  ['web_search', 'web'],
  ['http_fetch', 'web'],
  ['url_navigate', 'web'],
  ['write_program', 'code'],
  ['software_project_plan', 'code'],
  ['software_project_implementation', 'code'],
  ['algorithm', 'code'],
  ['test_status', 'test_status'],
  ['self_healing', 'self_healing'],
  ['self_heal', 'self_healing'],
  ['meta_explanation', 'meta_explanation'],
  ['learn_from_source', 'learn_from_source'],
  ['clarification', 'clarification'],
  ['unknown', 'unknown'],
  ['fallback', 'unknown'],
]);

let proseIndex = null;
let languageNames = null;

function index() {
  if (proseIndex) return proseIndex;
  proseIndex = new Map();
  const files = readdirSync(path.join(REPO_ROOT, 'data/seed'))
    .filter((name) => name.startsWith('multilingual-responses-thinking') && name.endsWith('.lino'))
    .sort();
  for (const file of files) {
    for (const record of childrenNamed(parseLino(readRepoFile(`data/seed/${file}`)), 'response')) {
      const intent = childValue(record, 'intent');
      if (!intent.startsWith('thinking_')) continue;
      proseIndex.set(`${intent}\u0000${childValue(record, 'language')}`, childValue(record, 'text'));
    }
  }
  return proseIndex;
}

function registeredLanguageNames() {
  if (languageNames) return languageNames;
  languageNames = new Map();
  for (const language of childrenNamed(parseLino(readRepoFile('data/seed/languages.lino')), 'language')) {
    languageNames.set(language.value, childValue(language, 'name'));
  }
  return languageNames;
}

/** `normalize_language`: the primary subtag, lowercased. */
export function normalizeLanguage(code) {
  return String(code).trim().toLowerCase().split(/[-_]/)[0];
}

/** `thinking_prose`: one sentence with its `{fields}` filled, or null. */
export function thinkingProse(intent, language, fields = []) {
  const table = index();
  const text = table.get(`${intent}\u0000${normalizeLanguage(language)}`)
    ?? table.get(`${intent}\u0000${FALLBACK_LANGUAGE}`);
  if (text === undefined) return null;
  return fields.reduce((rendered, [name, value]) => rendered.split(`{${name}}`).join(value), text);
}

/** `language_label`: the language `code` named in `answerLanguage`. */
export function languageLabel(answerLanguage, code) {
  const slug = normalizeLanguage(code) || 'unknown';
  const name = thinkingProse(`${LANGUAGE_NAME_INTENT_PREFIX}${slug}`, answerLanguage);
  if (name !== null) return name;
  return registeredLanguageNames().get(slug) || slug;
}

/** `humanize_meta_identifier`: `write_program` -> `write program`. */
export function humanizeMetaIdentifier(value) {
  let spaced = '';
  let previousLower = false;
  for (const character of String(value)) {
    if (/[A-Z]/.test(character) && previousLower) spaced += ' ';
    spaced += /[_:.\-/]/.test(character) ? ' ' : character;
    previousLower = /[a-z0-9]/.test(character);
  }
  return spaced.split(/\s+/).filter(Boolean).join(' ').trim().replace(/[A-Z]/g, (c) => c.toLowerCase());
}

function indefiniteArticle(phrase) {
  const first = String(phrase).trimStart()[0];
  return first && 'aeiou'.includes(first.toLowerCase()) ? 'an' : 'a';
}

/** `strip_agent_substep_prefix`: `agent_0_impulse` -> `impulse`. */
function stripAgentSubstepPrefix(step) {
  const match = /^agent_(\d+)_(.*)$/s.exec(step);
  return match ? match[2] : step;
}

function truncateDetail(value) {
  const trimmed = String(value).trim();
  const chars = [...trimmed];
  if (chars.length <= DETAIL_LIMIT) return trimmed;
  return `${chars.slice(0, DETAIL_LIMIT - 1).join('').trimEnd()}…`;
}

function genericSentence(language, canonical, trimmed, hasDetail) {
  const readable = humanizeMetaIdentifier(canonical);
  const label = readable || thinkingProse(`${STEP_INTENT_PREFIX}unnamed`, language) || canonical;
  const intent = `${STEP_INTENT_PREFIX}generic${hasDetail ? '' : PLAIN_INTENT_SUFFIX}`;
  const text = thinkingProse(intent, language, [['label', label], ['detail', trimmed]]);
  if (text !== null) return text;
  return hasDetail ? `${label}: ${trimmed}` : label;
}

/** `naturalize_thinking_step_in`. */
export function naturalizeThinkingStep(language, step, detail) {
  const canonical = stripAgentSubstepPrefix(step);
  const trimmed = truncateDetail(detail);
  const hasDetail = trimmed.length > 0;
  if (canonical === 'detect_language' || canonical === 'resolve_response_language') {
    const text = thinkingProse(`${STEP_INTENT_PREFIX}${canonical}`, language, [['language', languageLabel(language, detail)]]);
    if (text !== null) return text;
  }
  const detailStep = DETAIL_STEPS.find(([kind]) => kind === canonical);
  if (detailStep) {
    const [, placeholder, humanize] = detailStep;
    const intent = `${STEP_INTENT_PREFIX}${canonical}${hasDetail ? '' : PLAIN_INTENT_SUFFIX}`;
    const value = humanize ? humanizeMetaIdentifier(trimmed) : trimmed;
    const text = thinkingProse(intent, language, [[placeholder, value], ['article', indefiniteArticle(value)]]);
    if (text !== null) return text;
  }
  const always = ALWAYS_DETAIL_STEPS.find(([kind]) => kind === canonical);
  if (always) {
    const text = thinkingProse(`${STEP_INTENT_PREFIX}${canonical}`, language, [[always[1], trimmed]]);
    if (text !== null) return text;
  }
  if (PLAIN_STEPS.includes(canonical)) {
    const text = thinkingProse(`${STEP_INTENT_PREFIX}${canonical}`, language);
    if (text !== null) return text;
  }
  return genericSentence(language, canonical, trimmed, hasDetail);
}

/** `thinking_answer_language`: the language a finished trace narrates in. */
export function thinkingAnswerLanguage(steps) {
  for (const kind of ['resolve_response_language', 'detect_language']) {
    const step = steps.find((candidate) => stripAgentSubstepPrefix(candidate.step) === kind);
    if (step) {
      const slug = normalizeLanguage(step.detail);
      if (slug && registeredLanguageNames().has(slug)) return slug;
    }
  }
  return FALLBACK_LANGUAGE;
}

/** `thinking_narrative_in`: the human headline, or null. */
export function thinkingNarrative(language, steps) {
  const routeStep = steps.find((step) => stripAgentSubstepPrefix(step.step) === 'dispatch_handler')
    || steps.find((step) => stripAgentSubstepPrefix(step.step) === 'formalize');
  if (!routeStep) return null;
  const route = String(routeStep.detail).trim().toLowerCase();
  if (!route) return null;
  if (ROUTE_NARRATIVES.has(route)) {
    const text = thinkingProse(`${NARRATIVE_INTENT_PREFIX}${ROUTE_NARRATIVES.get(route)}`, language);
    if (text !== null) return text;
  }
  const task = humanizeMetaIdentifier(route);
  return thinkingProse(`${NARRATIVE_INTENT_PREFIX}generic`, language, [['task', task], ['article', indefiniteArticle(task)]]);
}

/**
 * `ThinkingStep::new` (+ `with_parent`), in struct field order: `summary` and
 * `parent_id` are omitted when empty, as serde skips them.
 */
export function thinkingStep(order, step, detail, level, sourceEvent, parentId = null, language = FALLBACK_LANGUAGE) {
  const out = {
    id: stableId('thinking_step', `${order}:${step}:${detail}:${level}:${sourceEvent}`),
    order,
    step,
    detail,
  };
  const summary = naturalizeThinkingStep(language, step, detail);
  if (summary) out.summary = summary;
  out.level = level;
  out.source_event = sourceEvent;
  if (parentId) out.parent_id = parentId;
  return out;
}

/** `localize_thinking_steps`: re-narrate every step in the trace's own language. */
export function localizeThinkingSteps(steps) {
  const language = thinkingAnswerLanguage(steps);
  return steps.map((step) => {
    const summary = naturalizeThinkingStep(language, step.step, step.detail);
    const out = { id: step.id, order: step.order, step: step.step, detail: step.detail };
    if (summary) out.summary = summary;
    out.level = step.level;
    out.source_event = step.source_event;
    if (step.parent_id) out.parent_id = step.parent_id;
    return out;
  });
}

/** Recast the worker's `steps` as localized `ThinkingStep`s. */
export function thinkingStepsFromWorker(steps) {
  const recast = steps.map((step, index) =>
    thinkingStep(index, String(step.step), String(step.detail ?? ''), String(step.level || 'high'), String(step.step)),
  );
  return localizeThinkingSteps(recast);
}

/** `render_thinking_steps`: headline, then one sentence per step. */
export function renderThinkingSteps(steps) {
  const language = thinkingAnswerLanguage(steps);
  const lines = [];
  const narrative = thinkingNarrative(language, steps);
  if (narrative !== null) lines.push(narrative);
  for (const step of steps) {
    const sentence = naturalizeThinkingStep(language, step.step, step.detail);
    lines.push(step.parent_id ? `  ↳ ${sentence}` : sentence);
  }
  return lines.join('\n');
}
