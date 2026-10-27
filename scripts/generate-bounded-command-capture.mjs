import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import * as formalAiBabelParser from '@babel/parser';
import {formatLinoValue} from '../js/agentic/crate/links_format.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const { parse } = formalAiBabelParser;
/** One source projection serves both repository and native packaged consumers. */
export function generateBoundedCommandCapture({repository = root, check = false} = {}) {
  const input = path.join(repository, 'js/server/bounded-command.mjs');
  const relative = 'data/meta/bounded-command-capture-source.lino';
  const source = fs.readFileSync(input, 'utf8');
  const tree = parse(source, { sourceType: 'module' });
  const functions = tree.program.body
    .filter(node => node.type === 'ExportNamedDeclaration')
    .map(node => node.declaration)
    .filter(node => node?.type === 'FunctionDeclaration' && node.id?.name === 'boundedCommand');
  assert.equal(functions.length, 1, 'canonical async supervisor must have exactly one declaration');
  assert.equal(functions[0].async, true);
  const body = source.slice(functions[0].start, functions[0].end);
  // Preserve actual source content; neither a counter nor a second implementation.
  const expected = 'bounded-command-capture-source\n' + body.split('\n')
    .map(line => '  line ' + formatLinoValue(line)).join('\n') + '\n';
  const outputs = [path.join(repository, relative), path.join(repository, 'rust/embedded', relative)];
  for (const output of outputs) {
    if (check) assert.equal(fs.readFileSync(output, 'utf8'), expected, 'bounded command capture projection drift: ' + path.relative(repository, output));
    else {fs.mkdirSync(path.dirname(output), {recursive:true});fs.writeFileSync(output, expected);}
  }
  return outputs;
}
if (process.argv[1] && fs.realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  assert(args.length === 0 || args.length === 1 && args[0] === '--check', 'expected no arguments or --check');
  generateBoundedCommandCapture({check:args.includes('--check')});
  console.log('bounded command capture projection ' + (args.includes('--check') ? 'verified' : 'generated'));
}
