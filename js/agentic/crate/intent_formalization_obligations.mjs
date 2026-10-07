// Formalize a request into obligations instead of matching phrases
// (rust/src/intent_formalization/obligations.rs, issue #1166).
//
// A graph is `{nodes, classifications: [[node_id, kind]], literals:
// [[node_id, value]]}`: `nodes` are the ledger nodes of
// crate/obligation_ledger.mjs (`buildObligationTree`) at clause level, `kind`
// is `ObligationKind::slug`. The graph's `language` and its Links Notation
// rendering are not ported (no caller in this root reads them).

import { meaningEvidencedIn, mentionsRole, mentionsRoleRaw } from '../write_lexicon.mjs';
import { utf8Len } from '../write_str.mjs';
import { textOutsideQuotedSegments } from './coding_program_contract.mjs';
import { normalizePrompt } from './engine.mjs';
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
  const clauseStarts = clausesWithSpans(text).map(([, span]) => utf16Index(text, span[0]));
  let previousEnd = 0;
  const outputs = [];
  for (const literal of quotedSegmentSpans(text)) {
    const clauseStart = clauseStarts.filter((start) => start <= literal.start).reduce((max, start) => Math.max(max, start), 0);
    const window = text.slice(Math.max(previousEnd, clauseStart), literal.start);
    previousEnd = literal.end;
    const introduction = window.split(/[\n.;。]/u).pop();
    if (meaningEvidencedIn('print_stdout', introduction.toLowerCase()) && !outputs.includes(literal.text)) {
      outputs.push(literal.text);
    }
  }
  return outputs;
}

/** Mirrors `fn obligation_gap_lines`. */
export function obligationGapLines(text) {
  const graph = formalizeRequest(text);
  return [...gapReport(graph), ...unboundOutputReport(graph, boundOutputLiterals(text))];
}
