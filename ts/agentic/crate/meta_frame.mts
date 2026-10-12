// The problem frame, the recursive work-unit tree and the need ledger the meta
// core records for every request (R330, R332, R333; R1013 server parity):
// rust/src/meta_frame.rs (`ProblemFrame`, `WorkUnit`, `NeedLedger` and their
// `record_*` functions) with the balanced splitter it consults,
// rust/src/selection_heuristics/splitting.rs (`balanced_split`,
// `complete_layers`).
//
// Spans are UTF-16 offsets here where Rust keeps UTF-8 byte offsets; only the
// span texts reach a record, so the projection is the same.

import { debugOption, stableId } from './engine_stable_identifier.mjs';
import { containsCjk } from './coding_catalog.mjs';
import { formatLinoRecord } from './links_format.mjs';
import { formalizeIntentRecord } from './intent_formalization.mjs';
import { orderedRequirementSpans, requirementListSpans, requirementOperandSpans } from './intent_formalization_requirements.mjs';
import { heuristicsFor, methodRegistry } from './method_registry.mjs';
import { mentionsRole, mentionsRoleRaw, wordsForRole } from './seed_meanings.mjs';
import { formalizePrompt } from './translation_formalization.mjs';
import { normalizePrompt as webNormalizePrompt } from './web_engine_core.mjs';

const ROLE_CLAUSE_CONTINUATION_MARKER = 'clause_continuation_marker';
const ROLE_OBSERVABLE_TASK_ACTION = 'observable_task_action';
const ROLE_SOFTWARE_AUTHORING_ACTION = 'software_authoring_action';
const ROLE_FOLLOWUP_INSTRUCTION_VERB = 'followup_instruction_verb';
const ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR = 'skill_procedure_clause_separator';
const OBLIGATION_ROLES = [ROLE_OBSERVABLE_TASK_ACTION, ROLE_SOFTWARE_AUTHORING_ACTION, ROLE_FOLLOWUP_INSTRUCTION_VERB];
const STRONG_SENTENCE_TERMINATORS = new Set(['?', '!', '。', '！', '？', '।', '॥']);
const NEED_TERMINATORS = new Set(['?', '!', '。', '！', '？']);
const CLAUSE_PUNCTUATION = new Set([',', ';', '，', '；', '、']);
const isWhitespace = (character) => character !== undefined && /^\s$/u.test(character);

/** `max_by_key`: the last element with the greatest key, or null. */
function lastMaxBy(items, key) {
  let best = null;
  for (const item of items) if (best === null || key(item) >= key(best)) best = item;
  return best;
}

/** Mirrors `fn formalize_span` in rust/src/meta_frame.rs. */
function formalizeSpan(span, language) {
  return formalizeIntentRecord(span, language, formalizePrompt(span, language));
}

// ------------------------------------------------------------ splitting.rs

/** Mirrors `fn sentence_spans` in rust/src/selection_heuristics/splitting.rs. */
function sentenceSpans(task) {
  const spans = [];
  let start = 0;
  const characters = Array.from(task);
  let index = 0;
  characters.forEach((character, position) => {
    const end = index + character.length;
    const strong = STRONG_SENTENCE_TERMINATORS.has(character);
    const next = characters[position + 1];
    const period = character === '.' && (next === undefined || isWhitespace(next));
    if (strong || period) {
      spans.push([start, end]);
      start = end;
    }
    index = end;
  });
  if (start < task.length) spans.push([start, task.length]);
  return spans.filter(([from, to]) => task.slice(from, to).trim() !== '');
}

/** Mirrors `fn joins_two_numbers` in rust/src/selection_heuristics/splitting.rs. */
function joinsTwoNumbers(text, start, end) {
  const before = text.slice(0, start).split(/\s+/u).filter(Boolean);
  const after = text.slice(end).split(/\s+/u).filter(Boolean);
  const digit = (token) => token !== undefined && /[0-9]/.test(token);
  return digit(before[before.length - 1]) && digit(after[0]);
}

/** Mirrors `fn marker_boundaries` in rust/src/selection_heuristics/splitting.rs. */
function markerBoundaries(text, marker, base) {
  const offsets = [];
  if (!marker) return offsets;
  const substring = containsCjk(marker);
  let search = 0;
  for (let at = text.indexOf(marker, search); at >= 0; at = text.indexOf(marker, search)) {
    search = at + marker.length;
    if (substring) {
      offsets.push(base + at);
      continue;
    }
    const before = Array.from(text.slice(0, at)).pop();
    const after = at + marker.length;
    const beforeBoundary = at === 0 || isWhitespace(before);
    const afterBoundary = after >= text.length || isWhitespace(Array.from(text.slice(after))[0]);
    if (beforeBoundary && afterBoundary && at > 0 && !joinsTwoNumbers(text, at, after)) offsets.push(base + at);
  }
  return offsets;
}

