// Meanings-driven decomposition of two-sided comparisons (issue #840):
// rust/src/agentic_coding/comparison.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { plainText } from './content.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { detect } from './crate/language.mjs';
import { isAsciiPunctuation, isWhitespace, replaceAllLiteral, splitWhitespace, trim,
  trimMatches } from './crate/rust_str.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { wordsForRole } from './crate/seed_meanings.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { resultCapability } from './progress.mjs';
import { normalizedPayload } from './tool_result.mjs';

const LEFT_SLOT = '{left}';
const RIGHT_SLOT = '{right}';
const EVIDENCE_SLOT = '{evidence}';
const ROLE_COMPARISON_INFIX = 'comparison_infix';
const ROLE_COMPARISON_RHS_MARKER = 'comparison_rhs_marker';
const ROLE_COMPARISON_QUERY_NOISE = 'comparison_query_noise';

const ALPHANUMERIC = /^[\p{Alphabetic}\p{N}]$/u;
const isAlnum = (character) => character !== undefined && ALPHANUMERIC.test(character);
const charCount = (text) => Array.from(text).length;
const byCharsDescending = (left, right) => charCount(right) - charCount(left);

/**
 * Mirrors `fn plan_comparison_step` in rust/src/agentic_coding/comparison.rs.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planComparisonStep(task, messages, toolNames) {
  const sides = comparisonSides(task);
  if (sides === null) return null;
  const [left, right] = sides;
  const evidence = comparisonEvidence(messages);
  const search = toolFor(toolNames, Capability.Search) ?? null;
  if (evidence.length === 0) return search === null ? null : planOne(search, jsonText({ query: left }));
  if (evidence.length === 1) return search === null ? null : planOne(search, jsonText({ query: right }));
  return finalAnswer(comparisonAnswer(task, left, right, evidence));
}

/**
 * Mirrors `fn comparison_sides` in rust/src/agentic_coding/comparison.rs:
 * `[left, right]` or null.
 * @param {string} task
 */
export function comparisonSides(task) {
  const normalized = normalizePrompt(task);
  const markers = [...wordsForRole(ROLE_COMPARISON_INFIX)].sort(byCharsDescending);
  for (const raw of markers) {
    const marker = normalizePrompt(raw);
    const position = boundedFind(normalized, marker);
    if (position === null) continue;
    const before = trim(normalized.slice(0, position));
    const after = trim(normalized.slice(position + marker.length));
    for (const rawRhs of wordsForRole(ROLE_COMPARISON_RHS_MARKER)) {
      const rhsMarker = normalizePrompt(rawRhs);
      const rhsPosition = boundedFind(after, rhsMarker);
      if (rhsPosition === null) continue;
      const left = cleanSide(after.slice(0, rhsPosition));
      const right = cleanSide(after.slice(rhsPosition + rhsMarker.length));
      if (left !== '' && right !== '') return [left, right];
    }
    if (before !== '' && after !== '') return [cleanSide(before), cleanSide(after)];
  }
  return null;
}

/** Mirrors `fn bounded_find` (non-overlapping `match_indices`). */
function boundedFind(text, marker) {
  const step = Math.max(marker.length, 1);
  for (let position = text.indexOf(marker); position >= 0 && position <= text.length;
    position = text.indexOf(marker, position + step)) {
    const before = Array.from(text.slice(0, position)).pop();
    const after = Array.from(text.slice(position + marker.length))[0];
    if (!isAlnum(before) && !isAlnum(after)) return position;
    if (marker === '' && position >= text.length) break;
  }
  return null;
}

/** Mirrors `fn clean_side`. */
function cleanSide(side) {
  let out = trimMatches(side, (character) => isWhitespace(character) || isAsciiPunctuation(character)
    || character === '？' || character === '。' || character === '«' || character === '»');
  const noise = [...wordsForRole(ROLE_COMPARISON_QUERY_NOISE)].sort(byCharsDescending);
  for (const surface of noise) out = removeBoundedSurface(out, normalizePrompt(surface));
  return splitWhitespace(out).join(' ');
}

/** Mirrors `fn remove_bounded_surface` (returns the edited text). */
function removeBoundedSurface(text, surface) {
  if (!surface) return text;
  let out = text;
  let offset = 0;
  for (;;) {
    const start = out.indexOf(surface, offset);
    if (start < 0) return out;
    const end = start + surface.length;
    const left = start === 0 || !isAlnum(Array.from(out.slice(0, start)).pop());
    const right = end === out.length || !isAlnum(Array.from(out.slice(end))[0]);
    if (left && right) {
      out = `${out.slice(0, start)} ${out.slice(end)}`;
      offset = start + 1;
    } else {
      offset = end;
    }
  }
}

/** Mirrors `fn comparison_evidence`. */
function comparisonEvidence(messages) {
  let currentTurn = 0;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role.toLowerCase() === 'user') {
      currentTurn = index + 1;
      break;
    }
  }
  const out = [];
  for (let index = currentTurn; index < messages.length; index += 1) {
    const message = messages[index];
    if (message.role.toLowerCase() !== 'tool') continue;
    if (resultCapability(messages, index) !== Capability.Search) continue;
    const payload = normalizedPayload(plainText(message.content));
    if (payload !== null && payload !== undefined && trim(payload) !== '') out.push(payload);
  }
  return out;
}

/** Mirrors `fn comparison_answer`. */
function comparisonAnswer(task, left, right, evidence) {
  const intent = evidence.length ? 'comparison_decomposed_evidence' : 'comparison_decomposed_no_evidence';
  let text = localizedResponse(intent, detect(task)) ?? '';
  text = replaceAllLiteral(text, LEFT_SLOT, left);
  text = replaceAllLiteral(text, RIGHT_SLOT, right);
  return replaceAllLiteral(text, EVIDENCE_SLOT, evidence.join('\n\n'));
}
