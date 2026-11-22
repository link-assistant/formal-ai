// A native program answer presents the run contract and any expected output.
// Historical toolchain probes do not establish that this particular code ran.
// Mirrors rust/src/engine/program_answer.rs.

import { childValue, parseLino, readRepoFile } from './lino.mjs';

const GUIDANCE_FILE = 'data/seed/coding-guidance.lino';
const FALLBACK_LANGUAGE = 'en';

let guidanceRows;
function guidance(key, language, values = {}) {
  guidanceRows ??= new Map(parseLino(readRepoFile(GUIDANCE_FILE)).children
    .filter((row) => childValue(row, 'record_type') === 'coding_guidance')
    .map((row) => [childValue(row, 'id'), row]));
  const row = guidanceRows.get(key);
  const template = childValue(row, language) || childValue(row, FALLBACK_LANGUAGE);
  return template.replace(/\{([a-z_]+)\}/g, (whole, name) => Object.hasOwn(values, name) ? String(values[name]) : whole);
}

function commandLines(program) {
  const lines = [];
  if (program.checkCommand) lines.push(guidance('execution-check-command', FALLBACK_LANGUAGE, { command: program.checkCommand }));
  lines.push(guidance('execution-run-command', FALLBACK_LANGUAGE, { command: program.runCommand }));
  return lines;
}

/**
 * Mirrors rust/src/engine/program_answer.rs `execution_report`.
 * @param {{checkCommand?: string|null, runCommand: string, output?: string|null, rediscoveredFrom?: string}} program
 * @param {string} responseLanguage
 * @returns {string}
 */
export function executionReport(program, responseLanguage) {
  const page = program.rediscoveredFrom;
  const statusLine = page
    ? guidance('execution-rediscovered', responseLanguage, { page })
    : guidance('execution-composed-line', responseLanguage, {
      status: guidance('execution-status-not-probed', responseLanguage),
    });
  const lines = [statusLine, ...commandLines(program)];
  if (program.output != null && program.output !== '') {
    const key = page ? 'execution-output-unavailable' : 'execution-output-not-probed';
    lines.push(`${guidance(key, responseLanguage)}:`, '```text', program.output, '```');
  }
  return lines.join('\n');
}

/**
 * The answer text with the worker's sandbox block replaced by the native
 * report; unchanged when the answer carries no program execution or the block
 * is no longer in the text.
 * @param {string} answer
 * @param {object|undefined} program the worker's `programExecution`
 * @returns {string}
 */
export function nativeProgramAnswer(answer, program) {
  if (!program || typeof program.block !== 'string' || !program.block) return answer;
  const at = answer.indexOf(program.block);
  if (at < 0) return answer;
  const report = executionReport(program, String(program.responseLanguage || FALLBACK_LANGUAGE));
  return answer.slice(0, at) + report + answer.slice(at + program.block.length);
}
