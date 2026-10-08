// A file operation followed by edits of the file it makes (PR #1188 G82):
// `Copy a.lino to b.lino. In b.lino replace 'x' with 'y', replace 'p' with
// 'q'.` was written as one literal file. When the first sentence is a seeded
// shell intent (copy, move) and every later sentence is an edit request whose
// file that sentence names, the sentences are planned one after another, each
// as the request it would be alone, over the same tool history; the answer
// states each in turn. Edits joined by a seeded sequence cue (`, then`, `and
// then`) that opens a clause are steps too (PR #1188 G99): `Insert the line
// «b2» after the line «b» in f.txt, then delete the line «c» from f.txt.`
// Each step sees only the tool calls made for it, so a later step reads the
// file as the step before left it. The JavaScript twin of
// rust/src/agentic_coding/request_sequence.rs.

import { finalAnswer } from './plan.mjs';
import { pathsIn } from './module_function.mjs';
import { replaceList } from './replace_list.mjs';
import { semanticShellCommandForTask } from './shell_command.mjs';
import { sentences } from './shell_command_policy.mjs';
import { instructionEnd } from './positional_edit.mjs';
import { containsCjk } from './crate/coding_catalog.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { isAlphanumeric, isWhitespace } from './write_str.mjs';
import {
  bareSurfaces, cleanCueToken, cleanPathToken, composeEditRequest, looksLikeFilePath, safeRelativePath, tokens,
} from './write_request.mjs';

/** The marks that close a clause before a sequence cue. */
const CLAUSE_MARKS = ',;.，；。';

/**
 * Mirrors `fn request_sequence`: the request's sentences when the first is a
 * seeded shell intent and every later one edits a file the first names, or
 * null.
 * @param {string} task
 */
export function requestSequence(task) {
  const parts = sentences(task).map((sentence) => sentence.text);
  if (parts.length < 2 || semanticShellCommandForTask(parts[0]) === null) return null;
  const named = pathsIn(parts[0]);
  for (const part of parts.slice(1)) {
    const target = replaceList(part)?.target ?? composeEditRequest(part)?.[0] ?? null;
    if (target === null || !named.includes(target)) return null;
  }
  return parts;
}

/**
 * Mirrors `fn sequence_cut`: where the step before a sequence cue at `at`
 * ends -- at the clause mark or the seeded joiner word (`and`) that opens the
 * cue's clause -- or null when the cue does not open a clause.
 * @param {string} task
 * @param {number} at
 */
function sequenceCut(task, at) {
  const lead = task.slice(0, at).trimEnd();
  if (lead === '') return null;
  if (CLAUSE_MARKS.includes(lead.at(-1))) return lead.length - 1;
  const word = lead.split(/\s+/u).at(-1);
  if (!bareSurfaces('file_edit_joiner_cue').includes(cleanCueToken(word))) return null;
  return lead.length - word.length;
}

/**
 * Mirrors `fn sequence_cues`: each unquoted seeded sequence cue of the
 * instruction that opens a clause, as `[stepEnd, nextStart]`.
 * @param {string} task
 */
function sequenceCues(task) {
  const end = instructionEnd(task);
  const segments = quotedSegmentSpans(task);
  const quoted = (at) => segments.some((segment) => at >= segment.start && at < segment.end);
  const cues = bareSurfaces('file-edit-sequence-cue').sort((a, b) => b.length - a.length);
  const out = [];
  let at = 0;
  while (at < end) {
    const cue = quoted(at) ? undefined : cues.find((surface) => task.slice(at, at + surface.length).toLowerCase() === surface
      && (containsCjk(surface) || (!isAlphanumeric(task[at - 1]) && !isAlphanumeric(task[at + surface.length]))));
    const cut = cue === undefined ? null : sequenceCut(task, at);
    if (cut === null) {
      at += 1;
      continue;
    }
    let next = at + cue.length;
    while (next < task.length && (isWhitespace(task[next]) || CLAUSE_MARKS.includes(task[next]))) next += 1;
    out.push([cut, next]);
    at = next;
  }
  return out;
}

/** The paths `step` names outside its quotes. */
function unquotedPaths(step) {
  const segments = quotedSegmentSpans(step);
  return tokens(step)
    .filter((token) => !segments.some((segment) => token.start < segment.end && token.end > segment.start))
    .map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path));
}

/**
 * Mirrors `fn sequence_steps`: the request cut at its sequence cues into
 * steps that each quote a text and name a file (a step naming none takes the
 * one file the others name), or null (PR #1188 G99).
 * @param {string} task
 */
export function sequenceSteps(task) {
  const cues = sequenceCues(task);
  if (cues.length === 0) return null;
  const bounds = [0, ...cues.flat(), task.length];
  const steps = [];
  for (let index = 0; index < bounds.length; index += 2) {
    steps.push(task.slice(bounds[index], bounds[index + 1]).trim().replace(/[\s,;，；.。]+$/u, ''));
  }
  if (steps.some((step) => step === '' || quotedSegmentSpans(step).length === 0)) return null;
  const named = [...new Set(steps.flatMap(unquotedPaths))];
  return steps.map((step) => {
    if (unquotedPaths(step).length > 0) return step;
    // A step naming no path, not even inside its quotes, edits the one file
    // the others name.
    if (named.length !== 1 || pathsIn(step).length > 0) return null;
    return `${step} in ${named[0]}`;
  }).reduce((all, step) => (all === null || step === null ? null : [...all, step]), []);
}

/** The conversation with its latest user turn asking `part` alone. */
function withRequest(messages, part) {
  let latest = -1;
  messages.forEach((message, index) => {
    if (String(message.role).toLowerCase() === 'user') latest = index;
  });
  return messages.map((message, index) => (index === latest ? { ...message, content: part } : message));
}

/**
 * Mirrors `fn plan_request_sequence_step`: the next step of the first
 * sentence not yet answered, planned by `planFor` as that sentence alone, or
 * the sentences' answers in order once each is answered.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {(messages: Array<object>, toolNames: Array<string>) => Promise<object|null>} planFor
 */
export async function planRequestSequenceStep(task, messages, toolNames, planFor) {
  const parts = requestSequence(task) ?? sequenceSteps(task);
  if (parts === null) return null;
  const { base, exchanges } = turnExchanges(messages);
  let taken = 0;
  const answers = [];
  for (const part of parts) {
    // A step is planned over the tool calls made for it alone: replayed one
    // exchange at a time until it answers or asks for its next call.
    const own = [];
    for (;;) {
      const plan = await planFor(withRequest([...base, ...own], part), toolNames);
      if (plan === null) return null;
      if (plan.kind === 'final') {
        answers.push(plan.answer);
        break;
      }
      if (taken === exchanges.length) return plan;
      own.push(...exchanges[taken]);
      taken += 1;
    }
  }
  return finalAnswer(answers.join('\n\n'));
}

/**
 * Mirrors `fn turn_exchanges`: the conversation up to its latest user turn,
 * and the tool exchanges after it, each an assistant message with the
 * messages that answer it.
 * @param {Array<object>} messages
 */
function turnExchanges(messages) {
  let latest = -1;
  messages.forEach((message, index) => {
    if (String(message.role).toLowerCase() === 'user') latest = index;
  });
  const exchanges = [];
  for (const message of messages.slice(latest + 1)) {
    if (String(message.role).toLowerCase() === 'assistant' || exchanges.length === 0) exchanges.push([message]);
    else exchanges.at(-1).push(message);
  }
  return { base: messages.slice(0, latest + 1), exchanges };
}
