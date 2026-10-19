import assert from 'node:assert/strict';
import {test} from 'node:test';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync,readdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {scanJavaScriptImplementations,trackedSourceFiles} from '../../../scripts/check-source-duplication.mjs';
import {parse} from '@babel/parser';
import traversal from '@babel/traverse';
import {structuralCallables,groupClones,assessDuplication,proveDerivedCallables,canonicalCloneGroups,proveFactoryCallables} from '../../../scripts/lib/source-duplication.mjs';
import {translateJsToTs,tokenize} from '../../../scripts/translate-es.mjs';
import {factoryOf} from '../../../scripts/generate-worker-crate-modules.mjs';
const traverse = traversal.default ?? traversal;
const scan = (source,file='fixture.mjs') => structuralCallables(source,file,{parse,traverse});
function fixtureFiles(root){return ['js','ts'].flatMap(folder=>readdirSync(join(root,folder),{withFileTypes:true}).filter(entry=>entry.isFile()).map(entry=>folder+'/'+entry.name));}
const duplicates = source => groupClones(scan(source));
test('comments, spacing and renamed local bindings do not hide clones',()=> {
 assert.equal(duplicates('function a(x){let y=x+1;return y*2} function b(input){/* differing comment */let next=input+1;return next*2}').length,1);
});
test('methods, arrows, nested functions and same-file clones are scanned',()=> {
 const records=scan('class C{m(x){return x+1}} const a=(v)=>{return v+1}; function outer(){return function z(q){return q+1}}');
 assert.equal(records.length,4);assert.equal(groupClones(records)[0].sites.length,3);
});
test('long functions have no maximum-line escape hatch',()=> {
 const body=Array.from({length:40},(_,i)=>'x += '+i+';').join('\n');
 assert.equal(duplicates('function a(x){'+body+'return x} function b(x){'+body+'return x}').length,1);
});
test('literals, properties, operators, captured bindings and effects remain distinct',()=> {
 for(const [a,b]of [['return x+1','return x+2'],['return x.left','return x.right'],['return x+1','return x-1'],['return x+globalA','return x+globalB'],['return read(x)','return write(x)']])assert.equal(duplicates('function a(x){'+a+'}function b(x){'+b+'}').length,0);
});
test('regex and template interpolation are parsed as syntax',()=> {
 assert.equal(duplicates('function a(x){return /[{}]/.test(x)}function b(v){return /[{}]/.test(v)}').length,1);
 const tick=String.fromCharCode(96), dollar=String.fromCharCode(36);
 assert.equal(duplicates('function a(x){return '+tick+'a'+dollar+'{x+1}'+tick+'}function b(v){return '+tick+'a'+dollar+'{v+2}'+tick+'}').length,0);
});
test('TypeScript annotations cannot hide identical runtime logic',()=> {
 const rows=scan('function a(x:number):number{return x+1}function b(v:string):string{return v+1}','fixture.mts');
 assert.equal(groupClones(rows).length,1);
});
test('syntax errors are refused rather than silently counted as clean',()=> {
 assert.throws(()=>scan('function broken( {'));
});
test('generated twins are candidates, never automatically exempt',()=> {
 const a=scan('function a(x){return x+1}','js/a.mjs').map(row=>({...row,file:'js/a.mjs'}));
 const b=scan('function a(x:number){return x+1}','ts/a.mts').map(row=>({...row,file:'ts/a.mts'}));
 assert.equal(groupClones([...a,...b]).length,1);
});
test('missing Rust structural coverage refuses strict acceptance',()=> {
 const report=assessDuplication({files:['rust/src/a.rs'],parsedFiles:[],clones:[],gaps:[]});
 assert.equal(report.accepted,false);assert.equal(report.gaps.length,1);
});
test('a baseline cannot authorize a clone, even if passed as extra data',()=> {
 const report=assessDuplication({files:['a.mjs'],parsedFiles:['a.mjs'],clones:[{fingerprint:'x'}],gaps:[],baseline:['x']});
 assert.equal(report.accepted,false);
});

