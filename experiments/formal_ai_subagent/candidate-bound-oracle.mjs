// Candidate consumption is structural evidence, never a semantic usefulness verdict.
import {
parse}
 from '@babel/parser';
import {
readFileSync,
lstatSync,
realpathSync}
 from 'node:fs';
import {
createHash}
 from 'node:crypto';
import {
resolve,
dirname,
relative,
isAbsolute}
 from 'node:path';
import {
spawnSync}
 from 'node:child_process';
import {
oracleProcessEvidence}
 from './oracle-source-review.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const assertionMethods=new Set(['equal',
'strictEqual',
'deepEqual',
'deepStrictEqual',
'notEqual',
'notStrictEqual',
'ok']);
const operators=new Set(['+',
'-',
'*',
'/',
'%',
'**',
'===',
'!==',
'<',
'<=',
'>',
'>=',
'&',
'|',
'^',
'<<',
'>>',
'>>>']);
function physical(binding){
 if(!binding||
typeof binding.path!=='string'||
!isAbsolute(binding.path)||
!/^([a-f0-9]{64})$/u.test(binding.sha256))throw Error('invalid physical binding');
 if(!lstatSync(binding.path).isFile()||
realpathSync(binding.path)!==binding.path)throw Error('noncanonical regular source required');
 const bytes=readFileSync(binding.path);
if(bytes.length>1048576||
sha(bytes)!==binding.sha256)throw Error('source bytes drift or bound');
return bytes.toString('utf8');
}
export function reviewCandidateBoundOracle(contract){
 const unknown={
protocol:'candidate-bound-original-oracle-v1',
complete:false,
semanticUsefulness:'Unknown',
validatedUsefulNetCodeBytes:null,
providerUsage:'Unknown',
cost:'Unknown',
qualityRequiresReview:true}
;
 try{
  if(!contract||
contract.transport!=='pure-esm'||
!Array.isArray(contract.dependencies)||
contract.dependencies.length>32)throw Error('unsupported transport');
  const declaration=JSON.parse(physical(contract.declaration));
  if(declaration.version!==1||
declaration.transport!=='pure-esm'||
JSON.stringify(declaration.oracle)!==JSON.stringify(contract.oracle)||
JSON.stringify(declaration.candidate)!==JSON.stringify(contract.candidate)||
JSON.stringify(declaration.dependencies)!==JSON.stringify(contract.dependencies)||
!Array.isArray(declaration.interface)||
!declaration.interface.length)throw Error('frozen declaration mismatch');
  const rows=[contract.candidate,
...contract.dependencies],
rowByPath=new Map(rows.map(row=>[row.path,
row]));
  if(rowByPath.size!==rows.length||
rowByPath.has(contract.oracle.path))throw Error('duplicate source identity');
  const parsed=new Map(),
exportsByPath=new Map(),
usedFiles=new Set();
let nodes=0;
  function parseSource(row){
const source=physical(row),
ast=parse(source,
{
sourceType:'module'}
);
parsed.set(row.path,
ast);
return ast;
}
  for(const row of rows){
const ast=parseSource(row),
names=new Map();
   for(const item of ast.program.body){
if(item.type!=='ExportNamedDeclaration'||
item.source||
item.specifiers.length||
item.declaration?.type!=='FunctionDeclaration')throw Error('candidate profile requires named pure function exports');
const fn=item.declaration;
if(!fn.id||
names.has(fn.id.name)||
fn.async||
fn.generator||
fn.params.some(p=>p.type!=='Identifier')||
fn.body.body.length!==1||
fn.body.body[0].type!=='ReturnStatement')throw Error('unknown candidate function');
names.set(fn.id.name,
fn);
}
   if(!names.size)throw Error('candidate has no source-owned interface');
exportsByPath.set(row.path,
names);
   for(const fn of names.values()){
const args=new Set(fn.params.map(p=>p.name));
if(args.size!==fn.params.length)throw Error('duplicate candidate parameter');
pure(fn.body.body[0].argument,
args);
}
  }
  function pure(node,
args){
if(!node||
++nodes>65536)throw Error('source node bound');
if(['StringLiteral',
'NumericLiteral',
'BooleanLiteral',
'NullLiteral'].includes(node.type))return;
if(node.type==='Identifier'&&
args.has(node.name))return;
if(node.type==='BinaryExpression'&&
operators.has(node.operator)){
pure(node.left,
args);
pure(node.right,
args);
return;
}
if(node.type==='UnaryExpression'&&
['+',
'-',
'!',
'~'].includes(node.operator)){
pure(node.argument,
args);
return;
}
throw Error('unsupported pure candidate expression');
}
  const declared=declaration.interface;
if(new Set(declared).size!==declared.length||
declared.some(n=>typeof n!=='string')||
JSON.stringify([...exportsByPath.get(contract.candidate.path).keys()].sort())!==JSON.stringify([...declared].sort()))throw Error('candidate interface mismatch');
  // This initial closed profile has no executable imported dependencies. Extra supplied rows cannot confer authority.
  if(contract.dependencies.length)throw Error('unused or unsupported dependency closure');
  const oracleSource=physical(contract.oracle),
ast=parse(oracleSource,
{
sourceType:'module'}
),
env=new Map(),
consumed=new Set(),
obligations=[],
tests=new Set();
  function bind(name,
value){
if(env.has(name)||
['process',
'globalThis',
'console'].includes(name))throw Error('shadowed oracle binding');
env.set(name,
value);
}
  function expression(node,
local){
if(!node||
++nodes>65536)throw Error('oracle node bound');
if(['StringLiteral',
'NumericLiteral',
'BooleanLiteral',
'NullLiteral'].includes(node.type))return new Set();
if(node.type==='Identifier'&&
local.has(node.name))return new Set(local.get(node.name));
if(node.type==='UnaryExpression'&&
['+',
'-',
'!',
'~'].includes(node.operator))return expression(node.argument,
local);
if(node.type==='ArrayExpression'){
const out=new Set();
for(const e of node.elements)for(const name of expression(e,
local))out.add(name);
return out;
}
if(node.type==='BinaryExpression'&&
operators.has(node.operator)){
return new Set([...expression(node.left,
local),
...expression(node.right,
local)]);
}
if(node.type==='CallExpression'&&
!node.optional&&
node.callee.type==='Identifier'&&
env.get(node.callee.name)?.kind==='candidate'){
const owner=env.get(node.callee.name);
if(node.arguments.length!==owner.arity)throw Error('candidate call arity mismatch');
const out=new Set([owner.export]);
for(const arg of node.arguments)for(const name of expression(arg,
local))out.add(name);
return out;
}
throw Error('unknown candidate-consuming expression');
}
  function statements(body,
local){
let count=0;
for(const node of body){
if(node.type==='VariableDeclaration'&&
node.kind==='const'){
for(const d of node.declarations){
if(d.id.type!=='Identifier'||
env.has(d.id.name)||
local.has(d.id.name))throw Error('unknown or shadowed oracle value');
local.set(d.id.name,
expression(d.init,
local));
}
continue;
}
if(node.type!=='ExpressionStatement'||
node.expression.type!=='CallExpression'||
node.expression.optional)throw Error('unsupported oracle control flow');
const call=node.expression;
if(call.callee.type!=='MemberExpression'||
call.callee.computed||
call.callee.object.type!=='Identifier'||
env.get(call.callee.object.name)?.kind!=='assert'||
!assertionMethods.has(call.callee.property.name)||
call.arguments.length<1)throw Error('unknown oracle assertion');
const assertedOperands=call.callee.property.name==='ok'?1:2;
if(call.arguments.length<assertedOperands||
call.arguments.length>assertedOperands+1)throw Error('assertion operand arity mismatch');
const seen=new Set();
for(let index=0;
index<call.arguments.length;
index++){
const names=expression(call.arguments[index],
local);
if(index<assertedOperands)for(const name of names)seen.add(name);
}
if(!seen.size)throw Error('assertion does not consume candidate');
for(const n of seen)consumed.add(n);
obligations.push({
start:call.start,
end:call.end,
exports:[...seen].sort()}
);
count++;
}
return count;
}
  for(const node of ast.program.body){
if(node.type==='ImportDeclaration'){
if(node.assertions?.length||
node.attributes?.length||
!node.specifiers.length)throw Error('unknown import attributes');
const module=node.source.value;
if(['node:assert/strict',
'node:assert'].includes(module)){
for(const s of node.specifiers){
if(s.type!=='ImportDefaultSpecifier'&&
s.type!=='ImportNamespaceSpecifier')throw Error('unsupported assert binding');
bind(s.local.name,
{
kind:'assert'}
);
}
}
else if(module==='node:test'){
for(const s of node.specifiers){
if(s.type!=='ImportDefaultSpecifier'&&
!(s.type==='ImportSpecifier'&&
s.imported.name==='test'))throw Error('unknown test binding');
bind(s.local.name,
{
kind:'test'}
);
}
}
else{
if(!module.startsWith('./')&&
!module.startsWith('../'))throw Error('nonrelative candidate import');
const resolved=resolve(dirname(contract.oracle.path),
module);
if(resolved!==contract.candidate.path)throw Error('oracle does not import declared candidate');
usedFiles.add(resolved);
for(const s of node.specifiers){
const fn=exportsByPath.get(resolved).get(s.imported?.name);
if(s.type!=='ImportSpecifier'||
!fn)throw Error('candidate import interface mismatch');
bind(s.local.name,
{
kind:'candidate',
export:s.imported.name,
arity:fn.params.length}
);
}
}
continue;
}
   if(node.type!=='ExpressionStatement'||
node.expression.type!=='CallExpression'||
node.expression.optional)throw Error('unsupported top-level oracle');
const call=node.expression;
if(call.callee.type!=='Identifier'||
env.get(call.callee.name)?.kind!=='test'||
call.arguments.length!==2||
call.arguments[0].type!=='StringLiteral'||
!call.arguments[0].value||
tests.has(call.arguments[0].value))throw Error('unknown or skipped test');
const cb=call.arguments[1];
if(!['ArrowFunctionExpression',
'FunctionExpression'].includes(cb.type)||
cb.async||
cb.generator||
cb.params.length||
cb.body.type!=='BlockStatement')throw Error('unknown test callback');
if(!statements(cb.body.body,
new Map()))throw Error('zero candidate assertions');
tests.add(call.arguments[0].value);
  }
  if(usedFiles.size!==1||
!tests.size||
consumed.size!==declared.length||
declared.some(name=>!consumed.has(name)))throw Error('candidate interface not fully consumed');
  return {
...unknown,
complete:true,
sourceBindings:[contract.declaration,
contract.oracle,
...rows],
expectedProcessTests:tests.size,
assertionObligations:obligations,
consumedExports:[...consumed].sort(),
scope:'closed candidate consumption and interface only; usefulness not established'}
;
 }
catch(error){
return {
...unknown,
reason:String(error.message)}
;
}
}
export function executeCandidateBoundOracle(contract){
 const review=reviewCandidateBoundOracle(contract);
if(!review.complete)return {
review,
structuralComplete:false,
semanticAcceptance:'Unknown'}
;
 const env={
...process.env}
;
delete env.NODE_OPTIONS;
delete env.NODE_PATH;
delete env.NODE_TEST_CONTEXT;
 const processResult=spawnSync(process.execPath,
['--test',
'--test-reporter=tap',
contract.oracle.path],
{
cwd:dirname(contract.oracle.path),
env,
encoding:'utf8',
timeout:10000,
maxBuffer:1048576}
);
 const after=reviewCandidateBoundOracle(contract),
completion=oracleProcessEvidence(processResult,
review.expectedProcessTests);
 return {
review,
after,
completion,
process:{
status:processResult.status,
signal:processResult.signal,
stdout:processResult.stdout,
stderr:processResult.stderr,
error:processResult.error?.message??null}
,
structuralComplete:after.complete&&
completion.complete,
semanticAcceptance:'Unknown',
validatedUsefulNetCodeBytes:null,
providerUsage:'Unknown',
cost:'Unknown'}
;
}
