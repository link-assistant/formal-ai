// Lower trusted typed setup recipes into client-owned tools
// (rust/src/agentic_coding/prerequisite_recovery.rs, issue #1159), with the
// part of `crate::prerequisite::classify_failure` (rust/src/prerequisite/mod.rs)
// the route reads.
//
// Rust gates every install on the process environment
// (`FORMAL_AI_INSTALL_GRANT=workspace`). The planner's JavaScript host has no
// process environment, so the grant is never present: a missing prerequisite
// is answered by the `prerequisite_grant_required_report`, exactly as the Rust
// server answers without the grant. The publisher fetch / setup-document
// recipe beyond the grant (crate::prerequisite::publisher) is therefore
// unreachable and not ported.

import { finalAnswer } from './plan.mjs';
import { agenticMessage } from './messages.mjs';
import { fill } from './work_item_steps.mjs';
import { splitWhitespace } from './write_str.mjs';

/** Mirrors `probe::COMMAND_NOT_FOUND_EXIT`. */
const COMMAND_NOT_FOUND_EXIT = 127;
/** Mirrors `probe::NOT_FOUND_MARKER`. */
const notFoundMarker = () => agenticMessage('prerequisite_recovery_not_found_marker');

/**
 * Mirrors `fn classify_failure` for the verdict the route reads:
 * `{program, observed: 'missing'|'unusable'}` or null.
 */
export function classifyFailure(command, exitCode, stderr) {
  const program = splitWhitespace(command)[0] ?? '';
  if (!program) return null;
  const missing = exitCode === COMMAND_NOT_FOUND_EXIT || stderr.includes(notFoundMarker());
  const denied = exitCode === 126 || stderr.includes('Permission denied');
  if (!missing && !denied) return null;
  return { program, observed: missing ? 'missing' : 'unusable' };
}

/**
 * Mirrors `fn plan_recovery`.
 * @param {Array<object>} messages
 * @param {Array<string>} tools
 * @param {string} command
 * @param {number|null} exitCode
 * @param {string} output
 */
export function planRecovery(messages, tools, command, exitCode, output) {
  const need = classifyFailure(command, exitCode, output);
  if (!need || need.observed !== 'missing') return null;
  return finalAnswer(fill('prerequisite_grant_required_report', [
    ['{program}', need.program],
    ['{command}', command],
    ['{output}', output],
  ]));
}
