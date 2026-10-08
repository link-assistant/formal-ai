#!/usr/bin/env node
// JavaScript twin of `scripts/check-file-size.rs` (PR #1188, SCRIPTS-B).
//
// Check maintained files (Rust at 1000 lines, every other authored text
// format at 1500) for maximum and warning line-count thresholds. Exits with
// error code 1 if any files exceed the hard limit. Same output, same exit code
// and the same `::warning` annotations as the Rust original, so a coding agent
// can run the gate without compiling Rust; CI runs both and diffs them
// (`data/meta/ci-gates/check-file-size-js-twin.lino`).
//
// Usage: node scripts/check-file-size.mjs
//
// The rules (limits, exclusions, the embedded-data check, the
// FILE_SIZE_WARNING_BASE growth filter) are the Rust script's; read its doc
// comments for why each exists.

import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

import {
  RustError, extension, fileName, lineCount, lines, readToString, trimStart, walkDir,
} from './lib/checks-rust-compat.mjs';

const RUST_LIMIT = { extension: 'rs', maxLines: 1000, warnLines: 900, label: 'Rust' };
const LINO_LIMIT = { extension: 'lino', maxLines: 1500, warnLines: 1400, label: 'Links Notation' };
export const FILE_LIMITS = [RUST_LIMIT, LINO_LIMIT];
export const WORKER_JS_LIMIT = { extension: 'js', maxLines: 1500, warnLines: 1400, label: 'Worker JavaScript' };
export const WORKFLOW_YAML_LIMIT = { extension: 'yml', maxLines: 1500, warnLines: 1400, label: 'GitHub Actions workflow' };
export const MAINTAINED_LIMIT = { extension: '*', maxLines: 1500, warnLines: 1400, label: 'Maintained' };
const MAINTAINED_EXTENSIONS = [
  'js', 'mjs', 'cjs', 'jsx', 'ts', 'tsx', 'css', 'html', 'md', 'py', 'sh', 'json', 'toml', 'yml',
  'yaml', 'txt', 'tsv', 'csv',
];
const UNMAINTAINED_FILE_NAMES = ['package-lock.json', 'bun.lock', 'yarn.lock'];
const UNMAINTAINED_PATH_FRAGMENTS = [
  'docs/case-studies/',
  'js/vendor/tree-sitter/',
  'rust/tests/fixtures/coding-discovery/python-docs/',
  'rust/tests/fixtures/coding-discovery/captured/',
  'rust/tests/fixtures/meta-reasoner/captures/',
];
const UNMAINTAINED_SUFFIXES = ['.bundle.js', '.min.js', '.min.css'];
const EXCLUDE_PATTERNS = ['target', '.git', 'node_modules', '.claude'];
const EXCLUDE_PATH_FRAGMENTS = ['dev/log/'];
const EMBEDDED_DATA_MESSAGE = 'Worker JavaScript must load Links Notation data from data/seed via seed_loader.js, not embed _LINO arrays or template literals.';

function shouldExclude(path) {
  return path.split('/').some((component) => EXCLUDE_PATTERNS.includes(component))
    || EXCLUDE_PATH_FRAGMENTS.some((fragment) => path.includes(fragment));
}

export function isWorkerJsPath(path) {
  const ext = extension(path);
  const hasJsExtension = ext !== null && ext.toLowerCase() === 'js';
  return path.endsWith('js/formal_ai_worker.js') || (path.includes('/js/worker/') && hasJsExtension);
}

function isWorkflowYamlPath(path) {
  const ext = extension(path)?.toLowerCase();
  return (ext === 'yml' || ext === 'yaml') && (path.startsWith('.github/') || path.includes('/.github/'));
}

