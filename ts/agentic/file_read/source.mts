// Source-read rendering consumes provider observations, never source JSON status keys.
import { sourceReadObservation, renderFailure } from '../tool_result.mjs';
import { FinalDisposition, FinalPayloadRole, resolvedFinalAnswer } from '../plan.mjs';
import { fileReadFinalAnswer } from './audit.mjs';
import { readRecordForPath } from './records.mjs';

/** Mirrors fn read_observation in file_read/source.rs. */
export function readObservation(records, progress, path) {
  const bound = progress.sourceReadFor(path);
  if (bound !== null) return bound;
  const legacy = readRecordForPath(records, path);
  if (legacy === null) return null;
  const read = sourceReadObservation(legacy.content, legacy.is_error, null, path);
  return { ...read, complete: false };
}
/** Mirrors fn source_read_answer in file_read/source.rs. */
export function sourceReadAnswer(paths, mode, records, progress, request) {
  const reads = paths.map((path) => readObservation(records, progress, path));
  const failed = reads.findIndex((read) => read?.error != null);
  if (failed >= 0) return resolvedFinalAnswer(renderFailure(paths[failed], reads[failed].error, request),
    FinalDisposition.Failure, 'file_read_observed');
  if (reads.some((read) => read === null)) return null;
  return resolvedFinalAnswer(fileReadFinalAnswer(mode, paths.map((path, index) => [path, reads[index].source ?? '']), request),
    reads.every((read) => read.complete) ? FinalDisposition.Finding : FinalDisposition.Unknown,
    'file_read_observed', mode.kind === 'audit' ? FinalPayloadRole.AuditReport : FinalPayloadRole.Finding);
}
