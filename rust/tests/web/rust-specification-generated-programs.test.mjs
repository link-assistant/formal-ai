import {

readFileSync}

 from 'node:fs';

import {

selfTranslate}

 from '../../../scripts/self-translation/envelope.mjs';

import {

lex,

topLevelItems}

 from '../../../scripts/self-translation/lexer.mjs';

import {

TEMPORARY_WORKAROUND}

 from '../../../scripts/lib/generate-native-specification-programs.mjs';

import test from 'node:test';

import assert from 'node:assert/strict';

import {

execFileSync}

 from 'node:child_process';

import {

resolve}

 from 'node:path';

import {

generate}

 from '../../../scripts/lib/generate-native-specification-programs.mjs';

import {

nativeStringContract}

 from '../../../scripts/lib/native-string-contract.mjs';

import {

specificationCases,

tokenize,

testFunctions}

 from '../../../scripts/lib/rust-specification-cases.mjs';

import {

typedProgramOf,

executeTypedProgram}

 from '../../../scripts/lib/rust-specification-programs.mjs';

import {

evaluate}

 from '../../../scripts/lib/rust-specification-values.mjs';

import {

WorkerHost}

 from '../../../js/server/worker-host.mjs';

const root=resolve(import.meta.dirname,

'../../..');

function producer(name='substitute',

input='text',

selector='language',

table='pairs',

output='changed',

from='before',

to='after') {

 return `pub fn ${name}(${input}:&str,${selector}:&str)->String { let ${table}:&[(&str,&str)] = match ${selector} { "cascade"=>&[("a","b"),("b","c")], "literal"=>&[("a","$&$${String.fromCharCode(96)}")], _=>&[("a","x")] }; let mut ${output}=${input}.to_string(); for (${from},${to}) in ${table} { ${output}=${output}.replace(${from},${to}); } ${output} }`;

}

const compile=source=>generate(source,

'rust/src/arbitrary/mod.rs',

'arbitrary',

'pub mod arbitrary;');

function inline(source) {

return typedProgramOf(testFunctions(tokenize(source))[0].body,

{

source,

root}

);

}

test('generated native registry is current',

()=> {

 assert.match(execFileSync(process.execPath,

[root+'/scripts/lib/generate-native-specification-programs.mjs',

'--check'],

{

encoding:'utf8'}

),

/GENERATION_CHECK_PASS/);

}

);

test('generic substitution AST preserves binding identity and replacement order',

async()=> {

 const first=compile(producer()).programs[0];

 const renamed=compile(producer('unrelated',

'material',

'variant',

'rewrites',

'result',

'needle',

'inserted')).programs[0];

 assert.deepEqual(first.program,

renamed.program);

 for(const [input,

selector,

expected] of [['aaa',

'cascade',

'ccc'],

['aaa',

'literal',

'$&$`$&$`$&$`'],

['aaa',

'other',

'xxx'],

['😀a😀',

'cascade',

'😀c😀']]) {

  const actual=await evaluate({

kind:'call',

binding:first,

args:[{

kind:'literal',

value:input}

,

{

kind:'literal',

value:selector}

]}

,

new Map(),

{

observations:[]}

);

  assert.equal(actual,

expected);

 }

 await assert.rejects(()=>evaluate({

kind:'call',

binding:first,

args:[{

kind:'literal',

value:'\ud800'}

,

{

kind:'literal',

value:'cascade'}

]}

,

new Map(),

{

observations:[]}

));

 for(const changed of [producer().replace('("a","b")',

'("","b")'),

producer().replace('let pairs',

'let mut pairs'),

producer().replace('.replace(before,after)',

'.unknown(before,after)'),

producer().replace('} changed }',

'effect(); } changed }')])assert.equal(compile(changed).programs.length,

0);

 for(const prefix of ['struct String;',

'type String=Custom;',

'trait T {}',

'use unknown::String;'])assert.throws(()=>compile(prefix+producer()));

}

);

test('native literal contracts refuse unknown decoding and source shadows',

()=> {

 for(const literal of ['"\\q"',

'"\\x41"',

'"\\u{a_b}"',

'"\\u{d800}"',

'"\\u{110000}"',

'r"raw"',

'r#"raw"#',

'r##"raw"##',

'br#"bytes"#',

'"\\\ncontinued"'])assert.throws(()=>nativeStringContract(literal));

 const imported='use formal_ai::summarization::apply_compound_words as reduce;';

 for(const body of ['assert_eq!(reduce("\\q","en"),"x");',

'let reduce="shadow"; assert_eq!(reduce("x","en"),"x");',

'assert_eq!(reduce("x","en").unknown(),"x");',

'assert_eq!(reduce("x",true),"x");',

'assert_eq!(reduce("x","en"),"x"); effect();'])assert.equal(inline(imported+' #[test] fn unrelated(){'+body+'}').program,

undefined);

 const scoped=inline(imported+' #[test] fn unrelated(){assert_eq!(reduce("in order to work","en"),"to work");} #[test] fn other(){assert_eq!("\\x41","A");}');

 assert.ok(scoped.program,

scoped.reason);

}

);

test('source compound and semantic assertions execute unchanged',

async()=> {

 const identifier='rust/tests/unit/specification/summarization_pipeline.rs::compound_words_and_semantic_primes_are_reversible_by_size';

 const [path,
name]=identifier.split('::');

 const source=readFileSync(root+'/'+path,
'utf8');

 const body=testFunctions(tokenize(source)).find(item=>item.name===name).body;

 const original=typedProgramOf(body,
{
source,
file:root+'/'+path,
root}
);

 assert.ok(original?.program);

 assert.equal(original.program.nativeAssertions,

2);

 const actual=await executeTypedProgram(new WorkerHost(),

original.program);

 assert.equal(actual.status,

'passed',

actual.failure);

 const stale=structuredClone(original.program);

 stale.fixtures[0].sha256='0'.repeat(64);

 const refused=await executeTypedProgram(new WorkerHost(),
stale);

 assert.equal(refused.status,
'failed');

 assert.equal(refused.observations.length,
0);

 assert.equal(actual.assertions,

2);

}

);

test('temporary compiler is tied to actual maintained translation blockers',

()=> {

 const source=readFileSync(root+'/rust/src/summarization/mod.rs',

'utf8');

 const generated=generate(source);

 assert.equal(generated.programs.length,

2);

 for(const binding of generated.programs) {

  const name=binding.path.split('::').at(-1);

  const item=topLevelItems(lex(source,

'Rust')).find(item=>item.tokens.some((t,

i)=>t.text==='fn'&&item.tokens[i+1]?.text===name));

  assert.ok(item);

  const translated=selfTranslate(source.slice(item.start,

item.end),

'Rust',

'JavaScript');

  assert.ok(translated.items.some(item=>item.status==='carried'&&item.reason===TEMPORARY_WORKAROUND.maintainedTranslatorDiagnostic));

  assert.ok(binding.sourceWitnesses.every(witness=>typeof witness.sha256==='string'&&witness.sha256.length===64));

 }

 assert.equal(TEMPORARY_WORKAROUND.upstreamIssue,

null);

}

);
