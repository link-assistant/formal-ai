// Compiler for arbitrary, freely-phrased natural-language procedures
// (issue #674): the part of rust/src/skill_procedure.rs
// (`compile_procedure`, `CompiledProcedure::{conformance_links_notation,
// restate_steps, links_notation}`, `ProcedureRun::links_notation`) and
// rust/src/skill_procedure/learning.rs (`default_capability_ledger`) that
// `agentic_coding::procedure` reaches. The artifact round trip lives in
// skill_procedure_artifact.mjs.
//
// A `CompiledProcedure` keeps the Rust field names: `{id,
// source_description, impulse_id, requirements: [{id, index, source_text,
// source_span}], trigger: {requirement_id, objects, source_text,
// source_span}, steps: [{id, index, requirement_id, kind, objects,
// target_language, source_text, source_span}], canonical_program}`. Spans are
// UTF-8 byte ranges, as in Rust.

import { cached, parseLino, readText } from '../host.mjs';
import { stableId } from './engine_stable_id.mjs';
import { impulseIdFor } from './intent_formalization.mjs';
import { matchIndices, orderedRequirementSpans } from './intent_formalization_requirements.mjs';
import { pushLinoNode } from './links_format.mjs';
import { isAlphanumeric, trim, utf8Len } from './rust_str.mjs';
import { responseFor } from './seed.mjs';
import { meaningsWithRole } from './seed_meanings.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

/** Mirrors `MINIMUM_STEPS` in rust/src/skill_procedure.rs. */
export const MINIMUM_STEPS = 2;
/** Mirrors `PROCEDURE_CONFORMANCE_TRIGGER` in rust/src/skill_procedure.rs. */
export const PROCEDURE_CONFORMANCE_TRIGGER = 'https://example.com/article';
/** Mirrors `crate::engine::KNOWLEDGE_SCHEMA_VERSION`. */
export const KNOWLEDGE_SCHEMA_VERSION = '0.2.0';

/** `crate::seed::ROLE_SKILL_PROCEDURE_*` and `ROLE_TRANSLATION_LANGUAGE` (rust/src/seed/roles/). */
export const ROLE_SKILL_PROCEDURE_TRIGGER_LEAD = 'skill_procedure_trigger_lead';
export const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR = 'skill_procedure_clause_separator';
export const ROLE_SKILL_PROCEDURE_STEP_VERB = 'skill_procedure_step_verb';
export const ROLE_SKILL_PROCEDURE_STEP_OBJECT = 'skill_procedure_step_object';
export const ROLE_TRANSLATION_LANGUAGE = 'translation_language';

const PROCEDURE_MEANINGS_FILE = 'data/seed/meanings-skill-procedure.lino';
const CAPABILITY_LEDGER_FILE = 'data/meta/procedure-capability-ledger.lino';

/** Mirrors `decode_codepoints` in rust/src/seed/parser.rs. */
function decodeCodepoints(raw) {
  let out = '';
  for (const part of raw.split(/\s+/u).filter(Boolean)) {
    const hex = part.startsWith('0x') || part.startsWith('0X');
    const parsed = hex ? parseInt(part.slice(2), 16) : parseInt(part, 10);
    if (Number.isFinite(parsed) && parsed <= 0x10ffff && !(parsed >= 0xd800 && parsed <= 0xdfff)) {
      out += String.fromCodePoint(parsed);
    }
  }
  return out;
}

/** Mirrors `fn surface_text` in rust/src/seed/meanings/parse.rs. */
function surfaceText(node) {
  const text = findChildValue(node, 'text');
  if (text) return text;
  const codepoints = findChildValue(node, 'codepoints');
  return codepoints ? decodeCodepoints(codepoints) : node.value;
}

/**
 * Mirrors `seed::parse_lexicon_text(PROCEDURE_MEANINGS_LINO)` for the fields
 * the compiler reads (`slug`, `roles`, lexeme word texts), the same walk as
 * js/agentic/crate/seed_meanings.mjs `lexicon` over a single file.
 */