/** Mirrors `fn clause_spans` in rust/src/selection_heuristics/splitting.rs. */
function clauseSpans(task, [start, end]) {
  const text = task.slice(start, end);
  const boundaries = [start];
  let offset = 0;
  for (const character of text) {
    offset += character.length;
    if (CLAUSE_PUNCTUATION.has(character)) boundaries.push(start + offset);
  }
  for (const marker of wordsForRole(ROLE_CLAUSE_CONTINUATION_MARKER)) boundaries.push(...markerBoundaries(text, marker, start));
  boundaries.push(end);
  const sorted = [...new Set(boundaries)].sort((left, right) => left - right);
  const spans = [];
  for (let index = 0; index + 1 < sorted.length; index += 1) {
    if (task.slice(sorted[index], sorted[index + 1]).trim() !== '') spans.push([sorted[index], sorted[index + 1]]);
  }
  return spans;
}

/** Mirrors `fn raw_spans` in rust/src/selection_heuristics/splitting.rs. */
function rawSpans(task) {
  const sentences = sentenceSpans(task);
  if (sentences.length > 1) return [sentences, true];
  return [clauseSpans(task, [0, task.length]), false];
}

/** Mirrors `fn head_action` in rust/src/selection_heuristics/splitting.rs. */
function headAction(segment) {
  if (containsCjk(segment)) return null;
  const first = segment.split(/\s+/u).filter(Boolean)[0];
  if (first === undefined) return null;
  return mentionsRole(ROLE_OBSERVABLE_TASK_ACTION, webNormalizePrompt(first)) ? first : null;
}

/** Mirrors `fn evidences_an_obligation` in rust/src/selection_heuristics/splitting.rs. */
function evidencesAnObligation(text) {
  const normalized = webNormalizePrompt(text);
  return OBLIGATION_ROLES.some((role) => mentionsRole(role, normalized) || mentionsRoleRaw(role, normalized));
}

/** Mirrors `fn carries_an_obligation` in rust/src/selection_heuristics/splitting.rs. */
function carriesAnObligation(task, [start, end], head) {
  const text = task.slice(start, end).trim();
  if (evidencesAnObligation(text)) return true;
  return head !== null && !containsCjk(text) && evidencesAnObligation(`${head} ${text}`);
}

/** Mirrors `fn checkable_spans` in rust/src/selection_heuristics/splitting.rs. */
function checkableSpans(task) {
  const [raw, sentenceLevel] = rawSpans(task);
  if (raw.length < 2) return [];
  if (sentenceLevel) return raw;
  const head = headAction(task.slice(raw[0][0], raw[0][1]));
  const out = [];
  let pending = null;
  for (const span of raw) {
    const merged = pending === null ? span : [pending[0], span[1]];
    if (carriesAnObligation(task, span, head)) {
      out.push(merged);
      pending = null;
    } else {
      pending = merged;
    }
  }
  if (pending !== null) {
    if (out.length) out[out.length - 1] = [out[out.length - 1][0], pending[1]];
    else out.push(pending);
  }
  return out.length < 2 ? rawSpans(task)[0] : out;
}

/** Mirrors `fn balanced_split` in rust/src/selection_heuristics/splitting.rs: `{left, right}` or null. */
function balancedSplit(task) {
  const spans = checkableSpans(task);
  if (spans.length < 2) return null;
  const cut = spans[Math.ceil(spans.length / 2)][0];
  return { left: task.slice(0, cut).trim(), right: task.slice(cut).trim() };
}

/** Mirrors `fn complete_layers` in rust/src/selection_heuristics/splitting.rs. */
function completeLayers(task) {
  const segments = checkableSpans(task).length;
  return segments < 2 ? 0 : Math.floor(Math.log2(segments));
}

/** Mirrors `fn split_heuristic` in rust/src/meta_frame.rs: the last `split` heuristic, a balanced splitter. */
function splitHeuristic(span) {
  const heuristics = heuristicsFor(methodRegistry(), 'split', '');
  return heuristics.length ? balancedSplit(span) : null;
}

// ------------------------------------------------------------ problem frame

/** Mirrors `fn split_sentences` in rust/src/meta_frame.rs. */
function splitSentences(text) {
  const guarded = [...requirementOperandSpans(text), ...requirementListSpans(text)];
  const sentences = [];
  let current = '';
  const characters = Array.from(text);
  let index = 0;
  characters.forEach((character, position) => {
    current += character;
    const at = index;
    index += character.length;
    if (guarded.some(([start, end]) => at >= start && at < end)) return;
    const next = characters[position + 1];
    if (NEED_TERMINATORS.has(character) || (character === '.' && (next === undefined || isWhitespace(next)))) {
      if (current.trim()) sentences.push(current.trim());
      current = '';
    }
  });
  if (current.trim()) sentences.push(current.trim());
  return sentences;
}

