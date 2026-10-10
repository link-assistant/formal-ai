import {resolve} from 'node:path';
import {generateScalars,generateEnumPredicates} from './native-scalar-registry.mjs';
import {

nativeStringContract}

 from './native-string-contract.mjs';

import fs from 'node:fs';

import{

createHash}

from 'node:crypto';

import{

lex,

topLevelItems}

from '../self-translation/lexer.mjs';

import{

selfTranslate}

from '../self-translation/envelope.mjs';

import{

observeCallableModule}

from '../self-translation/source-callable-catalog.mjs';

import{

tokenize}

from './rust-specification-cases.mjs';

import{

nativeFunctions}

from './rust-specification-bindings.mjs';

import{

close,

split}

from './rust-specification-values.mjs';

const root=resolve(import.meta.dirname,'../..');

const directory=import.meta.dirname;

const hash=value=>createHash('sha256').update(value).digest('hex');

export function substitutionProgram(declaration){

  if(!declaration.public||declaration.returns.map(t=>t.text).join('')!=='->String')throw Error('unproved public string return');

  const parameters=split(declaration.parameters);

if(parameters.length!==2||parameters.some(p=>p[0]?.kind!=='word'||p.slice(1).map(t=>t.text).join('')!==':&str'))throw Error('unproved substitution signature');

  const [input,

selector]=parameters.map(p=>p[0].text);

if(input===selector)throw Error('duplicate parameter');

const b=declaration.body,

equals=b.findIndex(t=>t.text==='=');

  if(b[0]?.text!=='let'||b[1]?.kind!=='word'||b.slice(2,

equals).map(t=>t.text).join('')!==':&[(&str,&str)]'||b[equals+1]?.text!=='match'||b[equals+2]?.text!==selector||b[equals+3]?.text!=='{')throw Error('unproved immutable substitution table');

  const table=b[1].text,

open=equals+3,

end=close(b,

open);

const arms=split(b.slice(open+1,

end));

if(arms.length<2)throw Error('substitution fallback absent');

const alternatives=[];

const selectors=new Set();

  for(const [index,

arm]of arms.entries()){

    const fallback=arm[0]?.text==='_';

if((!fallback&&arm[0]?.kind!=='string')||arm[1]?.text!=='='||arm[2]?.text!=='>'||arm[3]?.text!=='&'||arm[4]?.text!=='['||close(arm,

4)!==arm.length-1||(fallback&&index!==arms.length-1))throw Error('unproved substitution match arm');

    const pairs=split(arm.slice(5,

-1)).map(part=>{

if(part[0]?.text!=='('||close(part,

0)!==part.length-1)throw Error('unproved substitution tuple');

const values=split(part.slice(1,

-1));

if(values.length!==2||values.some(v=>v.length!==1||v[0].kind!=='string')||values[0][0].text==='')throw Error('unknown or empty replacement');

return values.map(v=>v[0].text);

}

);

    if(!fallback){

if(selectors.has(arm[0].text))throw Error('duplicate substitution selector');

selectors.add(arm[0].text);

}

alternatives.push({

...(fallback?{

fallback:true}

:{

selector:arm[0].text}

),

pairs}

);

  }

  if(!alternatives.at(-1).fallback)throw Error('substitution fallback absent');

const tail=b.slice(end+1);

const out=tail[3]?.text,

from=tail[13]?.text,

to=tail[15]?.text;

  if(new Set([input,

selector,

table,

out,

from,

to]).size!==6)throw Error('ambiguous substitution bindings');

  const expected=[';',

'let',

'mut',

out,

'=',

input,

'.',

'to_string',

'(',

')',

';',

'for',

'(',

from,

',',

to,

')',

'in',

table,

'{',

out,

'=',

out,

'.',

'replace',

'(',

from,

',',

to,

')',

';',

'}',

out];

  if(tail.length!==expected.length||tail.some((t,

i)=>t.text!==expected[i]))throw Error('unproved replacement loop or effects');

  return {

kind:'substitutionProgram',

signature:['string',

'string'],

returnType:'string',

program:{

selectorArgument:1,

inputArgument:0,

alternatives}

}

;

}

