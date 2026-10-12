#!/usr/bin/env node
// JavaScript twin of `scripts/check-minimal-core-boundary.rs` (PR #1188,
// SCRIPTS-B).
//
// Recursive burn-down ratchet for issue #918's compiled handler debt: every
// compiled handler source must be ledgered in
// `data/meta/core-boundary-ledger.lino` at exactly its line count, and the
// summed ceilings must match. Same output and exit code as the Rust original;
// CI runs both and diffs them
// (`data/meta/ci-gates/check-minimal-core-boundary-js-twin.lino`). Read the
// Rust script's doc comments for why each rule exists.
//
// Usage: node scripts/check-minimal-core-boundary.mjs

import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import {
  RustError, compareStrings, debugString, extension, fileName, ioErrorMessage, isFile, lineCount, lines,
  parseUnsigned, readToString, splitOnce, trim, walkDir,
} from './lib/checks-rust-compat.mjs';

const LEDGER_PATH = 'data/meta/core-boundary-ledger.lino';
const HANDLER_ROOT = 'rust/src/solver_handlers';
const HANDLERS_OUTSIDE_ROOT = [
  'rust/src/solver_handler_how.rs',
  'rust/src/solver_handler_how_synthesis.rs',
  'rust/src/solver_handler_units.rs',
  'rust/src/solver_handler_oracle.rs',
];
const GENERATED_MODULE_LIST = 'modules.rs';
const TOP_LEVEL_FIELDS = ['source_file_count_max', 'source_lines_max', 'outside_core_file_count_max', 'outside_core_lines_max'];

function unquote(value) {
  return value.length >= 2 && value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value;
}

function parseUsize(value, field) {
  const parsed = parseUnsigned(value);
  if (parsed.error) throw new RustError(`invalid ${field} value ${debugString(value)}: ${parsed.error}`);
  return parsed.value;
}

const emptyEntry = (path) => ({ path, disposition: '', baselineLines: 0, dataTarget: '', coreComponent: '', reason: '' });
const emptyComponent = (name) => ({ name, file: '', kind: '', reason: '' });

/** Parse the boundary ledger (`parse_ledger`); throws `RustError`. */
export function parseLedger(text) {
  const ledger = {
    source_file_count_max: 0, source_lines_max: 0, outside_core_file_count_max: 0, outside_core_lines_max: 0,
    entries: [], components: [],
  };
  let current = null;
  let component = null;
  const flush = () => {
    if (current) ledger.entries.push(current);
    if (component) ledger.components.push(component);
    current = null;
    component = null;
  };
  lines(text).forEach((line, index) => {
    if (line.startsWith('  source ')) {
      flush();
      current = emptyEntry(line.slice('  source '.length));
      return;
    }
    if (line.startsWith('  component ')) {
      flush();
      component = emptyComponent(trim(line.slice('  component '.length)));
      return;
    }
    const indentedField = line.startsWith('    ');
    const parts = splitOnce(trim(line), ' ');
    if (!parts) return;
    const [field, value] = parts;
    if (!indentedField) {
      if (TOP_LEVEL_FIELDS.includes(field)) ledger[field] = parseUsize(value, field);
      return;
    }
    if (component) {
      if (field === 'file') component.file = unquote(value);
      else if (field === 'kind') component.kind = unquote(value);
      else if (field === 'reason') component.reason = unquote(value);
    } else if (current) {
      if (field === 'disposition') current.disposition = unquote(value);
      else if (field === 'baseline_lines') current.baselineLines = parseUsize(value, field);
      else if (field === 'data_target') current.dataTarget = unquote(value);
      else if (field === 'core_component') current.coreComponent = unquote(value);
      else if (field === 'reason') current.reason = unquote(value);
    } else {
      throw new RustError(`line ${index + 1} has a source field without a source`);
    }
  });
  flush();
  return ledger;
}

function isHandlerSource(path) {
  return extension(path) === 'rs' && fileName(path) !== GENERATED_MODULE_LIST;
}

/** Every compiled handler source with its line count, keyed in `BTreeMap` order. */
export function sourceFiles(root) {
  const files = new Map();
  for (const entry of walkDir(join(root, HANDLER_ROOT))) {
    if (entry.error) throw new RustError(`walk ${HANDLER_ROOT}: ${entry.error}`);
    if (entry.type !== 'file' || !isHandlerSource(entry.path)) continue;
    const relative = entry.path.slice(root.length + 1);
    let content;
    try {
      content = readToString(entry.path);
    } catch (error) {
      throw new RustError(`read ${relative}: ${ioErrorMessage(error)}`);
    }
    files.set(relative, lineCount(content));
  }
  for (const outside of HANDLERS_OUTSIDE_ROOT) {
    const path = join(root, outside);
    if (!isFile(path)) continue;
    let content;
    try {
      content = readToString(path);
    } catch (error) {
      throw new RustError(`read ${outside}: ${ioErrorMessage(error)}`);
    }
    files.set(outside, lineCount(content));
  }
  return new Map([...files].sort(([a], [b]) => compareStrings(a, b)));
}

const sortedDifference = (left, right) => [...left].filter((item) => !right.has(item)).sort(compareStrings);