function procedureLexicon() {
  return cached('skill-procedure-lexicon', () => {
    const root = parseLino(readText(PROCEDURE_MEANINGS_FILE));
    const containers = root.name === 'meanings' ? [root] : (root.children || []).filter((child) => child.name === 'meanings');
    const meanings = [];
    for (const container of containers.length ? containers : [root]) {
      for (const node of container.children || []) {
        if (node.name === 'meanings') continue;
        const meaning = { slug: node.name === 'meaning' ? node.value : node.name, roles: [], lexemes: [] };
        for (const child of node.children || []) {
          if (child.name === 'role') meaning.roles.push(child.value);
          else if (child.name === 'lexeme') {
            meaning.lexemes.push({ words: (child.children || []).filter((word) => word.name === 'word' || word.name === 'surface').map((word) => ({ text: surfaceText(word) })) });
          } else if (child.name === 'surface') meaning.lexemes.push({ words: [{ text: surfaceText(child) }] });
        }
        meanings.push(meaning);
      }
    }
    return meanings;
  });
}

const procedureMeaningsWithRole = (role) => procedureLexicon().filter((meaning) => meaning.roles.includes(role));

/** Mirrors `fn surfaces` in rust/src/skill_procedure.rs: every non-empty surface text. */
const surfaces = (meaning) => meaning.lexemes.flatMap((lexeme) => lexeme.words.map((word) => word.text)).filter((text) => text !== '');

/** Mirrors `fn procedure_role_surfaces` in rust/src/skill_procedure.rs. */
function procedureRoleSurfaces(role) {
  return procedureMeaningsWithRole(role).flatMap(surfaces);
}

/** Mirrors `fn meaning_has_role` in rust/src/skill_procedure.rs (the global lexicon). */
export function meaningHasRole(slug, role) {
  return meaningsWithRole(role).some((meaning) => meaning.slug === slug);
}

/**
 * Mirrors `fn default_capability_ledger` in rust/src/skill_procedure/learning.rs:
 * the reviewed lessons as `[{canonical_kind, surfaces: [{language, text}]}]`.
 * Only the fields the classifier reads are kept; the review-evidence
 * validation Rust applies when loading is not re-run (the shipped ledger
 * has no lessons).
 */
function defaultCapabilityLedger() {
  return cached('skill-procedure-capability-ledger', () => {
    const root = parseRoot(readText(CAPABILITY_LEDGER_FILE)).children.find((node) => node.name === 'procedure_capability_ledger');
    if (!root) return [];
    return (root.children || []).filter((node) => node.name === 'lesson').map((node) => {
      const unique = new Map();
      for (const surface of (node.children || []).filter((child) => child.name === 'surface')) {
        unique.set(findChildValue(surface, 'language'), trim(findChildValue(surface, 'text')).toLowerCase());
      }
      return {
        canonical_kind: findChildValue(node, 'canonical_kind'),
        surfaces: [...unique.entries()].sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
          .map(([language, text]) => ({ language, text })),
      };
    });
  });
}

/** Whether `candidate` (`{start, length}`, UTF-16 start, UTF-8 length) beats `current`. */
const better = (candidate, current) => current === null || candidate.start < current.start
  || (candidate.start === current.start && candidate.length > current.length);

/** Mirrors `fn first_match_in` in rust/src/skill_procedure.rs over `meanings`. */
function firstMatchIn(meanings, hay, role) {
  let best = null;
  for (const meaning of meanings) {
    if (!meaning.roles.includes(role)) continue;
    for (const word of surfaces(meaning)) {
      for (const start of matchIndices(hay, word)) {
        const candidate = { slug: meaning.slug, start, end: start + word.length, length: utf8Len(word) };
        if (better(candidate, best)) best = candidate;
      }
    }
  }
  return best;
}

/** Mirrors `fn first_match` in rust/src/skill_procedure.rs (the global lexicon). */
function firstMatch(hay, role) {
  return firstMatchIn(meaningsWithRole(role), hay, role);
}

/** Mirrors `fn first_procedure_match` in rust/src/skill_procedure.rs. */
function firstProcedureMatch(hay, role) {
  return firstMatchIn(procedureLexicon(), hay, role);
}

/** Mirrors `fn is_ideographic` in rust/src/skill_procedure.rs. */
const isIdeographic = (character) => /^[㐀-鿿豈-﫿]$/u.test(character);

/** Mirrors `fn is_standalone` in rust/src/skill_procedure.rs (UTF-16 indices). */
function isStandalone(lower, start, end) {
  const matched = Array.from(lower.slice(start, end));
  if (matched.some(isIdeographic)) return true;
  const before = Array.from(lower.slice(0, start)).pop();
  const after = Array.from(lower.slice(end))[0];
  const alnum = (character) => character !== undefined && isAlphanumeric(character);
  return (!alnum(matched[0]) || !alnum(before)) && (!alnum(matched[matched.length - 1]) || !alnum(after));
}

