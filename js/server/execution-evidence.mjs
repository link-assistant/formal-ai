// The observation that turns a planned obligation into a satisfied one:
// rust/src/execution_evidence.rs (`Evidence::observed` and its content-addressed
// `evidence_id`). Only the constructor the Telegram execution path reaches is
// ported; `from_tool_result`, `names` and the Links Notation projection are not
// used by the JavaScript server.

import { createHash } from 'node:crypto';

import { stableId } from './ids.mjs';

/** Mirrors rust/src/execution_evidence.rs `ObservationKind::slug`. */
export const ObservationKind = Object.freeze({
  CommandExit: 'command_exit',
  FileBytes: 'file_bytes',
  ToolResult: 'tool_result',
  SymbolicCheck: 'symbolic_check',
});

/** Mirrors rust/src/execution_evidence.rs `EvidenceSource::slug`. */
export const EvidenceSource = Object.freeze({
  Harness: 'harness',
  LocalProcess: 'local_process',
  Engine: 'engine',
});

/**
 * Mirrors rust/src/source_fetch.rs `sha256_hex`.
 * @param {Uint8Array} bytes
 * @returns {string}
 */
export function sha256Hex(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}

/**
 * Mirrors rust/src/execution_evidence.rs `fingerprint_id`: command, argv, exit,
 * output hash, output length, kind and source, never a wall clock.
 */
function fingerprintId(command, argv, exitCode, sha, length, kind, source) {
  const exit = exitCode === null ? 'none' : String(exitCode);
  const fingerprint = [command, argv.join('\u001e'), exit, sha, String(length), kind, source].join('\u001f');
  return stableId('evidence', fingerprint);
}

/**
 * Mirrors rust/src/execution_evidence.rs `Evidence::observed`.
 * @param {string} command
 * @param {Array<string>} argv
 * @param {number|null} exitCode
 * @param {Uint8Array} observed
 * @param {string} kind an `ObservationKind` slug
 * @param {string} source an `EvidenceSource` slug
 */
export function observedEvidence(command, argv, exitCode, observed, kind, source) {
  const observedOutputSha256 = sha256Hex(observed);
  const observedByteLength = observed.length;
  return {
    evidence_id: fingerprintId(command, argv, exitCode, observedOutputSha256, observedByteLength, kind, source),
    for_need: '',
    produced_by: '',
    command,
    argv,
    exit_code: exitCode,
    observed_output_sha256: observedOutputSha256,
    observed_byte_length: observedByteLength,
    source_ids: [],
    kind,
    source,
    recorded_at: null,
  };
}
