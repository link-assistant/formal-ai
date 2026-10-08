#!/usr/bin/env node
// Derive Rust unit-test area moves from imported module families and doc comments.
// --write records a reviewed rename tree; --finish follows the existing rename
// tool with module lists, moved relative includes and test-duration path updates.
import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, posix, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadRenameMap } from './rename-by-rule.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const DIRECTORY = 'rust/tests/unit';
const MAP = 'data/meta/rename-map.lino';
const TREE = 'unit-tests-by-area';
const SHARD = `data/meta/rename-map/${TREE}.lino`;
const read = (path) => readFileSync(join(ROOT, path), 'utf8');
const tokens = (text) => text.toLowerCase().split(/[^a-z]+/u).filter((word) => word.length > 2);
const excludedNames = new Set(['mod', ...process.argv.filter((argument) => argument.startsWith('--exclude=')).map((argument) => argument.slice('--exclude='.length))]);
const abbreviations = new Map();
let collectingAbbreviations = false;
for (const line of read('data/meta/notation-rules.lino').split('\n')) {
  if (line === '  abbreviations') { collectingAbbreviations = true; continue; }
  if (collectingAbbreviations && /^ {0,2}\S/u.test(line)) break;
  if (collectingAbbreviations) {
    const match = /^    ([a-z0-9]+) ([a-z]+(?:-[a-z]+)*)$/u.exec(line);
    if (match) abbreviations.set(match[1], match[2]);
  }
}

function family(module) {
  const first = module.split('_')[0];
  if (['engine', 'solver', 'memory', 'seed', 'translation', 'formalization', 'reasoning', 'learning'].includes(first)) return first;
  if (first === 'cli') return 'command-line-interface';
  return module.split('_').map((word) => abbreviations.get(word) ?? word).join('-');
}

function assignments() {
  const names = readdirSync(join(ROOT, DIRECTORY)).filter((name) => name.endsWith('.rs'));
  const source = new Map(names.map((name) => [basename(name, '.rs'), read(`${DIRECTORY}/${name}`)]));
  const shared = new Set();
  for (const [name, text] of source) {
    // A top-level sibling reference keeps both sides in their original parent;
    // nested local super::* uses do not name a sibling and need no exception.
    for (const match of text.matchAll(/\bsuper::([a-z][a-z0-9_]*)/gu)) {
      if (source.has(match[1]) || existsSync(join(ROOT, DIRECTORY, match[1]))) {
        shared.add(name);
        shared.add(match[1]);
      }
    }
  }
  const rootModules = readdirSync(join(ROOT, 'rust/src'), { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs') || entry.isDirectory() && existsSync(join(ROOT, 'rust/src', entry.name, 'mod.rs')))
    .map((entry) => basename(entry.name, '.rs')).filter((name) => !['lib', 'main', 'formal_ai'].includes(name));
  const families = [...new Set(rootModules.map(family))];
  const exportedModules = new Map();
  for (const match of read('rust/src/lib.rs').matchAll(/pub use ([a-z][a-z0-9_]*)::\{([^}]+)\}/gu)) {
    for (const symbol of match[2].split(',').map((name) => name.trim()).filter(Boolean)) exportedModules.set(symbol, match[1]);
  }
  const proposed = [];
  for (const [name, text] of source) {
    if (excludedNames.has(name) || shared.has(name)
      || existsSync(join(ROOT, DIRECTORY, name))) continue;
    const imported = [...text.matchAll(/\bformal_ai::([a-z][a-z0-9_]*)/gu)].map((match) => match[1]);
    const publicImports = [...text.matchAll(/use formal_ai::\{([^}]+)\}/gu)]
      .flatMap((match) => match[1].split(',').map((name) => name.trim()))
      .map((name) => exportedModules.get(name)).filter(Boolean);
    const doc = text.split('\n').filter((line) => line.startsWith('//!')).join(' ');
    const description = new Set(tokens(`${name} ${doc}`));
    const scores = families.map((area) => ({ area,
      score: imported.filter((module) => family(module) === area).length * 10
        + publicImports.filter((module) => family(module) === area).length * 2
        + tokens(area).filter((word) => description.has(word)).length,
    })).filter(({ score }) => score > 0).sort((left, right) => right.score - left.score || left.area.localeCompare(right.area));
    if (!scores.length || (scores[1] && scores[0].score === scores[1].score)) continue;
    const area = scores[0].area;
    const module = area.replaceAll('-', '_');
    // Existing directory modules retain their own contract; do not merge a
    // newly derived area into them, or shadow an unmoved root module.
    if (existsSync(join(ROOT, DIRECTORY, area)) || existsSync(join(ROOT, DIRECTORY, module))
      || (source.has(module) && (shared.has(module) || excludedNames.has(module)))) continue;
    proposed.push({ from: `${DIRECTORY}/${name}.rs`, to: `${DIRECTORY}/${area}/${name}.rs`,
      area, reason: `imported module family and documentation evidence: ${area}; score ${scores[0].score}` });
  }
  const counts = new Map();
  for (const move of proposed) counts.set(move.area, (counts.get(move.area) ?? 0) + 1);
  return proposed.filter((move) => counts.get(move.area) >= 5
    && (!source.has(move.area.replaceAll('-', '_')) || proposed.some((candidate) => candidate.from === `${DIRECTORY}/${move.area.replaceAll('-', '_')}.rs` && candidate.area === move.area)));
}