/** Mirrors `fn first_step_match` in rust/src/skill_procedure.rs. */
function firstStepMatch(hay, ledger) {
  let best = firstProcedureMatch(hay, ROLE_SKILL_PROCEDURE_STEP_VERB);
  for (const lesson of ledger) {
    for (const surface of lesson.surfaces) {
      for (const start of matchIndices(hay, surface.text)) {
        const end = start + surface.text.length;
        if (!isStandalone(hay, start, end)) continue;
        const candidate = { slug: lesson.canonical_kind, start, end, length: utf8Len(surface.text) };
        if (better(candidate, best)) best = candidate;
      }
    }
  }
  return best;
}

/** Mirrors `fn objects_in` in rust/src/skill_procedure.rs: object slugs in mention order. */
function objectsIn(clause, skip) {
  const hits = [];
  for (const meaning of procedureMeaningsWithRole(ROLE_SKILL_PROCEDURE_STEP_OBJECT)) {
    let earliest = null;
    for (const word of surfaces(meaning)) {
      for (const start of matchIndices(clause, word)) {
        const end = start + word.length;
        if (skip !== null && start < skip[1] && skip[0] < end) continue;
        earliest = earliest === null ? start : Math.min(earliest, start);
      }
    }
    if (earliest !== null) hits.push([earliest, meaning.slug]);
  }
  hits.sort((left, right) => left[0] - right[0]);
  return hits.map(([, slug]) => slug);
}

/** Mirrors `fn gap_name` in rust/src/skill_procedure.rs. */
function gapName(step) {
  return (responseFor('skill_gap_name', 'en') ?? '').split('{step}').join(step);
}

/** Mirrors `ProcedureStep::arguments`. */
export function stepArguments(step) {
  return step.target_language === null ? [...step.objects] : [...step.objects, step.target_language];
}

/** Mirrors `fn requirement_id` in rust/src/skill_procedure.rs. */
export function requirementId(impulseId, index, requirement) {
  return stableId('procedure_requirement',
    `${impulseId}:${index}:${requirement.source_span[0]}..${requirement.source_span[1]}:${requirement.source_text}`);
}

/** Mirrors `fn canonical_program` in rust/src/skill_procedure.rs. */
export function canonicalProgram(trigger, steps) {
  let out = 'procedure\n  trigger\n';
  for (const object of trigger.objects) out += `    object ${object}\n`;
  for (const step of steps) {
    out += `  step\n    index ${step.index}\n    kind ${step.kind}\n`;
    for (const argument of stepArguments(step)) out += `    argument ${argument}\n`;
  }
  return out;
}

/** Mirrors the step id derivation in `compile_procedure_with_ledger`. */
export function stepId(procedureId, step) {
  return stableId('compiled_procedure_step', `${procedureId}:${step.index}:${step.kind}:${stepArguments(step).join('+')}`);
}

/**
 * Mirrors `fn compile_procedure` (via `compile_procedure_with_ledger`) in
 * rust/src/skill_procedure.rs: `{procedure}` on success, else `{error}` with
 * `{kind: 'not_a_procedure'}` or `{kind: 'uncompilable_step', step, span, gap}`.
 * @param {string} description
 */
