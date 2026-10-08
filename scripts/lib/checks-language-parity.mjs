// JavaScript twin of the structural census in `scripts/language-parity-lib.rs`
// that `scripts/check-debt-ratchet.rs` embeds for its `language_parity_gaps`
// measure (PR #1188, SCRIPTS-B). Only the live gap count is ported: the debt
// ratchet reads nothing else from that module.
//
// A meaning owns the `lexeme <language>` lines nested under it; a meaning that
// declares some of the five target languages but not all of them is one gap.

import { join } from 'node:path';

import {
  RustError, compareStrings, extension, ioErrorMessage, isDir, lines, readDirEntries, readToString,
  splitWhitespace, trim,
} from './checks-rust-compat.mjs';

const SEED_ROOT = 'data/seed';
const TARGET_LANGUAGES = ['en', 'ru', 'hi', 'zh', 'es'];

/** `Ord for Path`: component by component, not character by character. */
function comparePaths(left, right) {
  const a = left.split('/').filter(Boolean);
  const b = right.split('/').filter(Boolean);
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    const order = compareStrings(a[index], b[index]);
    if (order) return order;
  }
  return a.length - b.length;
}

function linoFiles(directory, output) {
  let entries;
  try {
    entries = readDirEntries(directory);
  } catch (error) {
    throw new RustError(`cannot read ${directory}: ${ioErrorMessage(error)}`);
  }
  for (const entry of entries) {
    if (isDir(entry.path)) linoFiles(entry.path, output);
    else if (extension(entry.path) === 'lino') output.push(entry.path);
  }
}

/** The length-prefixed structural path of an owner (`encode_path`). */
function encodePath(labels) {
  return labels.map((label) => `${Buffer.byteLength(label, 'utf8')}:${label}`).join('');
}

/** The gaps across `documents` (`[source, text]` pairs); throws `RustError`. */
export function gapsFromDocuments(documents) {
  const owners = new Map();
  for (const [source, text] of documents) {
    const stack = [];
    lines(text).forEach((line, index) => {
      const lineNumber = index + 1;
      const trimmed = trim(line);
      if (trimmed === '' || trimmed.startsWith('#')) return;
      const prefix = line.length - line.replace(/^[ \t]*/, '').length;
      if (line.slice(0, prefix).includes('\t')) {
        throw new RustError(`${source}:${lineNumber}: tab indentation is not a structural Links Notation indent`);
      }
      while (stack.length && stack.at(-1)[0] >= prefix) stack.pop();
      const words = splitWhitespace(trimmed);
      if (words[0] === 'lexeme' && words.length > 1 && TARGET_LANGUAGES.includes(words[1])) {
        const language = words[1];
        if (!stack.length) {
          throw new RustError(`${source}:${lineNumber}: target lexeme \`${language}\` has no structural owner`);
        }
        const owner = stack.at(-1)[1];
        const ownerPath = encodePath(stack.map(([, label]) => label));
        const key = `${source}\u0000${ownerPath}`;
        if (!owners.has(key)) owners.set(key, { source, ownerPath, meaning: owner, languages: new Set() });
        const entry = owners.get(key);
        if (entry.languages.has(language)) {
          throw new RustError(`${source}:${lineNumber}: meaning \`${owner}\` declares duplicate \`lexeme ${language}\``);
        }
        entry.languages.add(language);
      }
      stack.push([prefix, trimmed]);
    });
  }
  return [...owners.values()]
    .map((owner) => ({
      source: owner.source,
      ownerPath: owner.ownerPath,
      meaning: owner.meaning,
      present: TARGET_LANGUAGES.filter((language) => owner.languages.has(language)),
      missing: TARGET_LANGUAGES.filter((language) => !owner.languages.has(language)),
    }))
    .filter((gap) => gap.missing.length > 0)
    .sort((left, right) => compareStrings(left.source, right.source) || compareStrings(left.ownerPath, right.ownerPath));
}

/** `language_parity::current_gap_count`; throws `RustError` with Rust's text. */
export function currentGapCount(root) {
  const files = [];
  linoFiles(join(root, SEED_ROOT), files);
  files.sort(comparePaths);
  const documents = files.map((path) => {
    const source = path.startsWith(`${root}/`) ? path.slice(root.length + 1) : path;
    try {
      return [source, readToString(path)];
    } catch (error) {
      throw new RustError(`cannot read ${path}: ${ioErrorMessage(error)}`);
    }
  });
  return gapsFromDocuments(documents).length;
}