/** The limit a path is measured against, or `null` when it is not measured. */
export function fileLimit(path) {
  if (isWorkerJsPath(path)) return WORKER_JS_LIMIT;
  if (isWorkflowYamlPath(path)) return WORKFLOW_YAML_LIMIT;
  const ext = extension(path);
  if (ext === null) return null;
  return FILE_LIMITS.find((limit) => limit.extension === ext)
    ?? (isMaintainedText(path, ext) ? MAINTAINED_LIMIT : null);
}

function isMaintainedText(path, ext) {
  const name = fileName(path);
  const dataCacheCapture = path.includes('data/cache/') && ext === 'json';
  return MAINTAINED_EXTENSIONS.includes(ext)
    && !dataCacheCapture
    && !UNMAINTAINED_FILE_NAMES.includes(name)
    && !UNMAINTAINED_SUFFIXES.some((suffix) => name.endsWith(suffix))
    && !UNMAINTAINED_PATH_FRAGMENTS.some((fragment) => path.includes(fragment));
}

export function classifyLineCount(count, limit) {
  if (count > limit.maxLines) return 'violation';
  if (count > limit.warnLines) return 'warning';
  return 'within';
}

function relativePath(path, cwd) {
  if (path === cwd) return '';
  return path.startsWith(`${cwd}/`) ? path.slice(cwd.length + 1) : path;
}

function isEmbeddedLinoDataLine(line) {
  const trimmed = trimStart(line);
  const declaresLino = (trimmed.startsWith('const ') || trimmed.startsWith('let ') || trimmed.startsWith('var '))
    && trimmed.includes('_LINO')
    && trimmed.includes('=');
  return declaresLino && (trimmed.includes('= [') || trimmed.includes('= `'));
}

/**
 * Measure the files under `cwd`, or only those in `listed` when it is given.
 * `warn(line)` receives the stderr lines the Rust original prints.
 */
export function checkListedFiles(cwd, listed = null, warn = (line) => process.stderr.write(`${line}\n`)) {
  const result = { warnings: [], violations: [], embeddedDataViolations: [] };
  // Directories listed paths live under, so a walk need not open the rest.
  const listedDirectories = listed ? directoriesOf(listed) : null;
  // Pruning only skips directories none of whose files the Rust walk would
  // measure: an excluded component or fragment, or nothing committable below.
  const skip = (path) => {
    const relative = relativePath(path, cwd);
    return shouldExclude(`${relative}/`)
      || (listedDirectories !== null && !listedDirectories.has(relative));
  };
  for (const entry of walkDir(cwd, skip)) {
    if (entry.error || entry.type !== 'file') continue;
    const relative = relativePath(entry.path, cwd);
    if (shouldExclude(entry.path) || (listed && !listed.has(relative))) continue;
    const limit = fileLimit(entry.path);
    const checkWorkerData = isWorkerJsPath(entry.path);
    if (!limit && !checkWorkerData) continue;
    let content;
    try {
      content = readToString(entry.path);
    } catch (error) {
      if (!(error instanceof RustError)) throw error;
      warn(`Warning: Could not read ${entry.path}: ${error.message}`);
      continue;
    }
    if (limit) {
      const count = lineCount(content);
      const finding = { file: relative, lines: count, maxLines: limit.maxLines, warnLines: limit.warnLines, label: limit.label };
      const status = classifyLineCount(count, limit);
      if (status === 'violation') result.violations.push(finding);
      else if (status === 'warning') result.warnings.push(finding);
    }
    if (checkWorkerData) {
      lines(content).forEach((line, index) => {
        if (isEmbeddedLinoDataLine(line)) {
          result.embeddedDataViolations.push({ file: relative, line: index + 1, message: EMBEDDED_DATA_MESSAGE });
        }
      });
    }
  }
  return result;
}

function directoriesOf(paths) {
  const directories = new Set();
  for (const path of paths) {
    let at = path.lastIndexOf('/');
    while (at > 0) {
      const directory = path.slice(0, at);
      if (directories.has(directory)) break;
      directories.add(directory);
      at = directory.lastIndexOf('/');
    }
  }
  return directories;
}

