// `crate::execution_evidence::Evidence` (rust/src/execution_evidence.rs): the
// record a transcript tool result becomes. Only the constructors the agentic
// planner reaches (`Evidence::observed`, `Evidence::from_tool_result`) and the
// two readers (`reports_success`, `names`) are ported.

import { stableId } from './engine_stable_identifier.mjs';
import { sha256Hex } from './source_fetch.mjs';
import { reportedExitCode } from '../tool_result.mjs';

/** Mirrors `ObservationKind::slug` values. */
export const ObservationKind = Object.freeze({
  CommandExit: 'command_exit',
  FileBytes: 'file_bytes',
  ToolResult: 'tool_result',
  SymbolicCheck: 'symbolic_check',
});

/** Mirrors `EvidenceSource::slug` values. */
export const EvidenceSource = Object.freeze({
  Harness: 'harness',
  LocalProcess: 'local_process',
  Engine: 'engine',
});

/** Mirrors `fn fingerprint_id` in rust/src/execution_evidence.rs. */
function fingerprintId(command, argv, exitCode, sha, length, kind, source) {
  const exit = exitCode === null ? 'none' : String(exitCode);
  const fingerprint = [command, argv.join('\u001e'), exit, sha, String(length), kind, source].join('\u001f');
  return stableId('evidence', fingerprint);
}

/** Mirrors `fn argv_of` in rust/src/execution_evidence.rs. */
export function argvOf(command) {
  const argv = [];
  let current = '';
  let quote = null;
  for (const character of command) {
    if (quote !== null && character === quote) quote = null;
    else if (quote === null && (character === "'" || character === '"')) quote = character;
    else if (quote === null && /^\p{White_Space}$/u.test(character)) {
      if (current) argv.push(current);
      current = '';
    } else current += character;
  }
  if (current) argv.push(current);
  return argv;
}

/**
 * Mirrors `Evidence::observed` in rust/src/execution_evidence.rs.
 * @param {string} command
 * @param {Array<string>} argv
 * @param {number|null} exitCode
 * @param {Uint8Array} observed
 * @param {string} kind
 * @param {string} source
 */
export function observedEvidence(command, argv, exitCode, observed, kind, source) {
  const sha = sha256Hex(observed);
  return {
    evidence_id: fingerprintId(command, argv, exitCode, sha, observed.length, kind, source),
    for_need: '',
    produced_by: '',
    command,
    argv,
    exit_code: exitCode,
    observed_output_sha256: sha,
    observed_byte_length: observed.length,
    source_ids: [],
    kind,
    source,
    detail: { kind: 'none' },
    recorded_at: null,
  };
}

/**
 * Mirrors `Evidence::from_tool_result` in rust/src/execution_evidence.rs.
 * @param {string} command
 * @param {string} raw
 * @param {string} source
 */
export function evidenceFromToolResult(command, raw, source) {
  return observedEvidence(
    command,
    argvOf(command),
    reportedExitCode(raw),
    new TextEncoder().encode(raw),
    ObservationKind.ToolResult,
    source,
  );
}

/** Mirrors `Evidence::reports_success`. */
export function reportsSuccess(evidence) {
  return evidence.exit_code !== null ? evidence.exit_code === 0 : evidence.observed_byte_length > 0;
}

/** Mirrors `Evidence::names`. */
export function evidenceNames(evidence, needle) {
  if (!needle) return false;
  return evidence.command === needle || evidence.command.includes(needle)
    || evidence.argv.some((argument) => argument === needle);
}
