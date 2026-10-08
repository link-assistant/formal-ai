// Status and definition questions about the repository: a port of the pure
// half of rust/src/history_context/repository_qa.rs (issue #1180 R10, R11).
//
// The claim halves (`statusSubject`, `definitionSubjects`), the census reader
// (`censusDocumentSymbols`), the documentation-comment reader (`symbolDoc`) and
// both renders. Running git and walking the census directory stay with the
// native route; a JavaScript caller hands in the census text, the source, the
// formalized commits and the measured range.

import { localizedResponse } from './seed.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';

const NUMBER_PLACEHOLDER = '%number%';

function template(intent, language, values) {
  let out = localizedResponse(intent, language) ?? '';
  for (const [key, value] of values) out = out.split(`{${key}}`).join(value);
  return out;
}

/** Mirrors `cued_phrase`. */
function cuedPhrase(prompt, cues) {
  const lowered = String(prompt).toLowerCase();
  for (const [, phrases] of cues) {
    for (const phrase of phrases) if (lowered.includes(phrase.toLowerCase())) return phrase;
  }
  return null;
}

/** Mirrors `status_subject`. */
export function statusSubject(prompt, rules) {
  const qa = rules.repositoryQa;
  return qa.statusScript ? cuedPhrase(prompt, qa.statusCues) : null;
}

/** Mirrors `identifier_shaped`. */
function identifierShaped(token) {
  const startsLikeIdentifier = /^[A-Za-z_]/u.test(token);
  return startsLikeIdentifier && token.length > 1 && (token.includes('_') || /[a-z][A-Z]/u.test(token));
}

/** Mirrors `definition_subjects`. */
export function definitionSubjects(prompt, rules) {
  if (cuedPhrase(prompt, rules.repositoryQa.definitionCues) === null) return [];
  const names = [];
  for (const token of String(prompt).split(/[^A-Za-z0-9_]/u)) {
    if (identifierShaped(token) && !names.includes(token)) names.push(token);
  }
  return names;
}

/** Mirrors `census_document_symbols`. */
export function censusDocumentSymbols(text) {
  let target = '';
  let inSymbols = false;
  const symbols = [];
  for (const line of String(text).split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith('target ')) {
      target = trimmed.slice('target '.length).trim();
      continue;
    }
    if (trimmed === 'symbols') {
      inSymbols = true;
      continue;
    }
    if (!inSymbols) continue;
    const fields = trimmed.split(/\s+/u);
    if (fields.length === 4 && /^\d+$/u.test(fields[2]) && /^\d+$/u.test(fields[3])) {
      symbols.push({ target, kind: fields[0], name: fields[1], start: Number(fields[2]), end: Number(fields[3]) });
    }
  }
  return symbols;
}

/** Mirrors `symbol_doc`. */
export function symbolDoc(source, start, rules) {
  const qa = rules.repositoryQa;
  if (!qa.docPrefix || start < 2) return null;
  const lines = String(source).split('\n').slice(0, start - 1);
  const comment = [];
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const trimmed = lines[index].trim();
    if (trimmed.startsWith(qa.docPrefix)) {
      comment.push(trimmed.slice(qa.docPrefix.length).trim());
    } else if (qa.attributePrefix && trimmed.startsWith(qa.attributePrefix)) {
      if (comment.length > 0) break;
    } else {
      break;
    }
  }
  comment.reverse();
  const paragraph = [];
  for (const line of comment) {
    if (line === '') break;
    paragraph.push(line);
  }
  return paragraph.length ? paragraph.join(' ') : null;
}

/** Mirrors `pattern_value`. */
function patternValue(value, pattern) {
  const at = pattern.indexOf(NUMBER_PLACEHOLDER);
  if (at === -1 || typeof value !== 'string') return null;
  const prefix = pattern.slice(0, at);
  const suffix = pattern.slice(at + NUMBER_PLACEHOLDER.length);
  if (!value.startsWith(prefix) || !value.endsWith(suffix) || value.length < prefix.length + suffix.length) return null;
  const number = value.slice(prefix.length, value.length - suffix.length);
  return /^[0-9]+$/u.test(number) ? number : null;
}

/** Mirrors `issue_of`. */
function issueOf(first, rules, language) {
  if (first) {
    for (const trailer of rules.trailers) {
      const issue = patternValue(first.conversationId, trailer.conversation);
      if (issue !== null) return `#${issue}`;
    }
  }
  return template('repository_lineage_unrecorded', language, []);
}

/** Mirrors `short_sha`. */
function shortSha(event, rules) {
  const prefix = rules.records.find((record) => record.kind === 'commit')?.idPrefix ?? '';
  const sha = event.id.startsWith(prefix) ? event.id.slice(prefix.length) : event.id;
  return [...sha].slice(0, 9).join('');
}

/**
 * Mirrors `definition_answer`, with the symbol's source and the `git log -S`
 * commits handed in.
 */
export function definitionAnswer(symbol, source, events, language, rules) {
  const qa = rules.repositoryQa;
  const file = qa.sourceRoot ? `${qa.sourceRoot}/${symbol.target}` : symbol.target;
  const doc = source === null ? null : symbolDoc(source, symbol.start, rules);
  const docText = doc === null
    ? template('repository_definition_undocumented', language, [])
    : template('repository_definition_documented', language, [['doc', doc]]);
  const first = events[0];
  return template('repository_definition', language, [
    ['name', symbol.name],
    ['kind', symbol.kind],
    ['file', file],
    ['start', String(symbol.start)],
    ['end', String(symbol.end)],
    ['doc', docText],
    ['first', first ? shortSha(first, rules) : template('repository_lineage_unrecorded', language, [])],
    ['issue', issueOf(first, rules, language)],
  ]);
}

/** Mirrors `percentage`. */
function percentage(basisPoints) {
  if (!/^\d+$/u.test(basisPoints)) return null;
  const value = Number(basisPoints);
  return `${Math.floor(value / 100)}.${String(value % 100).padStart(2, '0')}%`;
}

/** Mirrors `capitalized`. */
function capitalized(text) {
  const [first, ...rest] = [...text];
  return first === undefined ? '' : first.toUpperCase() + rest.join('');
}

/**
 * Mirrors the render half of `status_answer`: `measured` is `{tag, commits,
 * sessions}` as git reported them, `events` the script's `git log --follow`
 * commits.
 */
export function statusAnswer(status, measured, ledgerText, events, language, rules) {
  const qa = rules.repositoryQa;
  const ledger = (parseRoot(ledgerText).children || [])[0];
  if (!ledger) return null;
  const newest = [...(ledger.children || [])]
    .reverse()
    .find((row) => row.name === 'release' && findChildValue(row, 'target_percentage_basis_points'));
  if (!newest) return null;
  const target = percentage(findChildValue(newest, 'target_percentage_basis_points'));
  if (target === null) return null;
  return capitalized(template('repository_status_explanation', language, [
    ['status', status],
    ['script', qa.statusScript],
    ['issue', issueOf(events[0], rules, language)],
    ['range', `${measured.tag}..HEAD`],
    ['commits', String(measured.commits)],
    ['sessions', String(measured.sessions)],
    ['trailer', findChildValue(ledger, 'session_trailer')],
    ['floor', findChildValue(ledger, 'release_cycle_floor')],
    ['unit', findChildValue(ledger, 'release_cycle_unit')],
    ['attribution', findChildValue(ledger, 'release_cycle_attribution')],
    ['target', target],
    ['target_tag', findChildValue(newest, 'tag')],
    ['policy', findChildValue(ledger, 'target_policy')],
    ['ledger', qa.statusLedger],
  ]));
}
