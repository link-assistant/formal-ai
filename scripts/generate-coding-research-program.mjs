#!/usr/bin/env node
// One canonical program, checked portable-core IR, one generated native runtime.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { workflowPin } from './translate-js-rust.mjs';
import { writeLink } from './self-translation/lino.mjs';
import { lex, topLevelItems } from './self-translation/lexer.mjs';
import { lowerGeneratedRust } from './lib/research-rust-lowering.mjs';
import { namedImports, resolveImport } from './lib/translation-workarounds.mjs';
export const RESEARCH_SOURCES = [
  'js/agentic/portable/coding_research_query.mjs',
  'js/agentic/portable/coding_research_rebinding.mjs',
  'js/agentic/portable/coding_research_acceptance.mjs',
  'js/agentic/portable/coding_research_state.mjs',
  'js/agentic/portable/source_layout.mjs',
  'js/agentic/portable/source_maintenance_state.mjs',
];
export const RESEARCH_META = 'data/meta/coding-research-program.lino';
export const RESEARCH_NATIVE = 'rust/src/coding_research_generated';
const digest = value => createHash('sha256').update(value).digest('hex');
function portableObservation(value) {
  if (typeof value === 'bigint') return { integer: String(value) };
  if (value === undefined) return { absent: true };
  if (value instanceof Map) return { entries: [...value].map(([key, item]) => [key, portableObservation(item)]) };
  if (Array.isArray(value)) return value.map(portableObservation);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map(key => [key, portableObservation(value[key])]));
  }
  return value;
}
/** Loading is explicit and pinned; missing compiler never triggers an install or fetch. */
export async function loadResearchCompiler(directory, expectedPin) {
  assert(directory && /^[0-9a-f]{40}$/u.test(expectedPin), 'PinnedResearchCompilerRequired');
  const marker = path.join(directory, '.formal-ai-meta-language-commit');
  const actual = fs.existsSync(path.join(directory, '.git'))
    ? execFileSync('git', ['-C', directory, 'rev-parse', 'HEAD'], { encoding: 'utf8' }).trim()
    : fs.existsSync(marker) ? fs.readFileSync(marker, 'utf8').trim() : null;
  assert.equal(actual, expectedPin, 'ResearchCompilerPinMismatch');
  const load = file => import(pathToFileURL(path.join(directory, 'js/src/translation', file)).href);
  const [{ parseJavaScript }, { checkProgram }, { emitRust }] = await Promise.all([
    load('javascript.js'), load('check.js'), load('emit-rust.js'),
  ]);
  return { parseJavaScript, checkProgram, emitRust, pin: expectedPin };
}
/** Actual checked IR is the meta intermediate; this manifest records its source-bound identity. */
/** Text-only Rust formatting precedes structured file-size partitioning. */
function formatGeneratedRust(source) {
  return execFileSync('rustfmt', ['--edition', '2024', '--emit', 'stdout', '--config', 'skip_children=true'], {
    input: source, encoding: 'utf8', timeout: 15000, maxBuffer: 8388608,
  });
}
/** Prune only closed function-only prelude modules; opaque modules are retained whole. */
export function pruneUnreachablePreludeFunctions(preludes, definitions) {
  const removed = [];
  const rendered = preludes.map((source, preludeIndex) => {
    const roots = topLevelItems(lex(source, 'Rust')).filter(item => !item.comment);
    if (roots.length !== 1) return source;
    const tokens = roots[0].tokens.filter(token => token.type !== 'comment');
    if (tokens[0]?.text !== 'pub' || tokens[1]?.text !== 'mod' || tokens[2]?.type !== 'ident'
        || tokens[3]?.text !== '{' || tokens.at(-1)?.text !== '}') return source;
    const bodyStart = tokens[3].end, bodyEnd = tokens.at(-1).start;
    const body = source.slice(bodyStart, bodyEnd);
    const functions = [];
    let comments = [];
    for (const item of topLevelItems(lex(body, 'Rust'))) {
      if (item.comment) { comments.push(item); continue; }
      const itemTokens = item.tokens.filter(token => token.type !== 'comment');
      if (itemTokens[0]?.text !== 'pub' || itemTokens[1]?.text !== 'fn' || itemTokens[2]?.type !== 'ident'
          || itemTokens.some(token => token.type === 'ident' && token.text.endsWith('!'))) return source;
      functions.push({ name: itemTokens[2].text, start: comments[0]?.start ?? item.start,
        end: item.end, tokens: itemTokens });
      comments = [];
    }
    if (!functions.length || new Set(functions.map(item => item.name)).size !== functions.length) return source;
    const outside = [...preludes.filter((_, index) => index !== preludeIndex), ...definitions].join('\n');
    const observed = lex(outside, 'Rust').filter(token => token.type === 'ident').map(token => token.text);
    const reachable = new Set(functions.filter(item => observed.includes(item.name)).map(item => item.name));
    let changed = true;
    while (changed) {
      changed = false;
      for (const item of functions) {
        if (!reachable.has(item.name)) continue;
        for (const dependency of functions) {
          if (!reachable.has(dependency.name) && item.tokens.some(token =>
            token.type === 'ident' && token.text === dependency.name)) {
            reachable.add(dependency.name); changed = true;
          }
        }
      }
    }
    let result = source;
    for (const item of [...functions].reverse()) {
      if (reachable.has(item.name)) continue;
      const start = bodyStart + item.start, end = bodyStart + item.end;
      result = result.slice(0, start) + result.slice(end);
      removed.push(tokens[2].text + '::' + item.name);
    }
    return result;
  });
  return { preludes: rendered, removed: removed.sort() };
}
/** Mechanical namespace relocation over emitted items; strings/comments remain exact data. */
export function relocateGeneratedRustNamespace(definitions, namespace) {
  assert(/^[a-z][a-z0-9_]*$/u.test(namespace), 'GeneratedNamespaceIdentifierRequired');
  const symbols = new Set();
  const declarations = new Set(['fn', 'mod', 'enum', 'struct', 'const', 'static', 'type']);
  for (const definition of definitions) {
    for (const item of topLevelItems(lex(definition, 'Rust'))) {
      if (item.comment) continue;
      const tokens = item.tokens.filter(token => token.type !== 'comment');
      const head = tokens.findIndex(token => token.type === 'ident' && declarations.has(token.text));
      assert(head >= 0 && tokens[head + 1]?.type === 'ident', 'OpaqueGeneratedRootDeclaration');
      const name = tokens[head + 1].text;
      assert(!symbols.has(name), 'DuplicateGeneratedRootDeclaration:' + name);
      symbols.add(name);
    }
  }
  assert(symbols.size > 0, 'GeneratedNamespaceSymbolTableRequired');
  let references = 0;
  const relocated = definitions.map(source => {
    const tokens = lex(source, 'Rust');
    const significant = tokens.filter(token => token.type !== 'comment');
    const insertions = [];
    for (let index = 0; index + 2 < significant.length; index += 1) {
      const [root, separator, name] = significant.slice(index, index + 3);
      if (root.type !== 'ident' || root.text !== 'crate' || separator.text !== '::') continue;
      assert(name.type === 'ident' && symbols.has(name.text), 'UnknownGeneratedNamespaceReference:' + name.text);
      insertions.push(name.start);
    }
    const prefix = namespace + '::';
    let rendered = source;
    for (const position of [...insertions].reverse()) rendered = rendered.slice(0, position) + prefix + rendered.slice(position);
    const positions = new Set(insertions);
    const expected = tokens.flatMap(token => positions.has(token.start)
      ? [['ident', namespace], ['punct', '::'], [token.type, token.text]] : [[token.type, token.text]]);
    assert.deepEqual(lex(rendered, 'Rust').map(token => [token.type, token.text]), expected, 'GeneratedNamespaceTokenDrift');
    let inverse = rendered;
    for (let index = insertions.length - 1; index >= 0; index -= 1) {
      const position = insertions[index] + index * prefix.length;
      assert.equal(inverse.slice(position, position + prefix.length), prefix, 'GeneratedNamespaceInverseSpan');
      inverse = inverse.slice(0, position) + inverse.slice(position + prefix.length);
    }
    assert.equal(inverse, source, 'GeneratedNamespaceInverseDrift');
    references += insertions.length;
    return rendered;
  });
  return { definitions: relocated, symbols: [...symbols].sort(), references, namespace };
}

