// Shared by the three text capability benchmarks (R1188-U18, U19, U21):
// scripts/measure-web-formalization.mjs, scripts/measure-round-trip-translation.mjs
// and scripts/measure-dependency-summarization.mjs.
//
// It installs the planner host the crate modules read the seed through,
// reads the cached page corpus, and keeps each capability's ratchet file. A
// ratchet file is one Links Notation record of `name value` rows; every value
// is a share in [0, 1] or a count, higher is better, and none may fall:
//
//   web-formalization-ratchet
//     source "scripts/measure-web-formalization.mjs"
//     sentence-coverage 0.051
//
// Shares are recorded rounded down to three decimals, and a measurement is
// compared after the same rounding, so float noise never fails the gate.

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { hasHost, installHost } from '../../js/agentic/host.mjs';
import { REPO_ROOT, childrenNamed, childValue, parseLino, readRepoFile } from '../../js/server/lino.mjs';

export const PAGE_CORPUS_DIRECTORY = 'data/benchmarks/web-formalization';
export const LANGUAGES = Object.freeze(['en', 'ru', 'hi', 'zh', 'es']);

/** Install the minimal planner host (repository files and the Links Notation parser). */
export function installTextHost() {
  if (!hasHost()) installHost({ readText: readRepoFile, parseLino });
}

/** The records named `name` in a parsed file, whether the file holds one record or several. */
export function recordsNamed(root, name) {
  return root.name === name ? [root] : childrenNamed(root, name);
}

/** The cached pages of one language: `{language, title, url, revision, text}`. */
export function readPages(language) {
  const root = parseLino(readRepoFile(`${PAGE_CORPUS_DIRECTORY}/${language}.lino`));
  return recordsNamed(root, 'web-formalization-page').map((page) => ({
    language,
    title: childValue(page, 'title'),
    url: childValue(page, 'source-url'),
    revision: childValue(page, 'revision-id'),
    text: childrenNamed(page, 'paragraph').map((paragraph) => paragraph.value).join('\n'),
  }));
}

/** A share rounded down to three decimals; 0 when there is nothing to divide. */
export function share(part, whole) {
  return whole === 0 ? 0 : Math.floor((part / whole) * 1000) / 1000;
}

/** The rows of a ratchet file as a Map of name to number (non-numeric rows skipped). */
export function readRatchet(relative) {
  let text;
  try {
    text = readFileSync(join(REPO_ROOT, relative), 'utf8');
  } catch {
    return new Map();
  }
  const root = parseLino(text);
  const record = root.children && root.children.length > 0 && root.name === '' ? root.children[0] : root;
  const out = new Map();
  for (const row of record.children || []) {
    const value = Number(row.value);
    if (row.value !== '' && Number.isFinite(value)) out.set(row.name, value);
  }
  return out;
}

/** Write a ratchet file: header comment lines, the record name, quoted fields, then the measures. */
export function writeRatchet(relative, { comments, name, fields, measures }) {
  const lines = comments.map((line) => `# ${line}`.trimEnd());
  lines.push(name);
  for (const [field, value] of fields) lines.push(`  ${field} "${value}"`);
  for (const [measure, value] of measures) lines.push(`  ${measure} ${value}`);
  writeFileSync(join(REPO_ROOT, relative), `${lines.join('\n')}\n`);
}

/** `{falls, rises}`: the measures below their recorded value, and the ones above it. */
export function compareRatchet(measures, recorded) {
  const falls = [];
  const rises = [];
  for (const [measure, value] of measures) {
    if (!recorded.has(measure)) {
      rises.push({ measure, recorded: null, value });
      continue;
    }
    const previous = recorded.get(measure);
    if (value < previous) falls.push({ measure, recorded: previous, value });
    if (value > previous) rises.push({ measure, recorded: previous, value });
  }
  for (const [measure, previous] of recorded) {
    if (!measures.some(([name]) => name === measure)) falls.push({ measure, recorded: previous, value: null });
  }
  return { falls, rises };
}

/**
 * Check (or with --write, record) the measures against a ratchet file and
 * exit non-zero when one falls.
 */
export function enforceRatchet(relative, ratchet, argv) {
  const recorded = readRatchet(relative);
  const { falls, rises } = compareRatchet(ratchet.measures, recorded);
  for (const fall of falls) console.error(`FALL ${fall.measure}: recorded ${fall.recorded}, measured ${fall.value}`);
  for (const rise of rises) console.log(`rise ${rise.measure}: recorded ${rise.recorded}, measured ${rise.value}`);
  if (falls.length > 0) {
    console.error(`${relative}: ${falls.length} measure(s) fell below the ratchet; fix the regression (the ratchet only goes up).`);
    process.exitCode = 1;
    return;
  }
  if (argv.includes('--write')) {
    writeRatchet(relative, ratchet);
    console.log(`wrote ${relative}`);
    return;
  }
  if (rises.length > 0) console.log(`${relative}: run with --write to raise the ratchet.`);
  console.log(`${relative}: no measure fell.`);
}
