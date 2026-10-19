// The native execution report of a catalog program answer: rust/src/engine.rs
// `execution_report` (`execution_status_phrase`, `execution_output_label`,
// `execution_command_lines`) over rust/src/coding/catalog `ProgramLanguage`
// (`execution_status`, `environment`, `execution_notes`).
//
// The browser worker reports what its Web Worker sandbox could run; the
// native server reports the toolchain run recorded in data/seed/toolchains.lino.
// The worker hands over both its own block and the facts the native report is
// built from (`programExecution`, js/worker/formal_ai_worker_write_program_and_research.js), and this
// module swaps one for the other. The wording lives in
// data/meta/server-messages.lino.

import { childValue, parseLino, readRepoFile } from './lino.mjs';
import { serverMessage } from './messages.mjs';

const TOOLCHAINS_FILE = 'data/seed/toolchains.lino';
const RECORD_PROBE = 'toolchain_probe';
const STATUS_VERIFIED = 'verified';
const STATUS_UNAVAILABLE = 'unavailable';
const STATUS_NOT_PROBED = 'not_probed';
const REPORT_LANGUAGES = new Set(['ru', 'hi', 'zh']);
const FALLBACK_LANGUAGE = 'en';
const NOTES_PREFIX = 'program_execution_notes_';
const TEXT_FENCE_OPEN = '```text';
const FENCE = '```';

let probes = null;

function probeRows() {
  if (probes) return probes;
  probes = [];
  const visit = (node) => {
    for (const child of node?.children || []) {
      if (childValue(child, 'record_type') === RECORD_PROBE) probes.push(child);
      else visit(child);
    }
  };
  visit(parseLino(readRepoFile(TOOLCHAINS_FILE)));
  return probes;
}

/** `prerequisite::probe::seed_field`: one field of a language's probe row. */
export function toolchainField(language, field) {
  const row = probeRows().find((node) => childValue(node, 'language') === language);
  return row ? childValue(row, field) : '';
}

/** `ProgramLanguage::execution_status`, as the message-key slug. */
export function executionStatus(language) {
  const status = toolchainField(language, 'execution_status');
  return status === STATUS_VERIFIED || status === STATUS_UNAVAILABLE ? status : STATUS_NOT_PROBED;
}

function reportLanguage(language) {
  return REPORT_LANGUAGES.has(language) ? language : FALLBACK_LANGUAGE;
}

/**
 * `ProgramLanguage::execution_notes`: the catalog row's note, else the seed's
 * `program_execution_notes_<slug>` report text.
 */
function executionNotes(language, seedReport) {
  const key = `${NOTES_PREFIX}${language}`;
  const note = serverMessage(key);
  if (note !== key) return note;
  const seeded = typeof seedReport === 'function' ? seedReport(key) : null;
  return seeded ?? key;
}

/**
 * Mirrors rust/src/engine.rs `execution_report`.
 * @param {{language: string, checkCommand?: string|null, runCommand: string, output: string}} program
 * @param {string} responseLanguage
 * @param {(intent: string) => string|null} [seedReport] the seed's English report text
 * @returns {string}
 */
export function executionReport(program, responseLanguage, seedReport) {
  if (program.rediscoveredFrom) return rediscoveredReport(program, responseLanguage);
  const status = executionStatus(program.language);
  const language = reportLanguage(responseLanguage);
  const statusLine = serverMessage(`program_execution_status_line_${language}`, {
    status: serverMessage(`program_execution_status_${status}_${language}`),
    environment: toolchainField(program.language, 'environment'),
  });
  const commands = [];
  if (program.checkCommand) commands.push(serverMessage('program_execution_check_command', { command: program.checkCommand }));
  commands.push(serverMessage('program_execution_run_command', { command: program.runCommand }));
  const outputLabel = serverMessage(`program_execution_output_${status}_${language}`);
  return [
    statusLine,
    ...commands,
    `${outputLabel}:`,
    TEXT_FENCE_OPEN,
    program.output,
    FENCE,
    executionNotes(program.language, seedReport),
  ].join('\n');
}

/**
 * The report of a program the documentation route rediscovered and no
 * recorded run verified (issue #1165): its status names the page and the
 * decomposition check, in the response language when the messages carry it,
 * and it borrows no run or note of the language's recorded program. Mirrors
 * the `rediscovered_from` branch of rust/src/engine.rs `execution_report`.
 * @param {{checkCommand?: string|null, runCommand: string, output: string, rediscoveredFrom: string}} program
 * @param {string} responseLanguage
 * @returns {string}
 */
function rediscoveredReport(program, responseLanguage) {
  const key = (language) => `program_execution_rediscovered_${language}`;
  const localized = serverMessage(key(responseLanguage), { page: program.rediscoveredFrom });
  const statusLine = localized === key(responseLanguage)
    ? serverMessage(key(FALLBACK_LANGUAGE), { page: program.rediscoveredFrom })
    : localized;
  const commands = [];
  if (program.checkCommand) commands.push(serverMessage('program_execution_check_command', { command: program.checkCommand }));
  commands.push(serverMessage('program_execution_run_command', { command: program.runCommand }));
  const outputLabel = serverMessage(`program_execution_output_${STATUS_UNAVAILABLE}_${reportLanguage(responseLanguage)}`);
  return [statusLine, ...commands, `${outputLabel}:`, TEXT_FENCE_OPEN, program.output, FENCE].join('\n');
}

/**
 * The answer text with the worker's sandbox block replaced by the native
 * report; unchanged when the answer carries no program execution or the block
 * is no longer in the text.
 * @param {string} answer
 * @param {object|undefined} program the worker's `programExecution`
 * @param {(intent: string) => string|null} [seedReport]
 * @returns {string}
 */
export function nativeProgramAnswer(answer, program, seedReport) {
  if (!program || typeof program.block !== 'string' || !program.block) return answer;
  const at = answer.indexOf(program.block);
  if (at < 0) return answer;
  const report = executionReport(program, String(program.responseLanguage || FALLBACK_LANGUAGE), seedReport);
  return answer.slice(0, at) + report + answer.slice(at + program.block.length);
}
