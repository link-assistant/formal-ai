// Data-driven request-class expansion for anticipatory dreaming: the twin of
// rust/src/anticipation/expansion.rs (`expand_class`, `expand_meanings`,
// `expand_operations`, `push_variant`, `template_subject`,
// `instantiate_variant`, `surface_present`, `replace_surface`).
//
// A predicted class is expanded into prompt variants from evidence only: the
// observed members of the class (parameter variants), the seeded operation
// vocabulary's other phrasings of an operation the prompt names, and the
// seeded meaning lexicon's other word forms of a meaning the prompt uses.

import { normalizePrompt } from '../agentic/crate/engine.mjs';
import { detect } from '../agentic/crate/language.mjs';
import { afterSlot, beforeSlot, lexicon, slotOf } from '../agentic/crate/seed_meanings.mjs';
import { operationMatches, operationVocabulary } from '../agentic/crate/seed_operation_vocabulary.mjs';

const ALPHANUMERIC = /[\p{Alphabetic}\p{N}]/u;

/** Mirrors `surface_present` in rust/src/anticipation/expansion.rs: a non-overlapping match bounded by non-alphanumerics. */
export function surfacePresent(haystack, needle) {
  if (!needle) return false;
  for (let start = haystack.indexOf(needle); start >= 0; start = haystack.indexOf(needle, start + needle.length)) {
    const left = Array.from(haystack.slice(0, start)).at(-1);
    const right = Array.from(haystack.slice(start + needle.length))[0];
    if ((left === undefined || !ALPHANUMERIC.test(left)) && (right === undefined || !ALPHANUMERIC.test(right))) return true;
  }
  return false;
}

/** Mirrors `replace_surface`: the first occurrence replaced, else the haystack. */
export function replaceSurface(haystack, needle, replacement) {
  const start = haystack.indexOf(needle);
  return start < 0 ? haystack : `${haystack.slice(0, start)}${replacement}${haystack.slice(start + needle.length)}`;
}

/** Mirrors `push_variant`. */
function pushVariant(state, prompt, source, baseEventId) {
  const normalized = normalizePrompt(prompt);
  if (!normalized || state.variants.length >= state.limit || state.seen.has(normalized)) return;
  state.seen.add(normalized);
  state.variants.push({ prompt: prompt.trim(), source, base_event_id: baseEventId });
}

const nonEmpty = (subject) => (subject !== '' ? subject : null);

/** Mirrors `template_subject`: `{slot: subject}` or `{bare: needle}`, or null. */
function templateSubject(form, prompt) {
  const before = normalizePrompt(beforeSlot(form));
  const after = normalizePrompt(afterSlot(form));
  const slot = slotOf(form);
  let subject = null;
  if (slot === 'prefix') {
    if (prompt.startsWith(before)) subject = nonEmpty(prompt.slice(before.length).trim());
  } else if (slot === 'suffix') {
    if (prompt.endsWith(after)) subject = nonEmpty(prompt.slice(0, prompt.length - after.length).trim());
  } else if (slot === 'circumfix') {
    if (prompt.startsWith(before)) {
      const rest = prompt.slice(before.length);
      if (rest.endsWith(after)) subject = nonEmpty(rest.slice(0, rest.length - after.length).trim());
    }
  } else {
    return surfacePresent(prompt, before) ? { bare: before } : null;
  }
  return subject === null ? null : { slot: subject };
}

/** Mirrors `instantiate_variant`. */
function instantiateVariant(matched, replacement, prompt, subject) {
  if (subject.bare !== undefined) return replaceSurface(prompt, subject.bare, normalizePrompt(replacement.text));
  const before = normalizePrompt(beforeSlot(replacement));
  const after = normalizePrompt(afterSlot(replacement));
  switch (slotOf(replacement)) {
    case 'prefix':
      return [before, subject.slot].join(' ');
    case 'suffix':
      return [subject.slot, after].join(' ');
    case 'circumfix':
      return [before, subject.slot, after].join(' ');
    default:
      return replaceSurface(prompt, normalizePrompt(matched.text.replaceAll('…', subject.slot)), normalizePrompt(replacement.text));
  }
}

/** Mirrors `expand_meanings`. */
function expandMeanings(observation, state) {
  const normalized = normalizePrompt(observation.prompt);
  const language = detect(observation.prompt);
  for (const meaning of lexicon()) {
    const forms = meaning.lexemes.filter((lexeme) => lexeme.language === language).flatMap((lexeme) => lexeme.words);
    for (const matched of forms) {
      const subject = templateSubject(matched, normalized);
      if (subject === null) continue;
      for (const replacement of forms) {
        if (replacement.text === matched.text) continue;
        pushVariant(state, instantiateVariant(matched, replacement, normalized, subject), `meaning:${meaning.slug}`, observation.event_id);
        if (state.variants.length >= state.limit) return;
      }
    }
  }
}

/** Mirrors `expand_operations`. */
function expandOperations(observation, state) {
  const normalized = normalizePrompt(observation.prompt);
  const language = detect(observation.prompt);
  for (const operation of operationVocabulary().filter((candidate) => operationMatches(candidate, normalized))) {
    const forms = operation.languages.get(language);
    if (!forms) continue;
    for (const phrase of forms.phrases) {
      const needle = normalizePrompt(phrase);
      if (!surfacePresent(normalized, needle)) continue;
      for (const replacement of forms.phrases) {
        pushVariant(state, replaceSurface(normalized, needle, normalizePrompt(replacement)), `operation:${operation.canonical}`, observation.event_id);
        if (state.variants.length >= state.limit) return;
      }
    }
  }
}

/**
 * Mirrors `expand_class` in rust/src/anticipation/expansion.rs.
 * @param {{id: string}} intentClass
 * @param {Array<{event_id: string, prompt: string, class: {id: string}}>} observations
 * @param {{max_variations_per_prediction: number}} config
 * @returns {Array<{prompt: string, source: string, base_event_id: string}>}
 */
export function expandClass(intentClass, observations, config) {
  const state = { variants: [], seen: new Set(), limit: config.max_variations_per_prediction };
  for (const observation of observations.filter((candidate) => candidate.class.id === intentClass.id)) {
    pushVariant(state, observation.prompt, `parameter:${observation.event_id}`, observation.event_id);
    expandOperations(observation, state);
    expandMeanings(observation, state);
    if (state.variants.length >= state.limit) break;
  }
  return state.variants;
}
