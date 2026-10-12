import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(path.join(root, 'package.json'));
const {
    parse
  } = require('@babel/parser'),
  YAML = require('yaml');
export function literalSourceList(source, name) {
  assert(Buffer.byteLength(source) <= 1048576, 'bounded source required');
  const ast = parse(source, {
      sourceType: 'module'
    }),
    found = [];
  function visit(node) {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'VariableDeclarator' && node.id?.type === 'Identifier' && node.id.name === name) found.push(node.init);
    for (const [key, value] of Object.entries(node)) {
      if (['loc', 'start', 'end', 'extra'].includes(key)) continue;
      if (Array.isArray(value)) value.forEach(visit);else if (value && typeof value === 'object') visit(value);
    }
  }
  visit(ast);
  assert.equal(found.length, 1, 'unique source-owned list required');
  const list = found[0];
  assert.equal(list.type, 'ArrayExpression');
  assert(list.elements.length > 0 && list.elements.length <= 256);
  const values = list.elements.map(node => {
    assert.equal(node?.type, 'StringLiteral', 'unknown source expression refused');
    assert.match(node.value, /^[A-Za-z0-9_.\/-]+$/u);
    assert(!node.value.split('/').includes('..'));
    return node.value;
  });
  assert.equal(new Set(values).size, values.length);
  return values;
}
export function requiredCensusSelectors(sourceRoot = root) {
  const native = literalSourceList(fs.readFileSync(path.join(sourceRoot, 'scripts/lib/qualified-native-data-collector.mjs'), 'utf8'), 'nativeRoots');
  const implementation = literalSourceList(fs.readFileSync(path.join(sourceRoot, 'scripts/collect-qualified-native-data-artifacts.mjs'), 'utf8'), 'implementationPaths').map(p => 'scripts/' + p);
  const selected = native.flatMap(p => [p, p + '/**']);
  return [...new Set([...selected, ...implementation, 'scripts/generate-census-source-selectors.mjs', 'rust/tests/web/census-source-selectors.test.mjs'])].sort();
}
export function checkedCensusSelectorRepair(text, required) {
  assert(Buffer.byteLength(text) <= 1048576);
  const doc = YAML.parseDocument(text);
  assert.equal(doc.errors.length, 0);
  const item = doc.getIn(['on', 'pull_request', 'paths'], true);
  assert(item?.type === 'SEQ' && Array.isArray(item.items));
  const values = item.items.map(p => {
    assert(['PLAIN', 'QUOTE_SINGLE', 'QUOTE_DOUBLE'].includes(p?.type) && typeof p.value === 'string');
    return p.value;
  });
  assert.equal(new Set(values).size, values.length);
  assert(required.length > 0 && required.length <= 512 && new Set(required).size === required.length);
  for (const p of required) assert.match(p, /^[A-Za-z0-9_./*-]+$/u);
  const missing = required.filter(p => !values.includes(p));
  if (!missing.length) return text;
  const last = item.items.at(-1);
  assert(last?.range);
  const at = last.range[1],
    lineStart = text.lastIndexOf('\n', last.range[0]) + 1;
  const prefix = text.slice(lineStart, last.range[0]);
  assert(/^ +-[ ]$/u.test(prefix));
  const indent = prefix.slice(0, -2);
  const output = text.slice(0, at) + missing.map(p => '\n' + indent + '- ' + JSON.stringify(p)).join('') + text.slice(at);
  const before = doc.toJSON(),
    after = YAML.parse(output);
  assert.deepEqual(after.on.pull_request.paths, [...values, ...missing]);
  delete before.on.pull_request.paths;
  delete after.on.pull_request.paths;
  assert.deepEqual(after, before, 'non-selector source semantics changed');
  return output;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert.deepEqual(process.argv.slice(2), ['--check']);
  const file = path.join(root, '.github/workflows/regenerate-self-ast-census.yml'),
    text = fs.readFileSync(file, 'utf8');
  assert.equal(checkedCensusSelectorRepair(text, requiredCensusSelectors()), text, 'census source selectors stale');
  console.log('census source selectors qualified');
}
