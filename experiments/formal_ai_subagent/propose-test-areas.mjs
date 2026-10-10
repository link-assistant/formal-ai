#!/usr/bin/env node
// Derive Rust unit-test area moves from imported module families and doc comments.
// --write records a reviewed rename tree; --finish follows the existing rename
// tool with module lists, moved relative includes and test-duration path updates.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
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

function moduleFamily(module, vocabulary = abbreviations) {
  const first = module.split('_')[0];
  if (['engine', 'solver', 'memory', 'seed', 'translation', 'formalization', 'reasoning', 'learning'].includes(first)) return first;
  if (first === 'cli') return 'command-line-interface';
  return module.split('_').map((word) => vocabulary.get(word) ?? word).join('-');
}

export function testAreaInventory(sourceRoot = ROOT, exclusions = [...excludedNames]) {
  const excluded = new Set(['mod', ...exclusions]);
  const read = path => readFileSync(join(sourceRoot, path), 'utf8');
  const records = [];
  const bindings = new Map();
  const sha = source => createHash('sha256').update(source).digest('hex');
  const observe = path => { const source = read(path); bindings.set(path, sha(source)); return source; };
  observe('rust/src/lib.rs');
  const notation = observe('data/meta/notation-rules.lino');
  const sourceAbbreviations = new Map();
  let collecting = false;
  for (const line of notation.split('\n')) {
    if (line === '  abbreviations') { collecting = true; continue; }
    if (collecting && /^ {0,2}\S/u.test(line)) break;
    if (collecting) { const match = /^    ([a-z0-9]+) ([a-z]+(?:-[a-z]+)*)$/u.exec(line);
      if (match) sourceAbbreviations.set(match[1],match[2]); }
  }
  const family = module => moduleFamily(module, sourceAbbreviations);
  const defer = (name, reason, evidence = {}) => records.push({path: `${DIRECTORY}/${name}.rs`,
    sourceSha256: sha(source.get(name)), status: 'Deferred', reason, ...evidence});
  const names = readdirSync(join(sourceRoot, DIRECTORY)).filter((name) => name.endsWith('.rs')).sort();
  const source = new Map(names.map((name) => [basename(name, '.rs'), observe(`${DIRECTORY}/${name}`)]));
  const shared = new Set();
  for (const [name, text] of source) {
    // A top-level sibling reference keeps both sides in their original parent;
    // nested local super::* uses do not name a sibling and need no exception.
    for (const match of text.matchAll(/\bsuper::([a-z][a-z0-9_]*)/gu)) {
      if (source.has(match[1]) || existsSync(join(sourceRoot, DIRECTORY, match[1]))) {
        shared.add(name);
        shared.add(match[1]);
      }
    }
  }
  const rootModules = readdirSync(join(sourceRoot, 'rust/src'), { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.rs') || entry.isDirectory() && existsSync(join(sourceRoot, 'rust/src', entry.name, 'mod.rs')))
    .map((entry) => basename(entry.name, '.rs')).filter((name) => !['lib', 'main', 'formal_ai'].includes(name));
  const families = [...new Set(rootModules.map(family))];
  const exportedModules = new Map();
  for (const match of observe('rust/src/lib.rs').matchAll(/pub use ([a-z][a-z0-9_]*)::\{([^}]+)\}/gu)) {
    for (const symbol of match[2].split(',').map((name) => name.trim()).filter(Boolean)) exportedModules.set(symbol, match[1]);
  }
  const proposed = [];
  for (const [name, text] of source) {
    if (excluded.has(name)) { defer(name, 'excluded-module'); continue; }
    if (shared.has(name)) { defer(name, 'shared-sibling-contract'); continue; }
    if (existsSync(join(sourceRoot, DIRECTORY, name))) { defer(name, 'existing-module-directory'); continue; }
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
    if (!scores.length) { defer(name, 'no-source-area-evidence'); continue; }
    if (scores[1] && scores[0].score === scores[1].score) { defer(name, 'tied-area-evidence', {scores}); continue; }
    const area = scores[0].area;
    const module = area.replaceAll('-', '_');
    // Existing directory modules retain their own contract; do not merge a
    // newly derived area into them, or shadow an unmoved root module.
    if (existsSync(join(sourceRoot, DIRECTORY, area)) || existsSync(join(sourceRoot, DIRECTORY, module))
      || (source.has(module) && (shared.has(module) || excluded.has(module)))) {
      defer(name, 'existing-or-protected-area', {scores}); continue;
    }
    proposed.push({ from: `${DIRECTORY}/${name}.rs`, to: `${DIRECTORY}/${area}/${name}.rs`,
      area, reason: `imported module family and documentation evidence: ${area}; score ${scores[0].score}` });
  }
  const counts = new Map();
  for (const move of proposed) counts.set(move.area, (counts.get(move.area) ?? 0) + 1);
  const moves = proposed.filter((move) => counts.get(move.area) >= 5
    && (!source.has(move.area.replaceAll('-', '_')) || proposed.some((candidate) => candidate.from === `${DIRECTORY}/${move.area.replaceAll('-', '_')}.rs` && candidate.area === move.area)));
  for (const move of proposed) {
    const name = basename(move.from, '.rs');
    if (moves.includes(move)) records.push({path:move.from, sourceSha256:sha(source.get(name)),
      status:'Proposed', area:move.area, destination:move.to, reason:move.reason});
    else defer(name, counts.get(move.area) < 5 ? 'area-below-minimum-five' : 'unmoved-root-module-shadow', {area:move.area});
  }
  // Directory existence is structural evidence and must be frozen as well as file bytes.
  const structure = {
    tests:readdirSync(join(sourceRoot,DIRECTORY),{withFileTypes:true}).map(entry=>({name:entry.name,directory:entry.isDirectory()})).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0),
    modules:[...rootModules].sort(),
  };
  return {schema:'source-derived-unit-test-area-inventory/v1',
    generatorSha256:sha(readFileSync(fileURLToPath(import.meta.url))),
    exclusions:[...excluded].sort(), minimumAreaFiles:5, structure,
    bindings:[...bindings].sort(([a],[b])=>a<b?-1:a>b?1:0).map(([path,sha256])=>({path,sha256})),
    records:records.sort((a,b)=>a.path<b.path?-1:a.path>b.path?1:0), moves};
}

export function checkTestAreaInventory(recorded, sourceRoot = ROOT, exclusions = [...excludedNames]) {
  const current = testAreaInventory(sourceRoot, exclusions);
  if (JSON.stringify(recorded) !== JSON.stringify(current)) throw new Error('stale or counterfeit test-area inventory');
  return current;
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
  if (process.argv.includes('--inventory')) {
    const output = process.argv.find(argument=>argument.startsWith('--output='))?.slice('--output='.length);
    if (!output) throw new Error('inventory requires an explicit output path');
    const sourceRoot = process.argv.find(argument=>argument.startsWith('--source-root='))?.slice('--source-root='.length) ?? ROOT;
    const inventory = testAreaInventory(sourceRoot);
    if (process.argv.includes('--check')) checkTestAreaInventory(JSON.parse(readFileSync(output,'utf8')), sourceRoot);
    else if (process.argv.includes('--write')) writeFileSync(output, JSON.stringify(inventory,null,2)+'\n');
    else throw new Error('inventory requires --write or --check');
    console.log(`qualified ${inventory.records.length} source-bound unit-test inventory records`);
    return;
  }
  if (process.argv.includes('--check')) throw new Error('--check requires an explicit inventory contract');
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
  const moves = testAreaInventory().moves;
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
