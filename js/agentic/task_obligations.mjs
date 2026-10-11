// A coding task that names several obligations is finished when all of them
// are (rust/src/agentic_coding/task_obligations.rs, issue #1099).
//
// An obligation is a `crate/obligation_ledger.mjs` node. `nextStep` returns
// `{kind: 'observe', node}`, `{kind: 'decompose', node}` or `{kind:
// 'report_gap', node_id, clause, span: [start, end], reason}` with `span` in
// UTF-8 bytes, exactly as Rust's `ObligationStep`.

import { Capability } from './capability.mjs';
import { classifyTool } from './capability_router.mjs';
import { composeGeneralChangePlan, GeneralPlanMode } from './general_planner.mjs';
import { observedPayload, reportedExitCode } from './tool_result.mjs';
import { Progress } from './progress.mjs';
import { records } from './transcript_evidence.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { EvidenceSource, ObservationKind, evidenceNames, observedEvidence, reportsSuccess } from './crate/execution_evidence.mjs';
import { formatLinoRecord } from './crate/links_format.mjs';
import {
  buildObligationTree, clausesWithSpans, collectLeaves, discharged, everyObligationDischarged, isObservable,
  observe, satisfiedCount, unsatisfiableCount,
} from './crate/obligation_ledger.mjs';
import { renderProtocolTemplate } from './crate/repository_workspace.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { DEFAULT_SPLIT_DEPTH_BOUND, splitOnceCheckable } from './crate/task_decomposition.mjs';
import { splitWhitespace, utf8Len } from './write_str.mjs';

/** Mirrors `fn obligations`: every leaf of an enumerated request, or null. */
export function obligations(request) {
  if (clausesWithSpans(request).length < 2) return null;
  return collectLeaves(agenticRoot(request));
}

/** Mirrors `fn next_step`. */
export function nextStep(request, messages) {
  if (obligations(request) === null) return null;
  const ledger = observedLedger(request, messages);
  const open = collectLeaves(ledger.root).filter((node) => !discharged(node));
  const observable = open.find((node) => isObservable(node.expectation));
  if (observable) return { kind: 'observe', node: observable };
  const splittable = open.find((node) => node.depth < DEFAULT_SPLIT_DEPTH_BOUND && splitOnceCheckable(node.clause).length >= 2);
  if (splittable) return { kind: 'decompose', node: splittable };
  const node = open[0];
  if (!node) return null;
  return {
    kind: 'report_gap',
    node_id: node.node_id,
    clause: node.clause,
    span: node.span,
    reason: node.expectation.kind === 'underivable' ? node.expectation.reason : node.expectation.kind,
  };
}

/** Mirrors `fn successfully_discharged`. */
export function successfullyDischarged(request, messages) {
  if (obligations(request) === null) return false;
  const ledger = observedLedger(request, messages);
  return everyObligationDischarged(ledger) && unsatisfiableCount(ledger) === 0 && satisfiedCount(ledger) >= 1;
}

/**
 * Mirrors `fn gap_answer`: the machine-readable gap record.
 * @param {[number, number]|{start: number, end: number}} span UTF-8 bytes
 */
export function gapAnswer(nodeId, clause, span, reason) {
  const [start, end] = Array.isArray(span) ? span : [span.start, span.end];
  return formatLinoRecord(nodeId, [
    ['record_type', 'obligation_gap'],
    ['clause', clause],
    ['span_start', String(start)],
    ['span_end', String(end)],
    ['reason', reason],
  ]);
}

const isWriteRecord = (record) => {
  const tool = splitWhitespace(record.command)[0];
  return tool !== undefined && classifyTool(tool) === Capability.Write;
};

/** Mirrors `fn observed_file_outcome`: the outcome object for `path`. */
export function observedFileOutcome(request, path, messages) {
  const ledger = {
    frame_id: stableId('obligation_ledger', request),
    root: {
      node_id: stableId('obligation', `delivery:${path}:${request}`),
      parent: null,
      clause: request,
      span: [0, utf8Len(request)],
      need_id: null,
      depth: 0,
      expectation: { kind: 'file_bytes', path, sha256: null },
      outcome: { kind: 'unattempted' },
      children: [],
    },
  };
  const progress = Progress.scan(messages);
  if (progress.attemptedWriteFor(path) && !progress.successfulWriteFor(path)) {
    const command = renderProtocolTemplate('obligation_write_observation', [['path', path]]) ?? 'obligation_write_observation';
    observe(ledger, observedEvidence(command, [path], null, new Uint8Array(0), ObservationKind.ToolResult, EvidenceSource.Harness));
    return ledger.root.outcome;
  }
  const all = records(messages);
  if (progress.successfulWriteFor(path)) {
    const reversed = [...all].reverse();
    const record = reversed.find((entry) => evidenceNames(entry, path) && reportsSuccess(entry))
      ?? reversed.find((entry) => evidenceNames(entry, path) && isWriteRecord(entry));
    if (record) return { kind: 'satisfied', record };
  }
  for (const record of all) {
    if (isWriteRecord(record)) observe(ledger, record);
  }
  return ledger.root.outcome;
}

function observedLedger(request, messages) {
  const ledger = { frame_id: stableId('obligation_ledger', request), root: agenticRoot(request) };
  for (const record of records(messages)) observeNonFileRecord(ledger.root, record);
  reconcileFileObservations(ledger.root, Progress.scan(messages));
  return ledger;
}

function agenticRoot(request) {
  const root = buildObligationTree(request, DEFAULT_SPLIT_DEPTH_BOUND);
  alignFileExpectations(root);
  return root;
}

const encoder = new TextEncoder();

function alignFileExpectations(node) {
  if (!node.children.length) {
    const plan = composeGeneralChangePlan(node.clause);
    if (plan !== null && node.expectation.kind === 'file_bytes') {
      node.expectation = { kind: 'file_bytes', path: plan.target, sha256: sha256Hex(encoder.encode(plan.content)) };
    }
  }
  for (const child of node.children) alignFileExpectations(child);
}

/** Mirrors `fn reconcile_file_observations`: exact verification receipts judge only their declared file leaf. */
function reconcileFileObservations(node, progress) {
  for (const child of node.children) reconcileFileObservations(child, progress);
  if (node.children.length || node.expectation.kind !== 'file_bytes') return;
  const plan = composeGeneralChangePlan(node.clause);
  if (plan?.mode !== GeneralPlanMode.LiteralFile || plan.target !== node.expectation.path) return;
  const raw = progress.latestRunOutputFor(plan.verification_command);
  if (raw === null || raw !== progress.latestSuccessfulRunOutputFor(plan.verification_command)) return;
  const payload = observedPayload(raw);
  if (payload === null) return;
  const record = observedEvidence(plan.verification_command, [plan.target], reportedExitCode(raw),
    encoder.encode(payload), ObservationKind.FileBytes, EvidenceSource.Harness);
  const isolated = { frame_id: stableId('file_observation', node.node_id), root: { ...node } };
  observe(isolated, record);
  node.outcome = isolated.root.outcome;
}

/** Mirrors `fn observe_non_file_record`: raw acknowledgements are not file-byte receipts. */
function observeNonFileRecord(root, record) {
  for (const node of collectLeaves(root)) {
    if (node.expectation.kind === 'file_bytes') continue;
    const isolated = { frame_id: stableId('non_file_observation', node.node_id), root: { ...node } };
    if (observe(isolated, record) !== null) {
      node.outcome = isolated.root.outcome;
      return;
    }
  }
}
