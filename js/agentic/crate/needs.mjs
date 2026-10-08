// The need record of plan 00 section 4.1 (rust/src/needs.rs): one thing the
// system does not know, with its Links Notation projection.

import { stableId } from './engine_stable_identifier.mjs';
import { trim } from './rust_str.mjs';

/** Mirrors `enum NeedKind` (`NeedKind::slug` values). */
export const NeedKind = Object.freeze({
  Concept: 'concept', Procedure: 'procedure', Part: 'part', Prerequisite: 'prerequisite',
  Evidence: 'evidence', Decision: 'decision', None: 'none',
});

/** Mirrors `enum NeedState` (`NeedState::slug` values). */
export const NeedState = Object.freeze({
  Open: 'open', Planned: 'planned', Satisfied: 'satisfied', Unsatisfiable: 'unsatisfiable',
});

/**
 * Mirrors `Need::raised`: a freshly raised need of one kind about one subject.
 * @param {string} kind
 * @param {string} subject
 * @param {string} language
 * @param {string} raisedBy
 */
export function raisedNeed(kind, subject, language, raisedBy) {
  return {
    need_id: stableId('need', `${kind}|${trim(subject)}|${trim(language)}`),
    kind,
    subject: trim(subject),
    language: trim(language),
    raised_by: trim(raisedBy),
    source_span: '',
    depth: 0,
    state: NeedState.Open,
    satisfied_by: null,
  };
}

/**
 * One `field value` line of a need record.
 * @param {string} field
 * @param {string} value
 * @returns {string}
 */
const bare = (field, value) => `  ${field} ${value}\n`;
/** @param {string} value */
const escape = (value) => value.replaceAll('\\', '\\\\').replaceAll('"', '\\"');
const quoted = (field, value) => `  ${field} "${escape(value)}"\n`;

/** Mirrors `Need::to_links_notation`. */
export function needLinksNotation(need) {
  let text = `need ${need.need_id}\n`;
  text += bare('kind', need.kind);
  text += quoted('subject', need.subject);
  text += bare('language', need.language);
  text += bare('raised_by', need.raised_by);
  text += quoted('source_span', need.source_span);
  text += bare('depth', String(need.depth));
  text += bare('state', need.state);
  if (need.satisfied_by !== null && need.satisfied_by !== undefined) text += bare('satisfied_by', need.satisfied_by);
  return text;
}
