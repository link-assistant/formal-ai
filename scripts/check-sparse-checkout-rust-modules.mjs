#!/usr/bin/env node
// Fail when a workflow sparse checkout lists a rust-script file but omits a
// sibling module that file loads through `#[path = "..."]` (issue #1634: the
// detect-code-changes.rs sparse lists missed github-output.rs and every
// change-detection job on main failed to compile).
// Set FORMAL_AI_SPARSE_CHECK_VERBOSE=1 to print every checked block.
import {existsSync, readFileSync, readdirSync} from 'node:fs';
import {dirname, join, posix, resolve} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const VERBOSE = process.env.FORMAL_AI_SPARSE_CHECK_VERBOSE === '1';

// Returns arrays of non-cone sparse patterns found in a workflow source.
export function sparseBlocks(source) {
  const lines = source.split(/\r?\n/u), blocks = [];
  for (let i = 0; i < lines.length; i++) {
    const match = /^(\s*)(?:-\s+)?sparse-checkout:\s*(.*)$/u.exec(lines[i]);
    if (!match) continue;
    const inline = match[2].trim();
    if (inline && inline !== '|' && inline !== '|-') {
      blocks.push({line: i + 1, patterns: inline.replace(/^["']|["']$/gu, '').split(/\\n|\s+/u).filter(Boolean)});
      continue;
    }
    const base = match[1].length, patterns = [];
    for (let j = i + 1; j < lines.length; j++) {
      if (!lines[j].trim()) continue;
      if (lines[j].search(/\S/u) <= base) break;
      patterns.push(lines[j].trim());
    }
    blocks.push({line: i + 1, patterns});
  }
  // Inline flow mappings: `with: {..., sparse-checkout: "a\nb"}`.
  lines.forEach((text, index) => {
    for (const m of text.matchAll(/[{,]\s*sparse-checkout:\s*"([^"]*)"/gu))
      blocks.push({line: index + 1, patterns: m[1].split('\\n').filter(Boolean)});
  });
  return blocks;
}

const covers = (patterns, path) => patterns.some(pattern => {
  const p = pattern.replace(/^\//u, '');
  return p === path || (p.endsWith('/') && path.startsWith(p)) || (!p.includes('.') && path.startsWith(p + '/'));
});

export function rustModules(path, read = file => readFileSync(join(ROOT, file), 'utf8')) {
  const seen = new Set(), queue = [path];
  while (queue.length) {
    const current = queue.pop();
    let text;
    try { text = read(current); } catch { continue; }
    for (const m of text.matchAll(/#\[path\s*=\s*"([^"]+)"\]/gu)) {
      const dep = posix.normalize(posix.join(posix.dirname(current), m[1]));
      if (!seen.has(dep)) { seen.add(dep); queue.push(dep); }
    }
  }
  return [...seen];
}

export function checkSource(file, source, read) {
  const problems = [];
  for (const block of sparseBlocks(source)) {
    for (const pattern of block.patterns) {
      const path = pattern.replace(/^\//u, '');
      if (!/\.rs$/u.test(path)) continue;
      for (const dep of rustModules(path, read)) {
        if (!covers(block.patterns, dep))
          problems.push(`${file}:${block.line}: sparse checkout lists /${path} but not its #[path] module /${dep}`);
      }
    }
    if (VERBOSE) console.error(`[sparse-check] ${file}:${block.line} ${block.patterns.join(' ')}`);
  }
  return problems;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const dir = join(ROOT, '.github', 'workflows');
  const problems = [];
  for (const name of readdirSync(dir).filter(n => /\.ya?ml$/u.test(n)).sort())
    problems.push(...checkSource(`.github/workflows/${name}`, readFileSync(join(dir, name), 'utf8')));
  const actions = join(ROOT, '.github', 'actions');
  if (existsSync(actions)) for (const name of readdirSync(actions, {recursive: true}).filter(n => /\.ya?ml$/u.test(n)))
    problems.push(...checkSource(`.github/actions/${name}`, readFileSync(join(actions, name), 'utf8')));
  for (const problem of problems) console.log(`::error::${problem}`);
  if (problems.length) process.exit(1);
  console.log('Every sparse checkout includes the #[path] modules of the Rust scripts it lists.');
}
