// The one-level checkable split of a task (rust/src/task_decomposition.rs,
// rust/src/task_decomposition/strategy.rs, the `TaskStrategyLedger::shipped`
// approvals of rust/src/task_decomposition/learning.rs).
//
// Only what the obligation ledger reaches is ported: `split_once_checkable`,
// `is_checkable`, `plans_for`, `missing_operation_contract`,
// `concrete_target`. The full `decompose_task` tree and the learning gate stay
// in Rust.

import { cached, readText } from '../host.mjs';
import { findChildValue, parseLinoRoot } from '../write_lino.mjs';
import { mentionsRole, mentionsRoleRaw, wordsForRole } from '../write_lexicon.mjs';
import { isAlphanumeric, isAsciiDigit, splitWhitespace, trim, trimEndMatches, trimMatches } from '../write_str.mjs';
import { containsCjk } from './coding_catalog.mjs';
import { normalizePrompt } from './engine.mjs';
import { stableId } from './engine_stable_id.mjs';
import { detect } from './language.mjs';
import { localizedResponse } from './seed.mjs';

/** Mirrors `DEFAULT_SPLIT_DEPTH_BOUND` in rust/src/recursive_execution.rs. */
export const DEFAULT_SPLIT_DEPTH_BOUND = 4;

function childrenOf(text, rootName) {
  const root = parseLinoRoot(text).children.find((node) => node.name === rootName);
  return root ? root.children || [] : null;
}

/** Mirrors `fn strategies` (`parse_strategies`). */
export function strategies() {
  return cached('write:task_strategies', () => (childrenOf(readText('data/meta/task-decomposition-strategies.lino'), 'task_decomposition_strategies') ?? [])
    .filter((node) => node.name === 'strategy' && node.id)
    .map((node) => ({
      id: node.id,
      activation: findChildValue(node, 'activation'),
      stages: (node.children || [])
        .filter((child) => child.name === 'stage' && child.id)
        .map((stage) => ({
          id: stage.id,
          text_intent: findChildValue(stage, 'text_intent'),
          completion_criterion: findChildValue(stage, 'completion_criterion'),
        }))
        .filter((stage) => stage.text_intent && stage.completion_criterion),
    }))
    .filter((strategy) => strategy.activation && strategy.stages.length));
}

/** Mirrors `fn parse_contract`: whether the invariant document is well formed. */
function contractPresent() {
  return cached('write:task_contract', () => {
    const root = parseLinoRoot(readText('data/meta/task-decomposition-invariant.lino')).children
      .find((node) => node.name === 'task_decomposition_contract');
    if (!root || findChildValue(root, 'record_type') !== 'meta_invariant') return false;
    return ['atomic', 'execution', 'binary', 'learning'].every((key) => findChildValue(root, key) !== '');
  });
}

/** Mirrors `TaskStrategyLedger::shipped`: the set of approved strategy ids. */
export function shippedLedger() {
  return cached('write:task_ledger', () => {
    if (!contractPresent()) return new Set();
    const nodes = childrenOf(readText('data/meta/task-decomposition-strategies.lino'), 'task_decomposition_strategies') ?? [];
    const parseCount = (raw) => (/^\+?[0-9]+$/.test(raw) ? Number(raw) : null);
    return new Set(nodes
      .filter((node) => node.name === 'approved_strategy' && node.id)
      .filter((node) => {
        const passed = parseCount(findChildValue(node, 'passed'));
        const failed = parseCount(findChildValue(node, 'failed'));
        if (passed === null || failed === null) return false;
        return ['failed_task_id', 'failure_evidence', 'suite', 'reviewer'].every((key) => findChildValue(node, key) !== '')
          && passed !== 0 && failed === 0;
      })
      .map((node) => node.id));
  });
}

/** Mirrors `fn plans_for`: the planned stages, or null. @param {Set<string>} ledger */
export function plansFor(task, ledger = shippedLedger()) {
  if (!missingOperationContract(task)) return null;
  const strategy = strategies().find((entry) => entry.activation === 'missing_operation_contract' && ledger.has(entry.id));
  if (!strategy) return null;
  const language = detect(task);
  const stages = strategy.stages.map((stage) => ({
    strategy_id: strategy.id,
    stage_id: stage.id,
    text: (localizedResponse(stage.text_intent, language) ?? '').split('{task}').join(task),
    completion_criterion: stage.completion_criterion,
  }));
  return stages.length ? stages : null;
}

/** Mirrors `fn missing_operation_contract`. */
export function missingOperationContract(task) {
  const normalized = task.toLowerCase();
  const namesWorkItem = mentionsRole('decomposable_task_noun', normalized) || mentionsRoleRaw('decomposable_task_noun', normalized);
  const asksForSoftware = mentionsRole('software_authoring_action', normalized)
    || mentionsRoleRaw('software_authoring_action', normalized);
  const referencesWorkItem = splitWhitespace(task).some(isRepositoryWorkItem);
  return referencesWorkItem || namesWorkItem || (asksForSoftware && concreteTarget(task) === null);
}

