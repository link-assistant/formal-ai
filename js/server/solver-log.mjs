// Shared formalization projection with the native server meta-core recorder.
import { recordMetaCore } from '../agentic/crate/meta_core.mjs';
import { nativeSolverLog as projectSolverLog } from '../agentic/crate/solver_event_projection.mjs';

/** The server supplies its meta-core recorder; the browser records the shared prelude. */
export function nativeSolverLog(result) {
  return projectSolverLog(result, recordMetaCore);
}