export function compileProcedureResult(description) {
  const ledger = defaultCapabilityLedger();
  const sourceRequirements = orderedRequirementSpans(description, procedureRoleSurfaces(ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR));
  const triggerPosition = sourceRequirements.findIndex((requirement) =>
    firstProcedureMatch(requirement.source_text.toLowerCase(), ROLE_SKILL_PROCEDURE_TRIGGER_LEAD) !== null);
  const notAProcedure = { error: { kind: 'not_a_procedure' } };
  if (triggerPosition < 0) return notAProcedure;
  const stepRequirements = sourceRequirements.slice(triggerPosition + 1);
  if (stepRequirements.length < MINIMUM_STEPS) return notAProcedure;
  const classified = stepRequirements.map((requirement) => firstStepMatch(requirement.source_text.toLowerCase(), ledger));
  if (classified.filter((found) => found !== null).length < MINIMUM_STEPS) return notAProcedure;
  const gap = classified.findIndex((found) => found === null);
  if (gap >= 0) {
    const requirement = stepRequirements[gap];
    return {
      error: { kind: 'uncompilable_step', gap: gapName(requirement.source_text), step: requirement.source_text, span: requirement.source_span },
    };
  }
  const impulseId = impulseIdFor(description);
  const requirements = sourceRequirements.map((requirement, index) => ({
    id: requirementId(impulseId, index + 1, requirement),
    index: index + 1,
    source_text: requirement.source_text,
    source_span: requirement.source_span,
  }));
  const triggerSource = sourceRequirements[triggerPosition];
  const trigger = {
    requirement_id: requirements[triggerPosition].id,
    objects: objectsIn(triggerSource.source_text.toLowerCase(), null),
    source_text: triggerSource.source_text,
    source_span: triggerSource.source_span,
  };
  const steps = classified.map((verb, index) => {
    const requirement = stepRequirements[index];
    const clause = requirement.source_text.toLowerCase();
    return {
      id: '',
      index: index + 1,
      requirement_id: requirements[triggerPosition + index + 1].id,
      kind: verb.slug,
      objects: objectsIn(clause, [verb.start, verb.end]),
      target_language: firstMatch(clause, ROLE_TRANSLATION_LANGUAGE)?.slug ?? null,
      source_text: requirement.source_text,
      source_span: requirement.source_span,
    };
  });
  const canonical = canonicalProgram(trigger, steps);
  const id = stableId('compiled_procedure', canonical);
  for (const step of steps) step.id = stepId(id, step);
  return {
    procedure: {
      id, source_description: description, impulse_id: impulseId, requirements, trigger, steps, canonical_program: canonical,
    },
  };
}

/**
 * Mirrors `compile_procedure(description).ok()`.
 * @param {string} description
 * @returns {object|null} a `CompiledProcedure`
 */
export function compileProcedure(description) {
  return compileProcedureResult(description).procedure ?? null;
}

/**
 * Mirrors `CompiledProcedure::conformance_links_notation`: one walk with the
 * deterministic conformance host (`kind(input)` per step) as a
 * `ProcedureRun::links_notation` record.
 * @param {object} procedure
 * @param {string} triggerValue
 */
export function conformanceLinksNotation(procedure, triggerValue) {
  let input = triggerValue;
  const outcomes = procedure.steps.map((step) => {
    const output = `${step.kind}(${input})`;
    input = output;
    return { step_id: step.id, kind: step.kind, output };
  });
  let out = '';
  out = pushLinoNode(out, 0, 'procedure_run', stableId('procedure_run', `${procedure.id}:${triggerValue}`));
  out = pushLinoNode(out, 2, 'status', 'completed');
  out = pushLinoNode(out, 2, 'package_id', procedure.id);
  out = pushLinoNode(out, 2, 'trigger_value', triggerValue);
  outcomes.forEach((outcome, index) => {
    out = pushLinoNode(out, 2, 'step_outcome', outcome.step_id);
    out = pushLinoNode(out, 4, 'index', String(index + 1));
    out = pushLinoNode(out, 4, 'kind', outcome.kind);
    out = pushLinoNode(out, 4, 'output', outcome.output);
  });
  return pushLinoNode(out, 2, 'answer', outcomes.length ? outcomes[outcomes.length - 1].output : '');
}

/** Mirrors `CompiledProcedure::restate_steps`: each step quoting its source span. */
export function restateSteps(procedure) {
  let out = '';
  for (const step of procedure.steps) {
    const args = stepArguments(step);
    out += `${step.index}. ${step.kind}`;
    if (args.length) out += `(${args.join(', ')})`;
    out += ` — "${step.source_text}" [${step.source_span[0]}..${step.source_span[1]}]\n`;
  }
  return out;
}

/** Mirrors `CompiledProcedure::links_notation`: the canonical, language-independent export. */
export function procedureLinksNotation(procedure) {
  let out = '';
  out = pushLinoNode(out, 0, procedure.id, null);
  out = pushLinoNode(out, 2, 'type', 'compiled_procedure');
  out = pushLinoNode(out, 2, 'schema_version', KNOWLEDGE_SCHEMA_VERSION);
  out = pushLinoNode(out, 2, 'package_kind', 'associative_package');
  out = pushLinoNode(out, 2, 'source', 'natural_language_procedure');
  for (const object of procedure.trigger.objects) out = pushLinoNode(out, 2, 'trigger_object', object);
  for (const step of procedure.steps) {
    out = pushLinoNode(out, 2, 'step', step.id);
    out = pushLinoNode(out, 4, 'index', String(step.index));
    out = pushLinoNode(out, 4, 'kind', step.kind);
    for (const argument of stepArguments(step)) out = pushLinoNode(out, 4, 'argument', argument);
  }
  return out;
}
