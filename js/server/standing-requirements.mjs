// Retained dreaming amendments applied to live answers:
// rust/src/dreaming_application.rs (`retained_amendments`,
// `solve_with_standing_requirements`, `solve_with_amendment_records`,
// `apply_retained_amendments`, `topic_matches`) and the pre-learned answer
// cache it falls back to, rust/src/anticipation.rs
// (`answer_from_prelearned_cache`).
//
// A stored `meta_algorithm_amendment` whose topic matches the task is fed to
// the solver as a user turn restating the requirement, and its rule is
// projected into the answer with an evidence link back to the record. The
// wording is data (data/meta/server-messages.lino).

import { EventLog, finalizeSimple } from './memory-answer.mjs';
import { serverMessage } from './messages.mjs';

const AMENDMENT_KIND = 'meta_algorithm_amendment';
const ANTICIPATION_SOURCE_KIND = 'anticipation_source';
export const STANDING_REQUIREMENT_INTENT = 'standing_requirement';

/** Mirrors `structured_value`: `key=value` loses its key, anything else is kept. */
function structuredValue(value, key) {
  if (value === null || value === undefined) return null;
  const text = String(value);
  return text.startsWith(`${key}=`) ? text.slice(key.length + 1) : text;
}

/** Mirrors `retained_amendments`: sorted by topic then id, adjacent repeats dropped. */
export function retainedAmendments(events) {
  const amendments = [];
  for (const event of events) {
    if (event.kind !== AMENDMENT_KIND) continue;
    const topic = (structuredValue(event.inputs, 'topic') ?? event.demo_label ?? null);
    const rule = (structuredValue(event.outputs, 'rule') ?? event.content ?? null);
    if (topic === null || rule === null) continue;
    const trimmedTopic = String(topic).trim();
    const trimmedRule = String(rule).trim();
    if (trimmedTopic && trimmedRule) amendments.push({ id: String(event.id), topic: trimmedTopic, rule: trimmedRule });
  }
  const order = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
  amendments.sort((left, right) => order(left.topic, right.topic) || order(left.id, right.id));
  return amendments.filter((amendment, index) => {
    const previous = amendments[index - 1];
    return !previous || previous.topic !== amendment.topic || previous.rule !== amendment.rule;
  });
}

const isAlphanumeric = (character) => /[\p{Alphabetic}\p{N}]/u.test(character);

/** Mirrors `contains_token`: `needle` bounded by non-alphanumerics on both sides. */
function containsToken(text, needle) {
  let start = text.indexOf(needle);
  while (start >= 0) {
    const end = start + needle.length;
    const before = [...text.slice(0, start)].pop();
    const after = [...text.slice(end)][0];
    if ((before === undefined || !isAlphanumeric(before)) && (after === undefined || !isAlphanumeric(after))) return true;
    start = text.indexOf(needle, end);
  }
  return false;
}

/** Mirrors `topic_matches`. */
export function topicMatches(prompt, topic) {
  const text = String(prompt).toLowerCase();
  const wanted = String(topic).toLowerCase();
  if (!wanted) return false;
  if (containsToken(text, wanted)) return true;
  const words = wanted.split(/\s+/u).filter(Boolean);
  return words.length > 1 && words.every((word) => containsToken(text, word));
}

/** Mirrors `matching_amendments`. */
function matchingAmendments(prompt, amendments) {
  return amendments.filter((amendment) => topicMatches(prompt, amendment.topic));
}

/** Mirrors `amendment_lines`. */
function amendmentLines(matching) {
  return matching
    .map((amendment) => serverMessage('standing_requirement_line', { topic: amendment.topic, rule: amendment.rule }))
    .join('\n');
}

/** Mirrors `append_amendments`. */
function appendAmendments(answer, matching) {
  if (!matching.length) return answer;
  return {
    ...answer,
    intent: answer.intent === 'unknown' ? STANDING_REQUIREMENT_INTENT : answer.intent,
    answer: `${answer.answer}\n\n${amendmentLines(matching)}`,
    evidence_links: [...answer.evidence_links, ...matching.map((amendment) => `${AMENDMENT_KIND}:${amendment.id}`)],
  };
}

/** Mirrors `apply_retained_amendments` (returns the amended answer). */
export function applyRetainedAmendments(prompt, answer, events) {
  return appendAmendments(answer, matchingAmendments(prompt, retainedAmendments(events)));
}

/**
 * Mirrors `solve_with_amendment_records`: each matching requirement becomes a
 * user turn ahead of the history, then the answer carries the rules.
 * @param {(prompt: string, history: Array<object>) => Promise<object>} solve
 */
export async function solveWithAmendmentRecords(solve, prompt, history, amendments) {
  const matching = matchingAmendments(prompt, amendments);
  const turns = matching.map((amendment) => ({
    role: 'user',
    content: serverMessage('standing_requirement_turn', { topic: amendment.topic, rule: amendment.rule }),
  }));
  return appendAmendments(await solve(prompt, [...turns, ...history]), matching);
}

/** Mirrors `content_field` in rust/src/anticipation.rs. */
function contentField(content, key) {
  if (content === null || content === undefined) return null;
  for (const raw of String(content).split('\n')) {
    const line = raw.trim();
    const space = line.indexOf(' ');
    if (space < 0 || line.slice(0, space) !== key) continue;
    let value = line.slice(space + 1).trim();
    if (value.length >= 2 && value.startsWith('"') && value.endsWith('"')) value = value.slice(1, -1);
    return value.split('""').join('"');
  }
  return null;
}

/**
 * Mirrors rust/src/anticipation.rs `answer_from_prelearned_cache_at`.
 * @param {(text: string) => string} normalizePrompt the seed normalizer
 */
export function answerFromPrelearnedCache(normalizePrompt, prompt, events, now = Math.floor(Date.now() / 1000)) {
  const normalized = normalizePrompt(prompt);
  const event = [...events].reverse().find((candidate) => {
    if (candidate.kind !== ANTICIPATION_SOURCE_KIND) return false;
    if (candidate.inputs === null || candidate.inputs === undefined) return false;
    if (normalizePrompt(candidate.inputs) !== normalized) return false;
    const expires = contentField(candidate.content, 'expires_at');
    return expires !== null && /^\+?\d+$/.test(expires) && now <= Number(expires);
  });
  if (!event || event.outputs === null || event.outputs === undefined) return null;
  const log = new EventLog();
  log.append('anticipation_prediction', contentField(event.content, 'prediction') ?? '');
  log.append('source:http', contentField(event.content, 'source_trace') ?? '');
  log.append('cache_hit', contentField(event.content, 'source_url') ?? '');
  return finalizeSimple(prompt, log, 'anticipation_cache', 'response:anticipation_cache', String(event.outputs), 1);
}

/**
 * Mirrors `solve_with_standing_requirements`.
 * @param {(prompt: string, history: Array<object>) => Promise<object>} solve
 * @param {(text: string) => string} normalizePrompt
 */
export async function solveWithStandingRequirements(solve, normalizePrompt, prompt, history, events) {
  const answer = await solveWithAmendmentRecords(solve, prompt, history, retainedAmendments(events));
  if (answer.intent !== 'unknown') return answer;
  return answerFromPrelearnedCache(normalizePrompt, prompt, events) ?? answer;
}
