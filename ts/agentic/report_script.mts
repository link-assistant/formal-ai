// The shell script an agentic report hands to the harness (#839):
// rust/src/agentic_coding/report_script.rs.

import { replaceAllLiteral } from './crate/rust_str.mjs';
import { agentInfoValue } from './crate/seed_agent_info.mjs';
import { agenticMessage } from './messages.mjs';

const STRICT_MODE = 'set -eu';
const SCRATCH_VARIABLE = 'report_dir';
const SCRATCH_SETUP = 'report_dir=$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-report.XXXXXX")\n'
  + 'trap \'rm -rf "$report_dir"\' EXIT';
const EXPORT_VARIABLE = 'export_dir';
const EXPORT_SETUP = 'export_dir=$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-export.XXXXXX")';

/** Mirrors `struct ReportScript` in rust/src/agentic_coding/report_script.rs. */
export class ReportScript {
  /** Mirrors `ReportScript::new`. */
  constructor() {
    this.programs = [];
    this.steps = [];
    this.scratch_dir = false;
    this.export_dir = false;
  }

  /** Mirrors `ReportScript::step`: add one step and remember its program. */
  step(program, command) {
    if (!this.programs.includes(program)) this.programs.push(program);
    this.steps.push(command);
  }

  /** Mirrors `ReportScript::scratch`: a quoted path in the scratch directory. */
  scratch(name) {
    this.scratch_dir = true;
    return `"$${SCRATCH_VARIABLE}/${name}"`;
  }

  /** Mirrors `ReportScript::export`: a quoted path in the export directory. */
  export(name) {
    this.export_dir = true;
    return `"$${EXPORT_VARIABLE}/${name}"`;
  }

  /** Mirrors `ReportScript::render`. */
  render() {
    const lines = [STRICT_MODE, ...this.programs.map(preflight)];
    if (this.scratch_dir) lines.push(SCRATCH_SETUP);
    if (this.export_dir) lines.push(EXPORT_SETUP);
    lines.push(...this.steps);
    return lines.join('\n');
  }
}

/** Mirrors `fn preflight`: the `command -v` guard for one program. */
function preflight(program) {
  const message = replaceAllLiteral(agentInfoValue('issue_report_command_missing') ?? '', '{command}', program);
  return agenticMessage('report_script_preflight', { command: program, message: shellQuote(message) });
}

/**
 * Mirrors `fn shell_quote` in rust/src/agentic_coding/report_script.rs.
 * @param {string} value
 */
export function shellQuote(value) {
  return `'${replaceAllLiteral(value, "'", "'\\''")}'`;
}
