// Formalize a request into obligations instead of matching phrases
// (rust/src/intent_formalization/obligations.rs, issue #1166).
//
// A graph is `{nodes, classifications: [[node_id, kind]], literals:
// [[node_id, value]]}`: `nodes` are the ledger nodes of
// crate/obligation_ledger.mjs (`buildObligationTree`) at clause level, `kind`
// is `ObligationKind::slug`. The graph's `language` and its Links Notation
// rendering are not ported (no caller in this root reads them).

import { cached, readText } from '../host.mjs';
import { meaningEvidencedIn, mentionsRole, mentionsRoleRaw, wordsForRole } from '../write_lexicon.mjs';
import { parseLinoRoot } from '../write_lino.mjs';
import { utf8Len } from '../write_str.mjs';
import { textOutsideQuotedSegments } from './coding_program_contract.mjs';
import { normalizePrompt } from './engine.mjs';
import { detect } from './language.mjs';
import { quotedSegmentSpans } from './normal_markov.mjs';
import { buildObligationTree, clausesWithSpans, expectationRules } from './obligation_ledger.mjs';
import { DEFAULT_SPLIT_DEPTH_BOUND } from './task_decomposition.mjs';

/** `ObligationKind::slug` of the kinds this module assigns. */
export const ObligationKind = Object.freeze({
  OutputLiteral: 'output_literal',
  CiWorkflow: 'ci_workflow',
  ProgramFile: 'program_file',
  CodeStyle: 'code_style',
  FileNaming: 'file_naming',
  CiBadge: 'ci_badge',
});

/** Mirrors `OBLIGATION_GAP_KIND`. */
export const OBLIGATION_GAP_KIND = 'obligation_gap';

/** Mirrors `WHEN_UNBOUND_OUTPUT`. */
const WHEN_UNBOUND_OUTPUT = 'unbound_output_literal';

/** Mirrors the `ROLE_*_OBLIGATION` constants, in `authoring_kind` order. */
const AUTHORING_ROLES = [
  ['code_style_obligation', ObligationKind.CodeStyle],
  ['file_naming_obligation', ObligationKind.FileNaming],
  ['ci_badge_obligation', ObligationKind.CiBadge],
];

/** Mirrors `fn authoring_kind`: the authoring kind of a clause, or null. */
export function authoringKind(clause) {
  const normalized = normalizePrompt(textOutsideQuotedSegments(clause));
  const lower = normalized.toLowerCase();
  if (mentionsRole('ci_workflow_request', normalized)) return ObligationKind.CiWorkflow;
  if (mentionsRole('program_request', normalized) || mentionsRole('coding_request_verb', normalized)) {
    return ObligationKind.ProgramFile;
  }
  const found = AUTHORING_ROLES.find(([role]) => mentionsRoleRaw(role, lower));
  return found ? found[1] : null;
}

/** Mirrors `fn classify_clause`: `[kind, literal|null]` or null. */
function classifyClause(clause) {
  const lower = normalizePrompt(textOutsideQuotedSegments(clause)).toLowerCase();
  const printEvidence = meaningEvidencedIn('print_stdout', lower) || mentionsRoleRaw('output_obligation_verb', lower);
  const quoted = quotedSegmentSpans(clause);
  if (printEvidence && quoted.length > 0) {
    const value = quoted[quoted.length - 1].text;
    if (value !== '') return [ObligationKind.OutputLiteral, value];
  }
  const kind = authoringKind(clause);
  return kind === null ? null : [kind, null];
}

/** Mirrors `fn formalize_request`. */
export function formalizeRequest(text) {
  const root = buildObligationTree(text, DEFAULT_SPLIT_DEPTH_BOUND);
  const clauseNodes = root.children.length > 0 ? root.children : [root];
  const graph = { nodes: [], classifications: [], literals: [] };
  for (const node of clauseNodes) {
    const classified = classifyClause(node.clause);
    if (classified !== null) {
      if (classified[1] !== null) graph.literals.push([node.node_id, classified[1]]);
      graph.classifications.push([node.node_id, classified[0]]);
    }
    graph.nodes.push(node);
  }
  return coreferencePass(graph);
}

