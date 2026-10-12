import { readFileSync, lstatSync, realpathSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import {
  loadRenameMap, indexByBasename, companionMoves, moduleDirectoryMoves,
  findReferences, renamedToken, renameRustModules,
  renameSeedRegistryStems, resortQuotedLists,
} from './rename-by-rule.mjs';
import { fullWordPath } from '../../scripts/measure-abbreviations.mjs';
import { readRules } from '../../scripts/lib/links-notation-names.mjs';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
function safePath(value) {
  if (typeof value !== 'string' || !value || value.includes('\\') ||
      value.startsWith('/') || value.split('/').some(part =>
        !part || part === '.' || part === '..')) {
    throw new Error('Unsafe selected source path');
  }
  return value;
}
function physicalSource(root, path) {
  const parts = safePath(path).split('/');
  for (let index = 1; index <= parts.length; index += 1) {
    const stat = lstatSync(join(root, ...parts.slice(0, index)));
    if (stat.isSymbolicLink()) throw new Error('Aliased selected source');
    if (index === parts.length && !stat.isFile()) {
      throw new Error('Selected source is not a regular file');
    }
  }
  const bytes = readFileSync(join(root, path));
  const text = bytes.toString('utf8');
  if (!Buffer.from(text).equals(bytes)) throw new Error('Invalid UTF-8 source');
  return { text, sha256: digest(bytes), bytes: bytes.length };
}

function guardDestination(root, path) {
  const parts = safePath(path).split('/');
  for (let index = 1; index <= parts.length; index += 1) {
    let stat;
    try {
      stat = lstatSync(join(root, ...parts.slice(0, index)));
    } catch (error) {
      if (error.code === 'ENOENT') return;
      throw error;
    }
    if (stat.isSymbolicLink()) throw new Error('Aliased rename destination');
    if (index === parts.length) throw new Error('Physical destination collision');
    if (!stat.isDirectory()) throw new Error('Unsupported rename parent');
  }
}

const transformationPaths = [
  'experiments/formal_ai_subagent/rename-by-rule.mjs',
  'scripts/measure-abbreviations.mjs',
  'scripts/lib/links-notation-names.mjs',
];
const transformationRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const importedOrigins = new Map(transformationPaths.map(path =>
  [path, physicalSource(transformationRoot, path).sha256]));
const aliasTransformationPaths = [
  'experiments/formal_ai_subagent/source-qualified-rust-aliases.mjs',
  'scripts/self-translation/lexer.mjs',
];
const importedAliasOrigins = new Map(aliasTransformationPaths.map(path =>
  [path, physicalSource(transformationRoot, path).sha256]));
function verifyTransformationOrigins(owners) {
  for (const path of transformationPaths) {
    const imported = importedOrigins.get(path);
    if (!owners.has(path) || owners.get(path).sha256 !== imported ||
        physicalSource(transformationRoot, path).sha256 !== imported) {
      throw new Error('Transformation origin mismatch');
    }
  }
}

// The inventory is a guarded, explicitly selected reference universe.
// This pure planner never asserts repository-wide inventory completeness.
export function selectedRenamePlan(root, manifest) {
  root = resolve(root);
  if (realpathSync(root) !== root || lstatSync(root).isSymbolicLink()) throw new Error('Aliased source root');
  const inventory = manifest.inventory.map(safePath);
  if (new Set(inventory).size !== inventory.length) {
    throw new Error('Duplicate inventory path');
  }
  if (digest(JSON.stringify(inventory)) !== manifest.inventorySha256) {
    throw new Error('Inventory drift');
  }
  const owners = new Map();
  for (const row of manifest.sources) {
    const path = safePath(row.path);
    if (owners.has(path) || !inventory.includes(path)) {
      throw new Error('Duplicate or unlisted source');
    }
    const before = physicalSource(root, path);
    if (before.sha256 !== row.sha256) throw new Error('Source drift');
    owners.set(path, before);
  }
  const dependencies = [
    'experiments/formal_ai_subagent/rename-by-rule.mjs',
    'scripts/measure-abbreviations.mjs',
    'scripts/lib/links-notation-names.mjs',
    'data/meta/abbreviation-rules.lino',
  ];
  for (const path of dependencies) {
    if (!owners.has(path)) throw new Error('Unbound transformation dependency');
  }
  verifyTransformationOrigins(owners);
  const visited = new Set();
  function bindMap(path) {
    safePath(path);
    if (visited.has(path)) return;
    if (!owners.has(path)) throw new Error('Unbound rename map');
    visited.add(path);
    for (const match of owners.get(path).text.matchAll(
      /^\s*include "([^"]+)"\s*$/gmu)) bindMap(match[1]);
  }
  bindMap('data/meta/rename-map.lino');
  const dictionaryPath = 'data/meta/notation-rules.lino';
  if (!owners.has(dictionaryPath)) throw new Error('Unbound dictionary');
  const dictionary = readRules(owners.get(dictionaryPath).text).abbreviations;
  const prefix = safePath(manifest.prefix) + '/';
  const primary = inventory.filter(path => path.startsWith(prefix) &&
    path.endsWith('.rs')).map(from => ({
      from, to: fullWordPath(from, dictionary),
    })).filter(move => move.from !== move.to);
  const moves = primary.flatMap(move => companionMoves(move,
    path => inventory.includes(path))).flatMap(move => [move,
    ...moduleDirectoryMoves(move, directory => inventory.filter(path =>
      path.startsWith(directory + '/')))]);
  const destinations = new Set();
  for (const move of moves) {
    if (!owners.has(move.from)) throw new Error('Missing moved source');
    if (inventory.includes(move.to) || destinations.has(move.to)) {
      throw new Error('Rename destination collision');
    }
    guardDestination(root, move.to);
    destinations.add(move.to);
  }
  const hasNamespaceSources = [...owners.keys()].some(path => path.endsWith('/src/lib.rs'));
  if (hasNamespaceSources) {
    for (const path of aliasTransformationPaths) {
      if (!owners.has(path) || owners.get(path).sha256 !== importedAliasOrigins.get(path) ||
          physicalSource(transformationRoot, path).sha256 !== importedAliasOrigins.get(path)) {
        throw new Error('Unbound alias transformation origin');
      }
    }
  }
  const namespaceSources = hasNamespaceSources
    ? new Map([...owners].map(([path, source]) => [path, source.text])) : undefined;
  const index = indexByBasename(inventory);
  const map = loadRenameMap(root);
  const rows = [];
  for (const [path, before] of owners) {
    let after = before.text;
    if (!map.exclude.some(prefix => path.startsWith(prefix))) {
      for (const reference of findReferences(after, path, moves, index).reverse()) {
        after = after.slice(0, reference.start) + renamedToken(reference.token,
          reference.move.from, reference.move.to) + after.slice(reference.end);
      }
      after = renameRustModules(after, path, moves, index, namespaceSources).text;
      after = renameSeedRegistryStems(after, path, moves).text;
      if (map.resort.includes(path)) after = resortQuotedLists(after);
    }
    const move = moves.find(move => move.from === path);
    if (move || after !== before.text) rows.push({
      beforePath: path, afterPath: move?.to ?? path,
      beforeSha256: before.sha256, afterSha256: digest(after),
      before: before.text, after, inverse: before.text,
    });
  }
  // Recheck every selected physical origin after deriving the plan.
  for (const [path, before] of owners) {
    if (physicalSource(root, path).sha256 !== before.sha256) {
      throw new Error('Concurrent selected source drift');
    }
  }
  verifyTransformationOrigins(owners);
  return { kind: 'SelectedRenamePlan', inventorySha256: manifest.inventorySha256,
    completeness: 'SelectedReferencesOnly', namespaceRaceProof: false,
    nativeCompilation: 'Pending', moves, rows };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, root, inputPath, expectedPath] = process.argv.slice(2);
  if (!['--plan', '--check'].includes(mode) || !root || !inputPath ||
      (mode === '--check' && !expectedPath)) {
    throw new Error('Usage: selected-rename-by-rule --plan|--check root manifest [expected-plan]');
  }
  const plan = selectedRenamePlan(root, JSON.parse(readFileSync(inputPath, 'utf8')));
  if (mode === '--check') {
    const expected = JSON.parse(readFileSync(expectedPath, 'utf8'));
    if (JSON.stringify(plan) !== JSON.stringify(expected)) {
      throw new Error('Generated selected rename plan differs');
    }
    console.log('Selected rename plan --check passed; repository completeness Unknown');
  } else console.log(JSON.stringify(plan, null, 2));
}