function escapeAnnotationProperty(value) {
  return value.replaceAll('%', '%25').replaceAll('\r', '%0D').replaceAll('\n', '%0A')
    .replaceAll(':', '%3A').replaceAll(',', '%2C');
}

function escapeAnnotationMessage(value) {
  return value.replaceAll('%', '%25').replaceAll('\r', '%0D').replaceAll('\n', '%0A');
}

export function warningAnnotation(finding) {
  const message = `${finding.label} file has ${finding.lines} lines (approaching limit of ${finding.maxLines}). Consider extracting content to keep at or below ${finding.warnLines} lines and prevent review and merge conflicts.`;
  return `::warning file=${escapeAnnotationProperty(finding.file)}::${escapeAnnotationMessage(message)}`;
}

export function growingPathsFromNumstat(numstat) {
  return lines(numstat).flatMap((line) => {
    const [added, removed, ...rest] = line.split('\t');
    if (rest.length === 0 || !/^\+?[0-9]+$/.test(added) || !/^\+?[0-9]+$/.test(removed)) return [];
    return Number(added) > Number(removed) ? [rest.join('\t')] : [];
  });
}

function committablePaths() {
  const output = spawnSync('git', ['ls-files', '-z', '--cached', '--others', '--exclude-standard'], { maxBuffer: 1 << 30 });
  if (output.error || output.status !== 0) return null;
  return new Set(output.stdout.toString('utf8').split('\0').filter(Boolean));
}

function growingPathsSince(base) {
  if (base === '' || [...base].every((ch) => ch === '0')) return null;
  const output = spawnSync('git', ['diff', '--numstat', '--diff-filter=ACMR', base, 'HEAD', '--'], { maxBuffer: 1 << 30 });
  if (output.error || output.status !== 0) return null;
  return growingPathsFromNumstat(output.stdout.toString('utf8'));
}

function printWarnings(warnings, out) {
  if (warnings.length === 0) return;
  for (const warning of warnings) {
    out(warningAnnotation(warning));
    out(`WARNING: ${warning.file} has ${warning.lines} lines (approaching ${warning.label} limit of ${warning.maxLines}, warning threshold: ${warning.warnLines})`);
  }
  out('');
  out('The following files are approaching their configured line limits:');
  for (const warning of warnings) out(`  ${warning.file}`);
  out('\nConsider extracting code to prevent concurrent PR merge limit violations.\n');
}

function printViolations(violations, out) {
  if (violations.length === 0) return;
  out('Found files exceeding the line limit:\n');
  for (const violation of violations) {
    out(`  ${violation.file}: ${violation.lines} lines (exceeds ${violation.label} limit of ${violation.maxLines})`);
  }
  out('\nPlease refactor or split these files to stay under their limits\n');
}

function printEmbeddedDataViolations(violations, out) {
  if (violations.length === 0) return;
  out('Found embedded Links Notation data in worker JavaScript:\n');
  for (const violation of violations) out(`  ${violation.file}:${violation.line}: ${violation.message}`);
  out('\nMove worker seed data to data/seed and load it through seed_loader.js\n');
}

function main() {
  const out = (line) => process.stdout.write(`${line}\n`);
  out('\nChecking file line limits: Rust 1000, every other maintained text format 1500...\n');
  const cwd = process.cwd();
  const result = checkListedFiles(cwd, committablePaths());
  const growing = growingPathsSince(process.env.FILE_SIZE_WARNING_BASE ?? '');
  printWarnings(growing ? result.warnings.filter((finding) => growing.includes(finding.file)) : result.warnings, out);
  if (result.violations.length === 0 && result.embeddedDataViolations.length === 0) {
    out('All checked files are within their line limits\n');
    process.exitCode = 0;
    return;
  }
  printViolations(result.violations, out);
  printEmbeddedDataViolations(result.embeddedDataViolations, out);
  process.exitCode = 1;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main();