function renderTree(moves) {
  return `\n  tree ${TREE}\n` + moves.map((move) => [
    '    rename', `      from "${move.from}"`, `      to "${move.to}"`, `      reason "${move.reason}"`,
  ].join('\n')).join('\n') + '\n';
}

function finish(moves) {
  const byName = new Map(moves.map((move) => [basename(move.from, '.rs'), move]));
  for (const move of moves) {
    const text = read(move.to);
    const rewritten = text.replace(/((?:include_str!|include_bytes!)\s*\(\s*|#\[path\s*=\s*)"([^"\n]+)"/gu,
      (whole, prefix, path) => {
        if (path.startsWith('/') || existsSync(join(ROOT, posix.normalize(posix.join(dirname(move.to), path))))) return whole;
        const target = posix.normalize(posix.join(dirname(move.from), path));
        const renamed = moves.find((candidate) => candidate.from === target)?.to ?? target;
        return `${prefix}"${posix.relative(dirname(move.to), renamed)}"`;
      });
    if (rewritten !== text) writeFileSync(join(ROOT, move.to), rewritten);
  }
  let moduleSource = read(`${DIRECTORY}/mod.rs`);
  for (const name of byName.keys()) {
    moduleSource = moduleSource.replace(new RegExp(`^mod ${name};\\n`, 'mu'), '');
  }
  const areas = [...new Set(moves.map((move) => basename(dirname(move.to))))].sort();
  for (const area of areas) {
    const members = moves.filter((move) => dirname(move.to) === `${DIRECTORY}/${area}`)
      .map((move) => basename(move.to, '.rs')).sort();
    const path = `${DIRECTORY}/${area}/mod.rs`;
    mkdirSync(join(ROOT, dirname(path)), { recursive: true });
    writeFileSync(join(ROOT, path), `//! Tests grouped by ${area.replaceAll('-', ' ')} module evidence.\n\n`
      + members.map((name) => `#[path = "${name}.rs"]\nmod ${name};\n`).join(''));
    const attribute = `#[path = "${area}/mod.rs"]`;
    const declaration = `mod ${area.replaceAll('-', '_')};`;
    moduleSource = moduleSource.split(`${attribute}\n`).join('').split(`${declaration}\n`).join('');
    moduleSource += `\n${attribute}\n${declaration}\n`;
  }
  writeFileSync(join(ROOT, DIRECTORY, 'mod.rs'), moduleSource);
  const files = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', '--', 'data/meta', '.github', 'scripts'], { cwd: ROOT, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }).trim().split('\n');
  for (const file of files) {
    if (!existsSync(join(ROOT, file)) || !/^(data\/meta\/test-durations\.lino|\.github\/|scripts\/)/u.test(file)
      || !/\.(?:lino|yml|yaml|mjs|rs|sh|py)$/u.test(file)) continue;
    let text = read(file);
    const before = text;
    for (const [name, move] of byName) {
      const area = basename(dirname(move.to)).replaceAll('-', '_');
      if (file === 'data/meta/test-durations.lino') {
        text = text.replace(new RegExp(`(\\btest ")${name}(?=::)`, 'gu'), `$1${area}::${name}`);
      }
      text = text.replace(new RegExp(`\\bunit::${name}::`, 'gu'), `unit::${area}::${name}::`);
      text = text.replace(new RegExp(`(--test unit )${name}(?=\\s|::|["'])`, 'gu'), `$1${area}::${name}`);
    }
    if (text !== before) writeFileSync(join(ROOT, file), text);
  }
}

function main() {
  if (process.argv.includes('--repair-map')) {
    const moves = loadRenameMap(ROOT).renames.filter((move) => move.tree === TREE)
      .map((move) => ({ ...move, from: `${DIRECTORY}/${basename(move.to)}` }));
    writeFileSync(join(ROOT, SHARD), `rename-map\n${renderTree(moves)}`);
    console.log(`restored ${moves.length} original root paths in the area shard`);
    return;
  }
  if (process.argv.includes('--finish')) {
    const moves = loadRenameMap(ROOT).renames.filter((move) => move.tree === TREE);
    finish(moves);
    console.log(`finished ${moves.length} area moves`);
    return;
  }
  const moves = assignments();
  const counts = new Map();
  for (const move of moves) counts.set(move.area, (counts.get(move.area) ?? 0) + 1);
  console.log(JSON.stringify(Object.fromEntries(counts), null, 2));
  console.log(`${moves.length} files proposed; ${readdirSync(join(ROOT, DIRECTORY)).filter((name) => name.endsWith('.rs')).length} top-level Rust files`);
  if (process.argv.includes('--write')) {
    const source = read(MAP);
    if (loadRenameMap(ROOT).renames.some((move) => move.tree === TREE)) throw new Error(`${TREE} already recorded`);
    mkdirSync(join(ROOT, dirname(SHARD)), { recursive: true });
    writeFileSync(join(ROOT, SHARD), `rename-map\n${renderTree(moves)}`);
    writeFileSync(join(ROOT, MAP), source + `  include "${SHARD}"\n`);
  }
}
if (import.meta.url === `file://${process.argv[1]}`) main();
