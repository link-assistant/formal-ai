// The runtime obligation tree an answer must discharge before it may finish
// (rust/src/obligation_ledger.rs, rust/src/obligation_ledger/derivation.rs).
//
// Shapes (Rust enum -> tagged object):
// - expectation: `{kind: 'file_bytes', path, sha256: string|null}`,
//   `{kind: 'command_exit', command, expected_exit}`, `{kind: 'output_hash',
//   command, sha256}`, `{kind: 'symbolic_check', check_id}`,
//   `{kind: 'underivable', reason}`; `kind` is `ObligationExpectation::slug`.
// - outcome: `{kind: 'unattempted'}`, `{kind: 'refuted', record, mismatch}`,
//   `{kind: 'satisfied', record}`, `{kind: 'unsatisfiable', reason}`.
// - node: `{node_id, parent, clause, span: [start, end], need_id, depth,
//   expectation, outcome, children}`; `span` is UTF-8 bytes, as in Rust.
// The `ProblemFrame` / `NeedLedger` bridge and the Links Notation projections
// are not ported (the agentic planner reads neither).

import { cached, readText } from '../host.mjs';
import { composeGeneralChangePlan } from '../general_planner.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';
import { wordsForRole } from '../write_lexicon.mjs';
import { isAlphanumeric, isWhitespace, lines, trim, trimEnd, trimEndMatches, utf8Len } from '../write_str.mjs';
import { debugOption, stableId } from './engine_stable_identifier.mjs';
import { evidenceNames, reportsSuccess } from './execution_evidence.mjs';
import { detect } from './language.mjs';
import { sha256Hex } from './source_fetch.mjs';
import { isCheckable, splitOnceCheckable } from './task_decomposition.mjs';

const encoder = new TextEncoder();

/** Mirrors `ObligationExpectation::is_observable`. */
export function isObservable(expectation) {
  return expectation.kind !== 'underivable';
}

/** Mirrors `fn clauses_with_spans`: `[clause, [start, end]]` in UTF-8 bytes. */
export function clausesWithSpans(request) {
  const cues = wordsForRole('enumeration_cue');
  if (!cues.length) return [[trim(request), [0, utf8Len(request)]]];
  const boundaries = [0];
  for (let index = 0; index < request.length; index += request.codePointAt(index) > 0xffff ? 2 : 1) {
    if (index === 0 || !opensAClause(request, index)) continue;
    const rest = request.slice(index).toLowerCase();
    if (cues.some((cue) => startsWithCue(rest, cue))) boundaries.push(index);
  }
  boundaries.push(request.length);
  const unique = boundaries.filter((value, at) => at === 0 || value !== boundaries[at - 1]);
  const out = [];
  for (let at = 0; at + 1 < unique.length; at += 1) {
    const raw = request.slice(unique[at], unique[at + 1]);
    const trimmed = trim(raw);
    if (trimmed === '') continue;
    const start = utf8Len(request.slice(0, unique[at] + raw.indexOf(trimmed)));
    out.push([trimmed, [start, start + utf8Len(trimmed)]]);
  }
  return out;
}

/**
 * Mirrors `fn opens_a_clause`: the line break is read before trimming, so a
 * cue opening a new line cuts even when the previous line ends without a
 * sentence mark (issue #1166).
 */
function opensAClause(request, index) {
  const lineEnd = trimEndMatches(request.slice(0, index), (character) => [' ', '\t', '\r'].includes(character));
  const before = trimEnd(lineEnd);
  return before === '' || lineEnd.endsWith('\n') || ['.', '!', '?', ';', ':', '。', '！', '？', '।', '॥'].some((mark) => before.endsWith(mark));
}

/** @param {string} character */
function isUnspacedScript(character) {
  const cp = character.codePointAt(0);
  return (cp >= 0x3040 && cp <= 0x30ff) || (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0x9fff)
    || (cp >= 0xac00 && cp <= 0xd7af) || (cp >= 0xf900 && cp <= 0xfaff);
}

function startsWithCue(rest, cue) {
  if (!rest.startsWith(cue)) return false;
  if (Array.from(cue).some(isUnspacedScript)) return true;
  const next = Array.from(rest.slice(cue.length))[0];
  return next === undefined || !isAlphanumeric(next);
}