/** Mirrors `fn coreference_pass`: one output-literal node per value. */
export function coreferencePass(graph) {
  const kindOf = (nodeId) => graph.classifications.find(([id]) => id === nodeId)?.[1] ?? null;
  const literalOf = (nodeId) => graph.literals.find(([id]) => id === nodeId)?.[1] ?? null;
  const seen = [];
  const kept = { nodes: [], classifications: [], literals: [] };
  for (const node of graph.nodes) {
    const isOutput = kindOf(node.node_id) === ObligationKind.OutputLiteral;
    const literal = literalOf(node.node_id);
    if (isOutput && literal !== null && seen.includes(literal)) continue;
    if (isOutput && literal !== null) seen.push(literal);
    kept.classifications.push(...graph.classifications.filter(([id]) => id === node.node_id));
    kept.literals.push(...graph.literals.filter(([id]) => id === node.node_id));
    kept.nodes.push(node);
  }
  return kept;
}

const hasNodeKind = (graph, nodeId, kind) => graph.classifications.some(([id, present]) => id === nodeId && present === kind);

/** Mirrors `ObligationGraph::obligations_of_kind`. */
export function obligationsOfKind(graph, kind) {
  return graph.nodes.filter((node) => hasNodeKind(graph, node.node_id, kind));
}

/** `ObligationKind` declaration order (the `Ord` `obligation_kinds` sorts by). */
const KIND_ORDER = Object.values(ObligationKind);

/** Mirrors `ObligationGraph::obligation_kinds`: distinct kinds, in declaration order. */
export function obligationKinds(graph) {
  return KIND_ORDER.filter((kind) => graph.classifications.some(([, present]) => present === kind));
}

/** Mirrors `ObligationGraph::has_obligation`. */
export function hasObligation(graph, kind) {
  return graph.classifications.some(([, present]) => present === kind);
}

/** Mirrors `ObligationGraph::literal_for`: the literal of an output-literal node, or null. */
export function literalFor(graph, node) {
  if (!hasNodeKind(graph, node.node_id, ObligationKind.OutputLiteral)) return null;
  return graph.literals.find(([id]) => id === node.node_id)?.[1] ?? null;
}

/** Mirrors `ObligationGraph::unique_output_literal`: the one demanded value, or null. */
export function uniqueOutputLiteral(graph) {
  const values = obligationsOfKind(graph, ObligationKind.OutputLiteral)
    .map((node) => literalFor(graph, node))
    .filter((value) => value !== null);
  return values.length === 1 ? values[0] : null;
}

/** Mirrors `ObligationGraph::underivable`: the clauses no rule could read, kept. */
export function underivable(graph) {
  return graph.nodes.filter((node) => node.expectation.kind === 'underivable');
}

/** Mirrors `fn request_carries_work_obligations` (the terminal-router guard, R1166-8). */
export function requestCarriesWorkObligations(text) {
  const graph = formalizeRequest(text);
  const authoring = graph.nodes.map((node) => authoringKind(node.clause)).filter((kind) => kind !== null);
  const namesOrBadges = (kind) => hasObligation(graph, kind) || authoring.includes(kind);
  return (hasObligation(graph, ObligationKind.OutputLiteral) && authoring.length > 0)
    || namesOrBadges(ObligationKind.FileNaming)
    || namesOrBadges(ObligationKind.CiBadge);
}

/** Mirrors `fn request_demands` (R1166-3). */
export function requestDemands(text, kind) {
  const graph = formalizeRequest(text);
  return hasObligation(graph, kind) || graph.nodes.some((node) => authoringKind(node.clause) === kind);
}

/** Mirrors `fn gap_line`. */
function gapLine(node, reason) {
  return `obligation ${node.node_id} span ${node.span[0]}:${node.span[1]} underivable ${reason}`;
}

/** Mirrors `ObligationGraph::gap_report`. */
export function gapReport(graph) {
  return underivable(graph).map((node) => gapLine(node, node.expectation.reason));
}

/** Mirrors `ObligationGraph::unbound_output_report`. */
export function unboundOutputReport(graph, bound) {
  const reason = expectationRules().find((rule) => rule.when === WHEN_UNBOUND_OUTPUT)?.reason ?? WHEN_UNBOUND_OUTPUT;
  return graph.nodes
    .filter((node) => hasNodeKind(graph, node.node_id, ObligationKind.OutputLiteral))
    .filter((node) => {
      const literal = graph.literals.find(([id]) => id === node.node_id);
      return literal !== undefined && !bound.includes(literal[1]);
    })
    .map((node) => gapLine(node, reason));
}

/** The UTF-16 index of a UTF-8 byte offset into `text`. */
function utf16Index(text, byteOffset) {
  let bytes = 0;
  let index = 0;
  for (const character of text) {
    if (bytes >= byteOffset) break;
    bytes += utf8Len(character);
    index += character.length;
  }
  return index;
}