test('filesystem guard scans generated and authored same-directory files without a baseline',()=> {
 const root=mkdtempSync(join(tmpdir(),'strict-duplication-'));
 try{mkdirSync(join(root,'js'));mkdirSync(join(root,'ts'));mkdirSync(join(root,'scripts'));mkdirSync(join(root,'js','agentic'));mkdirSync(join(root,'js','agentic','crate'));writeFileSync(join(root,'scripts','translate-es.mjs'),'qualified generator fixture');writeFileSync(join(root,'js','agentic','crate','es_tokenizer.mjs'),'qualified tokenizer fixture');
 writeFileSync(join(root,'js','a.mjs'),'function a(x){return x+1}');
 writeFileSync(join(root,'ts','a.mts'),'// generated\nfunction a(x:number){return x+1}');
 const report=scanJavaScriptImplementations(root,['js','ts'],{listSourceFiles:fixtureFiles,listFactoryMappings:()=>[]});assert.equal(report.accepted,false);assert.equal(report.cloneGroups,1);assert.equal(report.files,2);
 }finally{rmSync(root,{recursive:true,force:true});}
});
test('filesystem guard accepts distinct parsed logic and refuses parse errors and empty discovery',()=> {
 const root=mkdtempSync(join(tmpdir(),'strict-duplication-'));
 try{mkdirSync(join(root,'js'));mkdirSync(join(root,'ts'));mkdirSync(join(root,'scripts'));mkdirSync(join(root,'js','agentic'));mkdirSync(join(root,'js','agentic','crate'));writeFileSync(join(root,'scripts','translate-es.mjs'),'qualified generator fixture');writeFileSync(join(root,'js','agentic','crate','es_tokenizer.mjs'),'qualified tokenizer fixture');
 assert.equal(scanJavaScriptImplementations(root,['js','ts'],{listSourceFiles:fixtureFiles,listFactoryMappings:()=>[]}).accepted,false);
 writeFileSync(join(root,'js','a.mjs'),'function a(x){return x+1}');
 writeFileSync(join(root,'js','b.mjs'),'function b(x){return x+2}');
 assert.equal(scanJavaScriptImplementations(root,['js','ts'],{listSourceFiles:fixtureFiles,listFactoryMappings:()=>[]}).accepted,true);
 writeFileSync(join(root,'js','broken.mjs'),'function broken( {');
 assert.equal(scanJavaScriptImplementations(root,['js','ts'],{listSourceFiles:fixtureFiles,listFactoryMappings:()=>[]}).accepted,false);
 }finally{rmSync(root,{recursive:true,force:true});}
});
test('destructuring property semantics and captured binding identities cannot be alpha-renamed away',()=> {
 assert.equal(duplicates('function a({left}){return left}function b({right}){return right}').length,0);
 assert.equal(duplicates('function a(){return externalA}function b(){return externalB}').length,0);
});

function proofFor(source,target=translateJsToTs(source),sourceRows=scan(source,'js/agentic/a.mjs'),targetRows=scan(target,'ts/agentic/a.mts')) {
 return proveDerivedCallables({sourcePath:'js/agentic/a.mjs',targetPath:'ts/agentic/a.mts',source,target,sourceCallables:sourceRows,targetCallables:targetRows,translate:translateJsToTs,tokenize,generator:[{file:'scripts/translate-es.mjs',sha256:'test-only fixture identity'}]});
}
function pairGroups(source,target=translateJsToTs(source)) {
 return groupClones([...scan(source,'js/agentic/a.mjs').map(row=>({...row,file:'js/agentic/a.mjs'})),...scan(target,'ts/agentic/a.mts').map(row=>({...row,file:'ts/agentic/a.mts'}))]);
}
test('same uniquely mapped canonical meta node has two renderings and one definition',()=> {
 const source='export function quote(x){return x+1}';const proof=proofFor(source);assert.equal(proof.accepted,true);assert.equal(pairGroups(source).length,1);assert.equal(canonicalCloneGroups(pairGroups(source),[proof]).length,0);assert.equal(proof.nodes.length,1);assert.ok(proof.nodes[0].canonicalNode.metaTreeSha256);
});
test('forged generated headers and stale target bytes cannot authorize a shared definition',()=> {
 const source='function a(x){return x+1}';
 for(const target of ['// generated\n'+source,source.replace('+1','+2')])assert.equal(proofFor(source,target).accepted,false);
});
test('independent repeated source bodies remain violations after exact twin regeneration',()=> {
 const source='function a(x){return x+1}function b(y){return y+1}',proof=proofFor(source);assert.equal(proof.accepted,true);const violations=canonicalCloneGroups(pairGroups(source),[proof]);assert.equal(violations.length,1);assert.equal(violations[0].independentDefinitionCount,2);assert.equal(violations[0].sites.length,4);
});
test('duplicated target-only callables fail generation and remain visible',()=> {
 const source='function a(x){return x+1}',target=source+'function added(y){return y+1}',proof=proofFor(source,target);assert.equal(proof.accepted,false);assert.equal(canonicalCloneGroups(pairGroups(source,target),[proof])[0].independentDefinitionCount,3);
});
test('ambiguous source or target span mappings are refused',()=> {
 const source='function a(x){return x+1}',rows=scan(source,'js/agentic/a.mjs');assert.equal(proofFor(source,source,[...rows,rows[0]],rows).accepted,false);assert.equal(proofFor(source,source,rows,[...rows,rows[0]]).accepted,false);
});
test('source inventory requires cached plus nonignored new paths using NUL-safe Git output',()=> {
 let observed;const files=trackedSourceFiles('/fixture',['js','ts'],(command,args,options)=>{observed={command,args,options};return 'js/tracked-generated.js\0js/new source.mjs\0ts/twin.ts\0js/.gitignore\0';});assert.deepEqual(files,['js/new source.mjs','js/tracked-generated.js','ts/twin.ts']);assert.equal(observed.command,'git');for(const flag of ['--cached','--others','--exclude-standard','-z'])assert.ok(observed.args.includes(flag));
});