/** Mirrors `fn concrete_target`. */
export function concreteTarget(task) {
  for (const token of splitWhitespace(task)) {
    const cleaned = trimMatches(token, (character) => '<>()[]{},;:"\''.includes(character));
    if (cleaned === '' || isRepositoryWorkItem(cleaned)) continue;
    const dot = cleaned.lastIndexOf('.');
    const fileLike = cleaned.includes('/') || (dot > 0 && dot < cleaned.length - 1);
    const identifierLike = cleaned.includes('_')
      || (cleaned.includes('-') && Array.from(cleaned).every((character) => isAlphanumeric(character) || character === '-'));
    if (fileLike || identifierLike) return trimEndMatches(cleaned, (character) => character === '.');
  }
  return null;
}

function isRepositoryWorkItem(token) {
  const trimmed = trimEndMatches(token, (character) => '.。!?'.includes(character));
  let path = null;
  if (trimmed.startsWith('https://github.com/')) path = trimmed.slice('https://github.com/'.length);
  else if (token.startsWith('http://github.com/')) path = token.slice('http://github.com/'.length);
  if (path === null) return false;
  const segments = path.split('/');
  return segments.length === 4 && (segments[2] === 'issues' || segments[2] === 'pull')
    && Array.from(segments[3]).every(isAsciiDigit);
}

/** Mirrors `fn split_once_checkable`. */
export function splitOnceCheckable(task, ledger = shippedLedger()) {
  const segments = segment(task);
  if (segments.length < 2) return [];
  const merged = mergeUncheckable(distributeHeadAction(segments), ledger);
  return merged.length < 2 ? [] : merged;
}

/** Mirrors `fn is_checkable`. */
export function isCheckable(text) {
  return completionCriterionFor(text) !== null;
}

function completionCriterionFor(text) {
  const normalized = normalizePrompt(text);
  const observable = mentionsRole('observable_task_action', normalized);
  const unobservable = mentionsRole('unobservable_task_action', normalized);
  if (!observable || unobservable || missingOperationContract(text)) return null;
  return `observable_result:${concreteTarget(text) ?? stableId('task_result', normalized)}`;
}

function segment(task) {
  const sentences = splitSentences(task);
  return sentences.length > 1 ? sentences : splitClauses(task);
}

function splitSentences(text) {
  const chars = Array.from(text);
  const out = [];
  let current = '';
  chars.forEach((character, index) => {
    current += character;
    const strong = '?!。！？'.includes(character);
    const next = chars[index + 1];
    const period = character === '.' && (next === undefined || /^\p{White_Space}$/u.test(next));
    if (strong || period) {
      pushTrimmed(out, current);
      current = '';
    }
  });
  pushTrimmed(out, current);
  return out;
}

function splitClauses(sentence) {
  const markers = wordsForRole('clause_continuation_marker');
  const out = [];
  for (const chunk of sentence.split(/[,;，；、]/u)) {
    for (const piece of splitOnMarkers(chunk, markers)) pushTrimmed(out, piece);
  }
  return out;
}

function splitOnMarkers(text, markers) {
  let pieces = [text];
  for (const marker of markers) pieces = pieces.flatMap((piece) => splitPieceOnMarker(piece, marker));
  return pieces;
}

const asciiFold = (text) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());

function splitPieceOnMarker(piece, marker) {
  if (containsCjk(marker)) return piece.split(marker).map(trim);
  const toks = splitWhitespace(piece);
  const isMarker = (token) => asciiFold(token) === asciiFold(marker);
  if (!toks.some(isMarker)) return [piece];
  const out = [];
  let current = [];
  for (const token of toks) {
    if (isMarker(token)) {
      out.push(current.join(' '));
      current = [];
    } else current.push(token);
  }
  out.push(current.join(' '));
  return out;
}

function distributeHeadAction(segments) {
  const head = headAction(segments[0]);
  if (head === null) return [...segments];
  return segments.map((text, index) => {
    if (index === 0 || headAction(text) !== null || containsCjk(text) || splitWhitespace(text).length < 3) return text;
    return `${head} ${text}`;
  });
}

function headAction(text) {
  if (containsCjk(text)) return null;
  const first = splitWhitespace(text)[0];
  if (first === undefined) return null;
  return mentionsRole('observable_task_action', normalizePrompt(first)) ? first : null;
}

function mergeUncheckable(segments, ledger) {
  const out = [];
  let pending = null;
  for (const text of segments) {
    const joined = pending === null ? text : join(pending, text);
    pending = null;
    if (isCheckable(text) || plansFor(text, ledger) !== null) out.push(joined);
    else pending = joined;
  }
  if (pending !== null) {
    if (out.length) out[out.length - 1] = join(out[out.length - 1], pending);
    else out.push(pending);
  }
  return out;
}

function join(left, right) {
  return `${left}${containsCjk(left) || containsCjk(right) ? '，' : ', '}${right}`;
}

function pushTrimmed(out, candidate) {
  const trimmed = trim(candidate);
  if (trimmed !== '') out.push(trimmed);
}