/** Mirrors `fn split_clauses` in rust/src/meta_frame.rs. */
function splitClauses(sentence) {
  return orderedRequirementSpans(sentence, wordsForRole(ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR)).map((span) => span.source_text);
}

/** Mirrors `fn segment_needs` in rust/src/meta_frame.rs. */
function segmentNeeds(text) {
  return splitSentences(text).flatMap(splitClauses).map((clause) => clause.trim()).filter(Boolean);
}

/** Mirrors `ProblemFrame::from_formalization`. */
export function problemFrame(formalization) {
  const frameId = stableId('problem_frame', formalization.impulse_id);
  const needId = (index, span) => stableId('problem_need', `${frameId}:${index}:${span}`);
  const segments = segmentNeeds(formalization.source_text);
  const needs = segments.length <= 1
    ? [{ need_id: needId(0, formalization.source_text), source_span: formalization.source_text, kind: formalization.kind, route: formalization.route }]
    : segments.map((span, index) => {
      const segment = formalizeSpan(span, formalization.language);
      return { need_id: needId(index, span), source_span: span, kind: segment.kind, route: segment.route };
    });
  return {
    frame_id: frameId,
    impulse_id: formalization.impulse_id,
    language: formalization.language,
    kind: formalization.kind,
    route: formalization.route,
    needs: needs.map((need) => ({ ...need, status: 'pending' })),
  };
}

/** Mirrors `ProblemFrame::to_links_notation` (and `Need::to_links_notation`). */
function problemFrameLinksNotation(frame) {
  const pairs = [
    ['record_type', 'problem_frame'], ['frame_id', frame.frame_id], ['impulse_id', frame.impulse_id],
    ['language', frame.language], ['kind', frame.kind], ['need_count', String(frame.needs.length)],
  ];
  if (frame.route !== null) pairs.push(['route', frame.route]);
  for (const need of frame.needs) pairs.push(['need', need.need_id]);
  let out = formatLinoRecord(frame.frame_id, pairs);
  for (const need of frame.needs) {
    const needPairs = [
      ['record_type', 'problem_need'], ['need_id', need.need_id], ['source_span', need.source_span],
      ['kind', need.kind], ['status', need.status],
    ];
    if (need.route !== null) needPairs.push(['route', need.route]);
    out += `\n${formatLinoRecord(need.need_id, needPairs)}`;
  }
  return out;
}

/** Mirrors `fn record_problem_frame` in rust/src/meta_frame.rs. */
export function recordProblemFrame(log, formalization) {
  const frame = problemFrame(formalization);
  log.push({ kind: 'problem_frame', payload: problemFrameLinksNotation(frame) });
  log.push({ kind: 'problem_frame:need_count', payload: String(frame.needs.length) });
  for (const need of frame.needs) log.push({ kind: 'problem_frame:need', payload: `${need.kind} ${need.source_span}` });
  return frame;
}

// ------------------------------------------------------------ work units

/** Mirrors `WorkUnit::build` in rust/src/meta_frame.rs. */
function buildWorkUnit(span, parent, language, depth, maxDepth) {
  const unitId = stableId('work_unit', `${debugOption(parent)}:${depth}:${span}`);
  const route = formalizeSpan(span, language).route;
  const leaf = (reason) => ({ unit_id: unitId, parent, source_span: span, depth, atomic: true, reason, route, children: [] });
  if (depth >= maxDepth) return leaf('depth_bound');
  const split = splitHeuristic(span);
  if (split === null) return leaf(route !== null ? 'direct_method' : 'single_need');
  const children = [split.left, split.right].map((child) => buildWorkUnit(child, unitId, language, depth + 1, maxDepth));
  return { unit_id: unitId, parent, source_span: span, depth, atomic: false, reason: 'not_atomic', route, children };
}

/** Mirrors `WorkUnit::from_formalization`. */
export function workUnitTree(formalization, maxDepth) {
  const complete = completeLayers(formalization.source_text);
  const bound = complete === 0 ? maxDepth : Math.min(maxDepth, complete);
  return buildWorkUnit(formalization.source_text, null, formalization.language, 0, bound);
}

const unitCount = (unit) => 1 + unit.children.reduce((sum, child) => sum + unitCount(child), 0);
const leafCount = (unit) => (unit.atomic ? 1 : unit.children.reduce((sum, child) => sum + leafCount(child), 0));

/** Mirrors `WorkUnit::collect_leaves`. */
export function collectLeaves(unit, out = []) {
  if (unit.atomic) out.push(unit);
  else for (const child of unit.children) collectLeaves(child, out);
  return out;
}

