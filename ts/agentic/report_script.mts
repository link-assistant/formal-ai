import {quoteShellWord as shellQuote} from './portable/string_contracts.mjs';
// The shell script an agentic report hands to the harness (#839):
// rust/src/agentic_coding/report_script.rs.

import { replaceAllLiteral } from './crate/rust_str.mjs';
import { agentInfoValue } from './crate/seed_agent_info.mjs';
import { agenticMessage } from './messages.mjs';

const STRICT_MODE = 'set -eu';
const EXPORT_VARIABLE = 'export_dir';
const EXPORT_SETUP = 'export_dir=$(mktemp -d "${TMPDIR:-/tmp}/formal-ai-export.XXXXXX")';

/** Mirrors `struct ReportScript` in rust/src/agentic_coding/report_script.rs. */
export class ReportScript {
  /** Mirrors `ReportScript::new`. */
  constructor() {
    this.programs = [];
    this.steps = [];
    this.export_dir = false;
  }

  /** Mirrors `ReportScript::step`: add one step and remember its program. */
  step(program, command) {
    if (!this.programs.includes(program)) this.programs.push(program);
    this.steps.push(command);
  }

  /** Mirrors `ReportScript::export`: a quoted path in the export directory. */
  export(name) {
    this.export_dir = true;
    return `"$${EXPORT_VARIABLE}/${name}"`;
  }

  /** Mirrors `ReportScript::render`. */
  render() {
    const lines = [STRICT_MODE, ...this.programs.map(preflight)];
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
 * Mirrors `fn quote_shell_word` in rust/src/portable_string_contracts.rs.
 * @param {string} value
 */
export {shellQuote};
