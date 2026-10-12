// The lineage route's claim and answer: a port of
// rust/src/history_context/lineage.rs (issue #1180 R10, R11).
//
// `lineageSubjects` is the claim half (a seeded lineage cue plus a
// path-shaped token), `lineageAnswer` the render half over the commits
// `git log --follow` yields, formalized by `formalizeCommit`. Running git
// stays with the native route; a JavaScript caller hands the formalized
// commits in.

import { localizedResponse } from './seed.mjs';

const NUMBER_PLACEHOLDER = '%number%';
const LINEAGE_INTENT = 'repository_lineage';
const UNRECORDED_INTENT = 'repository_lineage_unrecorded';

/** Mirrors `is_path_char`. */
const isPathChar = (ch) => /^[A-Za-z0-9/._-]$/u.test(ch);

/** Rust's `trim_end_matches(ch)` / `trim_start_matches(ch)` for one char. */
function trimEndChar(text, ch) {
  let end = text.length;
  while (end > 0 && text[end - 1] === ch) end -= 1;
  return text.slice(0, end);
}
function trimStartChar(text, ch) {
  let start = 0;
  while (start < text.length && text[start] === ch) start += 1;
  return text.slice(start);
}

/**
 * Mirrors `lineage_subjects`: the repository paths a cued lineage question
 * names, in prompt order; empty without a cue.
 * @param {string} prompt
 * @param {object} rules the history rules (`lineageCues`)
 */
export function lineageSubjects(prompt, rules) {
  const lowered = String(prompt).toLowerCase();
  const cued = rules.lineageCues.some(([, phrases]) => phrases.some((phrase) => lowered.includes(phrase.toLowerCase())));
  if (!cued) return [];
  const subjects = [];
  let current = '';
  for (const ch of [...String(prompt), ' ']) {
    if (isPathChar(ch)) {
      current += ch;
      continue;
    }
    const run = trimStartChar(trimEndChar(current, '.'), '-');
    const shaped = run.includes('/') || run.includes('.');
    if (shaped && !run.startsWith('/') && !run.split('/').includes('..') && !subjects.includes(run)) {
      subjects.push(run);
    }
    current = '';
  }
  return subjects;
}

/** Mirrors `pattern_value`: the number a seed pattern carries in `value`. */
function patternValue(value, pattern) {
  const at = pattern.indexOf(NUMBER_PLACEHOLDER);
  if (at === -1 || typeof value !== 'string') return null;
  const prefix = pattern.slice(0, at);
  const suffix = pattern.slice(at + NUMBER_PLACEHOLDER.length);
  if (!value.startsWith(prefix) || !value.endsWith(suffix) || value.length < prefix.length + suffix.length) return null;
  const number = value.slice(prefix.length, value.length - suffix.length);
  return /^[0-9]+$/u.test(number) ? number : null;
}

function template(intent, language, values) {
  let out = localizedResponse(intent, language) ?? '';
  for (const [key, value] of values) out = out.split(`{${key}}`).join(value);
  return out;
}

/**
 * Mirrors `lineage_answer`: the lineage of `path` from its formalized
 * commits (chronological), or null when it has none.
 * @param {string} language
 * @param {string} path
 * @param {Array<object>} events commit events as `formalizeCommit` returns them
 * @param {object} rules
 */
export function lineageAnswer(language, path, events, rules) {
  const first = events[0];
  if (!first) return null;
  const commitPrefix = rules.records.find((record) => record.kind === 'commit')?.idPrefix ?? '';
  const short = (event) => {
    const sha = event.id.startsWith(commitPrefix) ? event.id.slice(commitPrefix.length) : event.id;
    return [...sha].slice(0, 9).join('');
  };
  const commits = events.map((event) => {
    const date = [...(event.sentAt ?? '')].slice(0, 10).join('');
    const subject = String(event.content ?? '').split('\n')[0] ?? '';
    return `- ${short(event)} ${date} ${subject}`;
  });
  let issue = null;
  for (const trailer of rules.trailers) {
    issue = patternValue(first.conversationId, trailer.conversation);
    if (issue !== null) break;
  }
  let pull = null;
  for (const evidence of first.evidence) {
    pull = patternValue(evidence, rules.merge.evidence);
    if (pull !== null) break;
  }
  const unrecorded = template(UNRECORDED_INTENT, language, []);
  const numbered = (number) => (number === null ? unrecorded : `#${number}`);
  return template(LINEAGE_INTENT, language, [
    ['path', path],
    ['count', String(events.length)],
    ['commits', commits.join('\n')],
    ['first', short(first)],
    ['issue', numbered(issue)],
    ['pull', numbered(pull)],
  ]);
}
