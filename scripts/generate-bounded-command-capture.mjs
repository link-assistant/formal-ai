import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const { parse } = createRequire(import.meta.url)('@babel/parser');
const input = path.join(root, 'js/server/bounded-command.mjs');
const output = path.join(root, 'data/meta/bounded-command-capture-source.lino');
const source = fs.readFileSync(input, 'utf8');
const tree = parse(source, { sourceType: 'module' });
const functions = tree.program.body
  .filter(node => node.type === 'ExportNamedDeclaration')
  .map(node => node.declaration)
  .filter(node => node?.type === 'FunctionDeclaration' && node.id?.name === 'boundedCommand');
assert.equal(functions.length, 1, 'canonical async supervisor must have exactly one declaration');
assert.equal(functions[0].async, true);
const body = source.slice(functions[0].start, functions[0].end);
// This is a readable code projection, not a second maintained implementation.
const expected = 'bounded-command-capture-source\n' + body.split('\n')
  .map(line => '  line ' + JSON.stringify(line)).join('\n') + '\n';
if (process.argv.includes('--check')) {
  assert.equal(fs.readFileSync(output, 'utf8'), expected, 'bounded command capture projection drift');
} else {
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(output, expected);
}
console.log('bounded command capture projection ' + (process.argv.includes('--check') ? 'verified' : 'generated'));