/** Mirrors `ExpectationRules::shipped` (`from_lino`). */
export function expectationRules() {
  return cached('write:obligation_rules', () => {
    const rules = [];
    for (const line of lines(readText('data/meta/obligation-evidence-contract.lino'))) {
      const trimmed = trim(line);
      if (trimmed.startsWith('rule ')) {
        rules.push({ name: unquoted(trim(trimmed.slice(5))), when: '', mode: '', expectation: '', otherwise: null, reason: '' });
        continue;
      }
      const rule = rules[rules.length - 1];
      if (!rule) continue;
      for (const key of ['when', 'mode', 'expectation', 'otherwise', 'reason']) {
        const value = field(trimmed, key);
        if (value !== null) {
          rule[key] = value;
          break;
        }
      }
    }
    return rules;
  });
}

function field(line, key) {
  if (!line.startsWith(key)) return null;
  const rest = line.slice(key.length);
  const first = Array.from(rest)[0];
  if (first === undefined || !isWhitespace(first)) return null;
  return unquoted(trim(rest));
}

function unquoted(raw) {
  return raw.length >= 2 && raw.startsWith('"') && raw.endsWith('"') ? raw.slice(1, -1) : raw;
}

const ruleWhen = (when) => expectationRules().find((rule) => rule.when === when) ?? null;

/** Mirrors `fn expected_digest`. */
export function expectedDigest(content) {
  let stated = trimEnd(content);
  const last = Array.from(stated).pop();
  if (last !== undefined && ['.', '。', '।', '!', '？', '?'].includes(last)) stated = stated.slice(0, -last.length);
  stated = trimEnd(stated);
  if (stated === '') return null;
  const normalized = stated.endsWith('\n') ? stated : `${stated}\n`;
  return sha256Hex(encoder.encode(normalized));
}

/** Mirrors `fn derive_expectation`. */
export function deriveExpectation(clause) {
  const plan = composeGeneralChangePlan(clause);
  if (plan !== null) {
    return shapeFor(plan, expectationRules().find((rule) => rule.when === 'general_plan_mode' && rule.mode === plan.mode) ?? null);
  }
  if (isCheckable(clause)) {
    const rule = ruleWhen('generated_check');
    if (rule === null || rule.expectation === 'symbolic_check') {
      return { kind: 'symbolic_check', check_id: `obligation:${stableId('obligation_check', trim(clause))}` };
    }
  }
  return { kind: 'underivable', reason: ruleWhen('no_artifact')?.reason ?? 'no_artifact_in_clause' };
}

function shapeFor(plan, rule) {
  if (rule === null) return { kind: 'underivable', reason: `no_rule_for_plan_mode_${plan.mode}` };
  const declared = expectedDigest(plan.content);
  const shape = declared === null && rule.otherwise !== null ? rule.otherwise : rule.expectation;
  const commandExit = { kind: 'command_exit', command: plan.verification_command, expected_exit: 0 };
  switch (shape) {
    case 'file_bytes': return { kind: 'file_bytes', path: plan.target, sha256: declared };
    case 'output_hash': return declared === null ? commandExit : { kind: 'output_hash', command: plan.verification_command, sha256: declared };
    case 'command_exit': return commandExit;
    case 'symbolic_check': return { kind: 'symbolic_check', check_id: `obligation:${stableId('obligation_check', plan.goal)}` };
    default: return { kind: 'underivable', reason: rule.reason };
  }
}

/** Mirrors `ObligationNode::leaf`. */
export function leafNode(parent, clause, span, depth, expectation) {
  const trimmed = trim(clause);
  return {
    node_id: stableId('obligation', `${debugOption(parent)}:${depth}:${trimmed}`),
    parent,
    clause: trimmed,
    span,
    need_id: null,
    depth,
    expectation,
    outcome: { kind: 'unattempted' },
    children: [],
  };
}

/** Mirrors `ObligationNode::build`. */
export function buildObligationTree(request, maxSplitDepth) {
  const root = leafNode(null, request, [0, utf8Len(request)], 0, deriveExpectation(request));
  const clauses = clausesWithSpans(request);
  if (clauses.length < 2) {
    expand(root, maxSplitDepth);
    return root;
  }
  root.expectation = { kind: 'underivable', reason: 'request_enumerates_several_clauses' };
  root.children = clauses.map(([clause, span]) => {
    const child = leafNode(root.node_id, clause, span, 1, deriveExpectation(clause));
    expand(child, maxSplitDepth);
    return child;
  });
  return root;
}