export function generateCodingResearchProgram(sources, compiler, profile = null) {
  const expectedSources = profile?.sources ?? RESEARCH_SOURCES;
  const nativeDirectory = profile?.nativeDirectory ?? RESEARCH_NATIVE;
  const metaPath = profile?.metaPath ?? RESEARCH_META;
  const namespace = profile?.namespace ?? 'coding_research_generated';
  const generatorPath = profile?.generatorPath ?? 'scripts/generate-coding-research-program.mjs';
  const metaRecord = profile?.metaRecord ?? 'coding-research-program';
  assert.deepEqual(sources.map(row => row.path), expectedSources, 'CompleteCanonicalResearchSourcesRequired');
  const known = new Set(sources.map(row => row.path));
  const flattened = sources.map(row => {
    const imports = namedImports(row.source);
    for (const imported of imports) {
      const target = resolveImport(row.path, imported.specifier, known);
      assert(target, 'ExternalCanonicalResearchImport');
      const unit = sources.find(candidate => candidate.path === target);
      for (const binding of imported.names) {
        assert.equal(binding.imported, binding.local, 'AliasedCanonicalResearchImport');
        assert(unit.source.includes('export function ' + binding.imported + '('), 'UnknownCanonicalResearchImport');
      }
    }
    const ranges = topLevelItems(lex(row.source, 'JavaScript')).filter(item =>
      !item.comment && item.tokens.filter(token => token.type !== 'comment')[0]?.text === 'import');
    assert.equal(ranges.length, imports.length, 'OpaqueCanonicalResearchImport');
    let source = row.source;
    for (const range of ranges.reverse()) source = source.slice(0, range.start) + source.slice(range.end);
    return source;
  });
  const source = flattened.join('\n');
  const checked = compiler.checkProgram(compiler.parseJavaScript(source));
  assert.equal(checked.main?.effects.length ?? 0, 0, 'EffectfulCanonicalResearchProgram');
  const emitted = compiler.emitRust(checked);
  const pruned = pruneUnreachablePreludeFunctions(emitted.preludes, emitted.definitions);
  const relocated = relocateGeneratedRustNamespace([...pruned.preludes, ...emitted.definitions], namespace);
  assert.equal((checked.externals ?? []).length, 0, 'ExternalGeneratedOwnershipContract');
  const lowering = relocated.definitions.map(definition => lowerGeneratedRust(definition, {
    ownedValueModel: 'pinned-meta-language-owned-values/v1', pure: true,
  }));
  const loweringImplementationHash = digest(fs.readFileSync(new URL('./lib/research-rust-lowering.mjs', import.meta.url)));
  const sourceHash = digest(source);
  const irHash = digest(JSON.stringify(portableObservation(checked)));
  const header = (profile?.description ? '//! ' + profile.description + '\n' : '')
    + '// Generated by ' + generatorPath + '; do not edit.\n'
    + '// source-sha256 ' + sourceHash + '; checked-ir-sha256 ' + irHash + '\n'
    + '// upstream-commit ' + compiler.pin + '\n'
    + '// native-namespace crate::' + namespace + '; token-checked relocation\n'
    + '// checked-owned-value-lowering-sha256 ' + loweringImplementationHash + '\n'
    + '#![allow(unused, unreachable_patterns, non_snake_case, non_camel_case_types, invalid_nan_comparisons)]\n';
  const groups = [];
  let group = [], lines = 0;
  for (const {source:rawDefinition} of lowering) {
    const definition = formatGeneratedRust(rawDefinition);
    const amount = definition.split('\n').length + 2;
    assert(amount < 900, 'GeneratedResearchDefinitionNeedsStructuredSplit');
    if (lines + amount > 850 && group.length) { groups.push(group); group = []; lines = 0; }
    group.push(definition); lines += amount;
  }
  if (group.length) groups.push(group);
  assert(groups.length > 0, 'EmptyGeneratedResearchProgram');
  const outputs = {};
  const modules = [];
  for (let index = 0; index < groups.length; index += 1) {
    const name = 'part' + (index + 1);
    outputs[nativeDirectory + '/' + name + '.rs'] = formatGeneratedRust(header + groups[index].join('\n\n') + '\n');
    modules.push('mod ' + name + ';', 'pub use ' + name + '::*;');
  }
  outputs[nativeDirectory + '/mod.rs'] = formatGeneratedRust(header + modules.join('\n') + '\n');
  outputs[metaPath] = '# Source/provenance observation of the actual checked portable-core IR.\n'
    + ['(' + metaRecord,
      ...[['source-sha256', sourceHash], ['checked-ir-sha256', irHash],
        ['upstream-commit', compiler.pin], ['runtime-count', '1'],
        ...sources.map(row => ['source-unit', ['path', { quoted: row.path }], ['sha256', digest(row.source)]])]
        .map(link => '  ' + writeLink(link)), ')'].join('\n') + '\n';
  return { outputs, sourceHash, irHash, compilerPin: compiler.pin,
    assumptions: emitted.assumptions, mappings: emitted.mappings, nativeParts: groups.length,
    lowering: {implementationSha256:loweringImplementationHash,
      operations:lowering.flatMap(row=>row.operations),
      eliminatedClones:lowering.flatMap(row=>row.eliminatedClones),
      allowances:lowering.flatMap(row=>row.allowances)},
    relocation: { namespace: relocated.namespace, references: relocated.references, symbols: relocated.symbols, prunedPreludeFunctions: pruned.removed } };
}
export async function checkCodingResearchProgram(directory, compilerDirectory, write = false, profile = null) {
  const pin = workflowPin(fs.readFileSync(path.join(directory, '.github/workflows/layered-ci.yml'), 'utf8'));
  const compiler = await loadResearchCompiler(compilerDirectory, pin);
  const sources = (profile?.sources ?? RESEARCH_SOURCES).map(relative => ({ path: relative,
    source: fs.readFileSync(path.join(directory, relative), 'utf8') }));
  const result = generateCodingResearchProgram(sources, compiler, profile);
  const drift = [];
  for (const [relative, text] of Object.entries(result.outputs)) {
    const full = path.join(directory, relative);
    if (!fs.existsSync(full) || fs.readFileSync(full, 'utf8') !== text) {
      drift.push(relative);
      if (write) { fs.mkdirSync(path.dirname(full), { recursive: true }); fs.writeFileSync(full, text); }
    }
  }
  return { ...result, drift };
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const mode = process.argv[2] ?? '--check';
  assert(['--check', '--write'].includes(mode));
  const flag = process.argv.indexOf('--meta-language');
  const directory = flag >= 0 ? process.argv[flag + 1] : process.env.FORMAL_AI_META_LANGUAGE;
  const result = await checkCodingResearchProgram(root, directory, mode === '--write');
  if (mode === '--check' && result.drift.length) {
    console.error('Canonical research program drift: ' + result.drift.join(', '));
    process.exitCode = 1;
  } else console.log('Canonical research program matches pinned checked IR');
}
