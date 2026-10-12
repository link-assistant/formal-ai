import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import {evaluateTemporaryInterpreter,
 TEMPORARY_INTERPRETER_PROFILES} from '../../../scripts/self-translation/ir.mjs';
const run = source => evaluateTemporaryInterpreter(source,
 'owned-string-array-v1');

test('owned dense-string operations match independent JavaScript values and fresh execution',
 async () => {
  const cases = ["function f(){const a=[];a.push('a','b');return a.join(',');}",
    "function f(){const a=['😀','雪'];const s='\\n';a.push(s);return a.join(s);}",
    "function f(){const a=['x',];a.push();return a.length;}",
    "function f(){const a=[];return a.push('x');}",
 "function f(){const a=[];return a.join('');}"];
  for (const source of cases) {
    const expected = vm.runInNewContext(source + ';f()');
    const first = await run(source),
 second = await run(source);
    assert.equal(first.state,
 'Evaluated');
 assert.equal(first.value,
 expected);
 assert.equal(second.value,
 expected);
    assert.notEqual(first.ir,
 second.ir);
 assert.equal(first.ir.effects,
 'OwnedLocalOnly');
 assert.equal(first.admitted,
 false);
    assert.equal(Object.isFrozen(first.ir.operations),
 true);
 assert.equal(first.ir.sourceBytes,
 Buffer.byteLength(source));
  }
});

test('unsupported roles refuse before any evaluation or partial mutation',
 async () => {
  const cases = ["function f(input){const a=[];a.push('x');return a.join('');}",
    "async function f(){const a=[];return a.join('');}",
 "function f(){const a=[];const b=a;return b.join('');}",
    "function f(){const a=[];a.push('x');unknown();return a.join('');}",
 "function f(){const a=[];return a.map(x=>x);}",
    "function f(){const a=[,];return a.join('');}",
 "function f(){const a=[...other];return a.join('');}",
    "function f(){const a=[];return a['join']('');}",
 "function f(){const a=[];a.x='y';return a.join('');}",
    "function f(){const a=[];return a;}",
 "function f(){const a=[];if(true)a.push('x');return a.join('');}",
    "function f(){const a=[];function g(){}return a.join('');}",
 "function f(){const a=[];return /x/;}",
    'function f(){const a=[];return `x`;}',
 "function f(){const a=[];return\na.join('');}",
    "function f(){const a=[];return a.join('')}",
 "function f(){const a=[];return a.join('');}f();",
    "import x from 'external';function f(){return 'x';}",
 "function f(){const a=[];const a=[];return a.length;}"];
  for (const source of cases) {
    const result = await run(source);
    assert.equal(result.state,
 'Unsupported',
 source);
 assert.equal(result.evaluated,
 false);
 assert.equal(result.ir,
 null);
  }
});

test('canonical literal boundaries and malformed Unicode refuse without coercion',
 async () => {
  for (const source of ["function f(){return 'a\nb';}",
 "function f(){return '\\01';}",
 "function f(){return '\\u0041';}",
    "function f(){return '\ud800';}",
 {toString(){throw new Error('caller coercion');
}}]) {
    const result = await run(source);
 assert.equal(result.state,
 'Unsupported');
 assert.equal(result.evaluated,
 false);
  }
  assert.equal((await run("function f(){return '\\0';}")).value,
 '\0');
});

test('unknown profile and caller metadata cannot select arbitrary code',
 async () => {
  assert.equal(Object.isFrozen(TEMPORARY_INTERPRETER_PROFILES),
 true);
  assert.equal((await evaluateTemporaryInterpreter('function f(){return "x";}',
 'foreign')).reason,
 'UnsupportedTemporaryProfile');
  assert.equal((await evaluateTemporaryInterpreter('function f(){return "x";}',
 {profile: 'owned-string-array-v1'})).reason,
 'UnsupportedTemporaryProfile');
});

test("canonical reserved bindings and all JavaScript return line terminators refuse",
async()=>{for(const source of ["function delete(){return 'x';}",
"function f(){const switch=[];return switch.length;}",
... [0x2028,
0x2029].map(code=>"function f(){return"+String.fromCharCode(code)+"'x';}")]){const result=await run(source);
assert.equal(result.state,
"Unsupported");
assert.equal(result.evaluated,
false);
assert.equal(result.ir,
null);
}});

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';

test('typed source spans conserve Unicode bindings and operation order', async () => {
  const source = "// 😀 source\nfunction f(){const text='rain';const a=['雪'];a.push(text);return a.join('\\t');}";
  const result = await run(source);
  assert.equal(result.state, 'Evaluated');
  assert.equal(result.value, vm.runInNewContext(source + ';f()'));
  assert.equal(result.ir.sourceSha256, createHash('sha256').update(Buffer.from(source)).digest('hex'));
  assert.deepEqual(result.ir.operations.map(operation => operation.kind), ['declareString', 'declareArray', 'effect', 'return']);
  const bytes = Buffer.from(source);
  for (const binding of result.ir.bindings) assert.equal(bytes.subarray(binding.span.start, binding.span.end).toString(), binding.name);
  for (const operation of result.ir.operations) {
    assert.ok(operation.span.start >= 0 && operation.span.end <= bytes.length);
    assert.equal(bytes.subarray(operation.span.end - 1, operation.span.end).toString(), ';');
    assert.equal(Object.isFrozen(operation), true);
  }
});

test('string element types and finite profile bounds reject before evaluation', async () => {
  const cases = ["function f(){const a=[1];return a.length;}", "function f(){const a=[];a.push(null);return a.length;}",
    "function f(){const a=[];return a.join();}", "function f(){const a=[];return a.join(1);}",
    "function f(){const a=[];return a.length();}", 'function f(){const a=[' + Array(1025).fill("''").join(',') + '];return a.length;}',
    "function f(){return '" + 'x'.repeat(16385) + "';}", '// ' + 'x'.repeat(65536) + "\nfunction f(){return 'x';}"];
  for (const source of cases) {
    const result = await run(source);
    assert.equal(result.state, 'Unsupported'); assert.equal(result.evaluated, false); assert.equal(result.ir, null);
  }
});

test('fixed lazy module digest refuses isolated source tamper before first evaluation', async () => {
  const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), 'owned-array-source-'));
  const paths = ['scripts/self-translation/ir.mjs', 'scripts/self-translation/owned-string-array.mjs',
    'scripts/self-translation/frontends.mjs', 'scripts/self-translation/lexer.mjs', 'scripts/self-translation/constructs.mjs',
    'scripts/self-translation/lino.mjs', 'data/meta/self-translation/constructs.lino'];
  try {
    for (const name of paths) {
      const target = path.join(root, name);
      fs.mkdirSync(path.dirname(target), {recursive: true});
      fs.copyFileSync(new URL('../../../' + name, import.meta.url), target, fs.constants.COPYFILE_EXCL);
    }
    fs.appendFileSync(path.join(root, 'scripts/self-translation/owned-string-array.mjs'), '\n// isolated tamper\n');
    const fixture = await import(pathToFileURL(path.join(root, 'scripts/self-translation/ir.mjs')));
    const result = await fixture.evaluateTemporaryInterpreter("function f(){return 'x';}", 'owned-string-array-v1');
    assert.equal(result.reason, 'TemporaryModuleSourceDrift'); assert.equal(result.evaluated, false);
    assert.equal(fixture.FRAGMENT, 'portable-pure-v1');
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