function expand(node, maxSplitDepth) {
  if (isObservable(node.expectation) || node.depth >= maxSplitDepth) return;
  const pieces = splitOnceCheckable(node.clause);
  if (pieces.length < 2) return;
  let cursor = 0;
  const children = [];
  for (const piece of pieces) {
    const trimmed = trim(piece);
    if (trimmed === '') continue;
    const offset = node.clause.slice(cursor).indexOf(trimmed);
    let span = node.span;
    if (offset >= 0) {
      const start = node.span[0] + utf8Len(node.clause.slice(0, cursor + offset));
      cursor += offset + trimmed.length;
      span = [start, start + utf8Len(trimmed)];
    }
    const child = leafNode(node.node_id, trimmed, span, Math.min(255, node.depth + 1), deriveExpectation(trimmed));
    expand(child, maxSplitDepth);
    children.push(child);
  }
  if (children.length >= 2) node.children = children;
}

/** Mirrors `ObligationNode::discharged`. */
export function discharged(node) {
  if (!node.children.length) return node.outcome.kind === 'satisfied' || node.outcome.kind === 'unsatisfiable';
  return node.children.every(discharged);
}

/** Mirrors `ObligationNode::collect_leaves`. */
export function collectLeaves(node, out = []) {
  if (!node.children.length) out.push(node);
  else for (const child of node.children) collectLeaves(child, out);
  return out;
}

/** Mirrors `fn mismatch_template`. */
export function mismatchTemplate(id, language) {
  const record = parseLinoRoot(readText('data/seed/obligation-mismatch.lino')).children
    .find((node) => findChildValue(node, 'record_type') === 'obligation_mismatch' && findChildValue(node, 'id') === id);
  if (!record) return '';
  const localized = findChildValue(record, language);
  return trim(localized) === '' ? findChildValue(record, 'en') : localized;
}

function mismatchSentence(node, id, bindings) {
  return bindings.reduce((sentence, [placeholder, value]) => sentence.split(placeholder).join(value),
    mismatchTemplate(id, detect(node.clause)));
}

/** Mirrors `ObligationNode::judge`: the outcome `record` reaches, or null. */
function judge(node, record) {
  const expectation = node.expectation;
  const satisfied = { kind: 'satisfied', record };
  const refuted = (id, bindings) => ({ kind: 'refuted', record, mismatch: mismatchSentence(node, id, bindings) });
  switch (expectation.kind) {
    case 'file_bytes': {
      if (!evidenceNames(record, expectation.path)) return null;
      if (expectation.sha256 !== null) {
        return record.observed_output_sha256 === expectation.sha256 ? satisfied : refuted('file_bytes_digest', [
          ['{path}', expectation.path], ['{observed}', record.observed_output_sha256], ['{expected}', expectation.sha256],
        ]);
      }
      return reportsSuccess(record) ? satisfied : refuted('file_bytes_no_success', [['{path}', expectation.path]]);
    }
    case 'command_exit':
      if (!evidenceNames(record, expectation.command)) return null;
      return record.exit_code === expectation.expected_exit ? satisfied : refuted('command_exit', [
        ['{command}', expectation.command],
        ['{observed}', record.exit_code === null ? 'none' : String(record.exit_code)],
        ['{expected}', String(expectation.expected_exit)],
      ]);
    case 'output_hash':
      if (!evidenceNames(record, expectation.command)) return null;
      return record.observed_output_sha256 === expectation.sha256 ? satisfied : refuted('output_hash', [
        ['{command}', expectation.command], ['{observed}', record.observed_output_sha256], ['{expected}', expectation.sha256],
      ]);
    case 'symbolic_check':
      if (!evidenceNames(record, expectation.check_id)) return null;
      return reportsSuccess(record) ? satisfied : refuted('symbolic_check', [['{check}', expectation.check_id]]);
    default:
      return null;
  }
}

/**
 * Mirrors `ObligationLedger::observe`: the node id the record moved, or null.
 * @param {{frame_id: string, root: object}} ledger
 */
export function observe(ledger, record) {
  for (const leaf of collectLeaves(ledger.root)) {
    if (leaf.outcome.kind === 'satisfied') continue;
    const outcome = judge(leaf, record);
    if (outcome !== null) {
      leaf.outcome = outcome;
      return leaf.node_id;
    }
  }
  return null;
}

function countOf(ledger, kind) {
  return collectLeaves(ledger.root).filter((leaf) => leaf.outcome.kind === kind).length;
}

/** Mirrors `ObligationLedger::every_obligation_discharged`. */
export const everyObligationDischarged = (ledger) => discharged(ledger.root);
/** Mirrors `ObligationLedger::satisfied_count`. */
export const satisfiedCount = (ledger) => countOf(ledger, 'satisfied');
/** Mirrors `ObligationLedger::unsatisfiable_count`. */
export const unsatisfiableCount = (ledger) => countOf(ledger, 'unsatisfiable');