/** Mirrors `fn bound_output_literals`. */
export function boundOutputLiterals(text) {
  const clauses = clausesWithSpans(text);
  const clauseStarts = clauses.map(([, span]) => utf16Index(text, span[0]));
  const verbFinal = verbFinalLanguage(detect(text));
  const literals = quotedSegmentSpans(text);
  let previousEnd = 0;
  const bound = [];
  literals.forEach((literal, index) => {
    const clauseStart = clauseStarts.filter((start) => start <= literal.start).reduce((max, start) => Math.max(max, start), 0);
    const window = text.slice(Math.max(previousEnd, clauseStart), literal.start);
    previousEnd = literal.end;
    const introduction = window.split(/[\n.;。]/u).pop();
    const next = index + 1 < literals.length ? literals[index + 1].start : text.length;
    if (meaningEvidencedIn('print_stdout', introduction.toLowerCase())
      || (verbFinal && followedByPrint(text.slice(literal.end, next)))) bound.push([literal.start, literal.text]);
  });
  clauses.forEach(([clause], index) => {
    if (quotedSegmentSpans(clause).length > 0) return;
    const output = unquotedOutput(clause);
    if (output !== null) bound.push([clauseStarts[index], output]);
  });
  bound.sort((left, right) => left[0] - right[0]);
  const outputs = [];
  for (const [, value] of bound) if (!outputs.includes(value)) outputs.push(value);
  return outputs;
}

/** Mirrors `fn verb_final_language` in rust/src/solver_handlers/formalization_task.rs. */
function verbFinalLanguage(language) {
  const root = cached('intent_formalization_obligations:formal-targets',
    () => parseLinoRoot(readText('data/seed/formal-targets.lino') ?? '').children[0] ?? null);
  return (root?.children || []).some((node) => node.name === 'natural_language' && node.id === language
    && (node.children || []).some((child) => child.name === 'verb_final'));
}

/**
 * Mirrors `fn followed_by_print`: whether the words right after a quoted
 * literal — up to the sentence break or the first seeded clause separator —
 * evidence `print_stdout`, the way a verb-final language states the output
 * before its verb ("जो "Namaste" प्रिंट करे और उसे चलाओ").
 */
function followedByPrint(window) {
  const separators = wordsForRole('skill_procedure_clause_separator');
  const sentence = window.split(/[\n.;。]/u)[0];
  const words = [];
  for (const word of sentence.split(/\s+/u).filter(Boolean)) {
    if (separators.includes(bareWord(word))) break;
    words.push(word);
  }
  return meaningEvidencedIn('print_stdout', words.join(' ').toLowerCase());
}

/** Mirrors `UNQUOTED_OUTPUT_ENDS`. */
const UNQUOTED_OUTPUT_ENDS = /[.!?;:\u3002\uff01\uff1f\u0964]$/u;

/** Mirrors `UNQUOTED_OUTPUT_PEEL`. */
const UNQUOTED_OUTPUT_PEEL = /[.,;:\u3002\u0964]+$/u;

/** Mirrors `fn bare_word`. */
function bareWord(word) {
  return word.replace(/^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu, '').toLowerCase();
}

/** Mirrors `fn unquoted_output` (PR #1188 T18). */
function unquotedOutput(clause) {
  const words = clause.split(/\s+/u).filter(Boolean);
  const printAt = words.findIndex((word) => meaningEvidencedIn('print_stdout', bareWord(word)));
  if (printAt < 0) return null;
  const separators = wordsForRole('skill_procedure_clause_separator');
  const kept = [];
  for (const word of words.slice(printAt + 1)) {
    if (separators.includes(bareWord(word))) break;
    kept.push(word);
    if (UNQUOTED_OUTPUT_ENDS.test(word)) break;
  }
  const output = kept.join(' ').replace(UNQUOTED_OUTPUT_PEEL, '');
  const lower = normalizePrompt(output.toLowerCase());
  const functionWords = wordsForRole('statement_function_word');
  const describes = output.split(/\s+/u).some((word) => functionWords.includes(bareWord(word)))
    || mentionsRole('coding_structure', lower)
    || (mentionsRole('program_task_alias', lower) && !mentionsRole('social_greeting', lower));
  return /^\p{Uppercase}/u.test(output) && !describes ? output : null;
}

/** Mirrors `fn obligation_gap_lines`. */
export function obligationGapLines(text) {
  const graph = formalizeRequest(text);
  return [...gapReport(graph), ...unboundOutputReport(graph, boundOutputLiterals(text))];
}