test('serialized or forged successful proof metadata is not derivation authority',()=> {
 const source='function a(x){return x+1}',proof=proofFor(source),forged=JSON.parse(JSON.stringify(proof));
 assert.equal(canonicalCloneGroups(pairGroups(source),[forged]).length,1);
 assert.throws(()=>{proof.nodes[0].canonicalId='forged';},TypeError);
});

function factoryProof(source,target=factoryOf('crate/probe.mjs',source),rows=scan(target,'js/worker/probe.js')) {
 return proveFactoryCallables({sourcePath:'js/agentic/a.mjs',targetPath:'js/worker/probe.js',source,target,sourceCallables:scan(source,'js/agentic/a.mjs'),targetCallables:rows,regenerate:text=>factoryOf('crate/probe.mjs',text),tokenize,generator:[{file:'factoryOf',sha256:'fixture'}]});
}
function factoryGroups(source,target=factoryOf('crate/probe.mjs',source)) {
 return groupClones([...scan(source,'js/agentic/a.mjs').map(row=>({...row,file:'js/agentic/a.mjs'})),...scan(target,'js/worker/probe.js').map(row=>({...row,file:'js/worker/probe.js'}))]);
}
test('exact independently regenerated factory callables share canonical source identity',()=> {
 const source='export function a(x){return x+1}\n',proof=factoryProof(source);assert.equal(proof.accepted,true);assert.equal(proof.nodes.length,1);assert.equal(canonicalCloneGroups(factoryGroups(source),[proof]).length,0);
});
test('factory forged headers, stale bytes and added duplicate callables remain failures',()=> {
 const source='export function a(x){return x+1}\n',target=factoryOf('crate/probe.mjs',source);
 for(const stale of ['// generated\n'+target,target.replace('x+1','x+2'),target.replace('return Object.freeze','function copied(x){return x+1}\nreturn Object.freeze')])assert.equal(factoryProof(source,stale).accepted,false);
});
test('factory source duplicates remain two definitions and ambiguous target spans refuse',()=> {
 const source='export function a(x){return x+1}\nexport function b(y){return y+1}\n',proof=factoryProof(source);assert.equal(proof.accepted,true);assert.equal(canonicalCloneGroups(factoryGroups(source),[proof])[0].independentDefinitionCount,2);
 const target=factoryOf('crate/probe.mjs',source),rows=scan(target,'js/worker/probe.js');assert.equal(factoryProof(source,target,[...rows,rows[1]]).accepted,false);
});
test('source factory and TypeScript paths form one transitive proven definition',()=> {
 const source='export function a(x){return x+1}\n',target=factoryOf('crate/probe.mjs',source),workerProof=factoryProof(source),tsProof=proveDerivedCallables({sourcePath:'js/worker/probe.js',targetPath:'ts/worker/probe.ts',source:target,target:translateJsToTs(target),sourceCallables:scan(target,'js/worker/probe.js'),targetCallables:scan(target,'ts/worker/probe.ts'),translate:translateJsToTs,tokenize,generator:[]});
 const raw=groupClones([...factoryGroups(source)[0].sites,...scan(target,'ts/worker/probe.ts').filter(row=>row.syntaxPath!=='body[0].expression.right').map(row=>({...row,file:'ts/worker/probe.ts'}))]);assert.equal(canonicalCloneGroups(raw,[workerProof,tsProof]).length,0);assert.equal(canonicalCloneGroups(raw,[tsProof,workerProof]).length,0);
});