/** Every way the ledger and the tree disagree (`audit`). */
export function audit(ledger, files) {
  const errors = [];
  const seen = new Set();
  const active = new Set();
  let totalLines = 0;
  let outsideCoreFiles = 0;
  let outsideCoreLines = 0;
  for (const entry of ledger.entries) {
    if (entry.path === '' || seen.has(entry.path)) {
      errors.push(`duplicate or empty ledger source ${debugString(entry.path)}`);
      continue;
    }
    seen.add(entry.path);
    if (trim(entry.reason) === '') errors.push(`${entry.path} has no audit reason`);
    if (entry.disposition === 'migrate' || entry.disposition === 'promote') {
      active.add(entry.path);
      if (!files.has(entry.path)) {
        errors.push(`${entry.path} is marked ${entry.disposition} but is absent; mark it delete`);
        continue;
      }
      const actualLines = files.get(entry.path);
      totalLines += actualLines;
      if (actualLines > entry.baselineLines) {
        errors.push(`${entry.path} grew from ${entry.baselineLines} to ${actualLines} lines`);
      } else if (actualLines < entry.baselineLines) {
        errors.push(`${entry.path} shrank from ${entry.baselineLines} to ${actualLines} lines; lower its reviewed baseline`);
      }
      if (entry.disposition === 'migrate') {
        outsideCoreFiles += 1;
        outsideCoreLines += actualLines;
        if (trim(entry.dataTarget) === '') errors.push(`${entry.path} has no data_target`);
        if (entry.coreComponent !== '') {
          errors.push(`${entry.path} is migration debt but names core_component ${entry.coreComponent}`);
        }
      } else {
        if (trim(entry.coreComponent) === '') errors.push(`${entry.path} is promoted without a core_component`);
        if (entry.dataTarget !== '') {
          errors.push(`${entry.path} is promoted but also names data_target ${debugString(entry.dataTarget)}`);
        }
      }
    } else if (entry.disposition === 'delete') {
      if (files.has(entry.path)) errors.push(`${entry.path} is marked delete but still exists`);
      if (entry.baselineLines !== 0) errors.push(`${entry.path} is deleted but has a nonzero baseline`);
    } else {
      errors.push(`${entry.path} has invalid disposition ${debugString(entry.disposition)}; expected migrate, promote, or delete`);
    }
  }
  const actual = new Set(files.keys());
  for (const path of sortedDifference(actual, active)) errors.push(`unledgered handler source ${path}`);
  for (const path of sortedDifference(active, actual)) {
    if (!errors.some((error) => error.startsWith(path))) errors.push(`ledger source ${path} is absent`);
  }
  const componentNames = new Set();
  const componentFiles = new Set();
  for (const record of ledger.components) {
    if (record.name === '' || componentNames.has(record.name)) {
      errors.push(`duplicate or empty component name ${debugString(record.name)}`);
    }
    componentNames.add(record.name);
    if (trim(record.file) === '') {
      errors.push(`component ${record.name} has no file`);
    } else if (componentFiles.has(record.file)) {
      errors.push(`component ${record.name} repeats file ${record.file}`);
    } else {
      componentFiles.add(record.file);
    }
    if (trim(record.kind) === '') errors.push(`component ${record.name} has no kind`);
    if (trim(record.reason) === '') errors.push(`component ${record.name} has no audit reason`);
    if (files.has(record.file)) {
      errors.push(`component ${record.name} file ${record.file} is inside the handler census; ledger it as a source`);
    }
  }
  for (const [label, measured, ceiling] of [
    ['source_file_count_max', files.size, ledger.source_file_count_max],
    ['source_lines_max', totalLines, ledger.source_lines_max],
    ['outside_core_file_count_max', outsideCoreFiles, ledger.outside_core_file_count_max],
    ['outside_core_lines_max', outsideCoreLines, ledger.outside_core_lines_max],
  ]) {
    if (measured > ceiling) errors.push(`${label} grew from ${ceiling} to ${measured}`);
    else if (measured < ceiling) errors.push(`${label} improved from ${ceiling} to ${measured}; lower the reviewed ceiling`);
  }
  return errors;
}

/** The report line on success; throws `RustError` with every failure. */
export function run(root) {
  if (!isFile(join(root, LEDGER_PATH))) {
    throw new RustError(`run from the repository root; ${LEDGER_PATH} was not found`);
  }
  let text;
  try {
    text = readToString(join(root, LEDGER_PATH));
  } catch (error) {
    throw new RustError(`read ${LEDGER_PATH}: ${ioErrorMessage(error)}`);
  }
  const ledger = parseLedger(text);
  const files = sourceFiles(root);
  const errors = audit(ledger, files);
  for (const record of ledger.components) {
    if (trim(record.file) === '') continue;
    if (!isFile(join(root, record.file))) errors.push(`component ${record.name} file ${record.file} is absent`);
  }
  if (errors.length) throw new RustError(errors.join('\n'));
  return `minimal-core boundary: ${files.size} handler sources, ${ledger.outside_core_lines_max} outside-core lines`;
}

function main() {
  try {
    process.stdout.write(`${run(process.cwd())}\n`);
  } catch (error) {
    if (!(error instanceof RustError)) throw error;
    process.stderr.write(`minimal-core boundary audit failed:\n${error.message}\n`);
    process.exitCode = 1;
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main();