/** Mirrors `WorkUnit::to_links_notation`. */
function workUnitLinksNotation(unit) {
  const pairs = [
    ['record_type', 'work_unit'], ['unit_id', unit.unit_id], ['source_span', unit.source_span],
    ['depth', String(unit.depth)], ['atomic', String(unit.atomic)], ['atomicity_reason', unit.reason],
  ];
  if (unit.parent !== null) pairs.push(['parent', unit.parent]);
  if (unit.route !== null) pairs.push(['route', unit.route]);
  for (const child of unit.children) pairs.push(['child', child.unit_id]);
  let out = formatLinoRecord(unit.unit_id, pairs);
  for (const child of unit.children) out += `\n${workUnitLinksNotation(child)}`;
  return out;
}

/** Mirrors `WorkUnit::emit_events`. */
function emitWorkUnitEvents(unit, log) {
  log.push({ kind: 'work_unit:enter', payload: `${unit.depth} ${unit.source_span}` });
  for (const child of unit.children) emitWorkUnitEvents(child, log);
  log.push({ kind: 'work_unit:exit', payload: `${unit.depth} ${unit.reason}` });
}

/** Mirrors `fn record_work_units` in rust/src/meta_frame.rs. */
export function recordWorkUnits(log, formalization, maxDepth) {
  const root = workUnitTree(formalization, maxDepth);
  log.push({ kind: 'work_unit', payload: workUnitLinksNotation(root) });
  log.push({ kind: 'work_unit:count', payload: String(unitCount(root)) });
  log.push({ kind: 'work_unit:leaf_count', payload: String(leafCount(root)) });
  emitWorkUnitEvents(root, log);
  return root;
}

// ------------------------------------------------------------ need ledger

/** Mirrors `fn best_leaf_for` in rust/src/meta_frame.rs. */
function bestLeafFor(leaves, span) {
  const exact = leaves.find((leaf) => leaf.source_span === span);
  if (exact) return exact;
  const overlapping = leaves.filter((leaf) => span.includes(leaf.source_span) || leaf.source_span.includes(span));
  return lastMaxBy(overlapping, (leaf) => new TextEncoder().encode(leaf.source_span).length);
}

/** Mirrors `NeedLedger::resolve`. */
export function needLedger(frame, root) {
  const leaves = collectLeaves(root);
  return {
    frame_id: frame.frame_id,
    rows: frame.needs.map((need) => {
      const leaf = bestLeafFor(leaves, need.source_span);
      return {
        need_id: need.need_id,
        source_span: need.source_span,
        status: leaf && leaf.route !== null ? 'planned' : 'blocked',
        leaf_reason: leaf ? leaf.reason : null,
        unit_id: leaf ? leaf.unit_id : null,
        route: leaf ? leaf.route : null,
      };
    }),
  };
}

/** Mirrors `NeedLedger::count_with`. */
export const ledgerCount = (ledger, status) => ledger.rows.filter((row) => row.status === status).length;

/** Mirrors `NeedLedger::to_links_notation` (and `LedgerRow::to_links_notation`). */
export function needLedgerLinksNotation(ledger) {
  const pairs = [
    ['record_type', 'need_ledger'], ['frame_id', ledger.frame_id], ['row_count', String(ledger.rows.length)],
    ['planned', String(ledgerCount(ledger, 'planned'))], ['satisfied', String(ledgerCount(ledger, 'satisfied'))],
    ['blocked', String(ledgerCount(ledger, 'blocked'))],
  ];
  for (const row of ledger.rows) pairs.push(['row', row.need_id]);
  let out = formatLinoRecord(stableId('need_ledger', ledger.frame_id), pairs);
  for (const row of ledger.rows) {
    const rowPairs = [['record_type', 'need_ledger_row'], ['need_id', row.need_id], ['source_span', row.source_span], ['status', row.status]];
    if (row.leaf_reason !== null) rowPairs.push(['leaf_reason', row.leaf_reason]);
    if (row.unit_id !== null) rowPairs.push(['unit_id', row.unit_id]);
    if (row.route !== null) rowPairs.push(['route', row.route]);
    out += `\n${formatLinoRecord(stableId('need_ledger_row', `${row.need_id}:${row.status}`), rowPairs)}`;
  }
  return out;
}

/** Mirrors `fn record_need_ledger` in rust/src/meta_frame.rs. */
export function recordNeedLedger(log, frame, root) {
  const ledger = needLedger(frame, root);
  log.push({ kind: 'need_ledger', payload: needLedgerLinksNotation(ledger) });
  log.push({ kind: 'need_ledger:accounted_for', payload: String(ledger.rows.length > 0 && ledger.rows.every((row) => row.status !== 'pending')) });
  for (const row of ledger.rows) log.push({ kind: 'need:status', payload: `${row.status} ${row.source_span}` });
  return ledger;
}
