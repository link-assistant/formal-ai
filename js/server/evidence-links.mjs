// The solver event log and its evidence projection (rust/src/event_log.rs
// `EventLog::append` and `build_evidence_links`). The port lives in
// js/agentic/crate/event_log.mjs, which the browser worker runs as well
// (R1188-U29); the server modules and their tests import it from here.

export { EventLog, buildEvidenceLinks, eventLogEvidenceLinks, evidenceLink } from '../agentic/crate/event_log.mjs';