export function generate(source,

path='rust/src/summarization/mod.rs',

namespace='summarization',

publicRoot=fs.readFileSync(root+'/rust/src/lib.rs',

'utf8')){

  nativeStringContract(source);

  const environment=lex(source,

'Rust').filter(t=>t.type!=='comment');

  if(environment.some((t,

i)=>t.text==='trait'||(['struct',

'enum',

'type'].includes(t.text)&&environment[i+1]?.text==='String')))throw Error('unknown native String or trait environment');

  if(environment.some(t=>t.text==='use')&&source!==fs.readFileSync(root+'/rust/src/summarization/mod.rs',

'utf8'))throw Error('unqualified native import environment');

  const rootTokens=tokenize(publicRoot);

if(!rootTokens.some((t,

i)=>t.text==='pub'&&rootTokens[i+1]?.text==='mod'&&rootTokens[i+2]?.text===namespace&&rootTokens[i+3]?.text===';'))throw Error('native module is not public');

  const declarations=nativeFunctions(tokenize(source));

const programs=[],

refusals=[];

  for(const declaration of declarations){

try{

const binding=substitutionProgram(declaration);

programs.push({

path:'formal_ai::'+namespace+'::'+declaration.name,

...binding,

sourceWitnesses:[{

path,

sha256:hash(source)}

,

{

path:'rust/src/lib.rs',

sha256:hash(publicRoot)}

]}

);

}

catch(error){

refusals.push({

name:declaration.name,

reason:error.message}

);

}

}

  return {

programs,

refusals}

;

}

function rendering(programs){

return '// Generated source-qualified substitution programs. Unknown algorithms remain refused.\nexport const GENERATED_NATIVE_PROGRAMS = '+JSON.stringify(programs,

null,

2)+';\n';

}

if(process.argv[1]===new URL(import.meta.url).pathname){

  const path='rust/src/summarization/mod.rs',

source=fs.readFileSync(root+'/'+path,

'utf8');

const generated=generate(source);
const scalars=generateScalars(source);
const predicates=generateEnumPredicates(root);
const combinedPrograms=[...generated.programs,...scalars.programs,...predicates.programs];

const output=directory+'/generated-native-programs.mjs';

const code=rendering(combinedPrograms);

  const actualItems=topLevelItems(lex(source,

'Rust'));

const translation=[];

  for(const binding of combinedPrograms){

const name=binding.path.split('::').at(-1);

const item=binding.sourceSpan??actualItems.find(item=>item.tokens.some((t,

i)=>t.text==='fn'&&item.tokens[i+1]?.text===name));

if(!item)throw Error('actual source span absent');

const producer=binding.sourceWitnesses?.[0]?.path;
const actualSource=producer?fs.readFileSync(root+'/'+producer,'utf8'):source;
const actual=actualSource.slice(item.start,

item.end);

translation.push({

path:binding.path,

sourceSha256:hash(actual),

source:actual,

maintainedTranslation:selfTranslate(actual,

'Rust',

'JavaScript')}

);

}

  const observed=observeCallableModule(fs.readFileSync(root+'/js/agentic/crate/summarization.mjs',

'utf8'),

new URL('file://'+root+'/js/agentic/crate/summarization.mjs').href);

  if(process.argv.includes('--check')){

if(fs.readFileSync(output,

'utf8')!==code)throw Error('generated registry is stale');

console.log('GENERATION_CHECK_PASS');

}

  else{

fs.writeFileSync(output,

code);

fs.writeFileSync(directory+'/generation-observation.json',

JSON.stringify({

path,

sourceSha256:hash(source),

...generated,
scalarPrograms:scalars,

translation,

sourceCallableCatalog:observed,

temporaryWorkaround:{

scope:'Source AST substitution-table loop compiler until maintained translator supports this exact Rust shape',

retirement:'Remove when maintained selfTranslate(actualRustSource, Rust, JavaScript) emits executable twins for every currently carried shape',

upstreamIssue:null,

reason:'No upstream issue was created or invented; diagnostics are recorded below.'}

}

,

null,

2));

console.log('REGISTRY_GENERATED');

}

}

export const TEMPORARY_WORKAROUND = {

  "nativeShape": "public (&str,&str)->String; immutable literal &[(&str,&str)] match table; String::to_string; ordered replace loop",

  "maintainedTranslatorDiagnostic": "a parameter or result without a JSDoc number, boolean or string type: &[",

  "sourceQualification": "Registry records full producer/module SHAs; --check deterministically regenerates actual source AST",

  "retirement": "Remove this compiler when maintained selfTranslate emits executable twins for every currently carried exact shape",

  "upstreamIssue": null,

  "upstreamIssueStatus": "No issue created; diagnostic is the recorded blocker"
}

;
