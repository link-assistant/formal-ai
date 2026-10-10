import fs from 'node:fs';
import path from 'node:path';
import {
  createHash}
from 'node:crypto';
import {
  pathToFileURL}
from 'node:url';
import {tokenize as tokenizeRuntime} from '../../scripts/translate-es.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const ownerPath=owner=>typeof owner==='string'&&/^[A-Za-z0-9_./-]+$/u.test(owner)&&!path.isAbsolute(owner)&&!owner.split('/').some(x=>x==='..'||x==='')&&/\.(mjs|lino|rs)$/u.test(owner)&&/^(js|data|rust\/(src|tests\/web)|experiments\/(js_dogfood|formal_ai_subagent))\//u.test(owner);
function bounded(file){
  const stat=fs.lstatSync(file);
  if(!stat.isFile()||stat.isSymbolicLink()||stat.size>4*1024*1024)throw Error('UnownedBoundedPacket');
  const bytes=fs.readFileSync(file);
  if(bytes.length!==stat.size)throw Error('PacketReadDrift');
  return bytes;
}
function physical(root,
owner){
  const file=path.join(root,
  owner);
  let parent=path.dirname(file);
  while(!fs.existsSync(parent))parent=path.dirname(parent);
  if(!fs.lstatSync(parent).isDirectory()||fs.realpathSync(parent)!==parent)throw Error('UnownedSourceParent');
  if(!fs.existsSync(file))return null;
  if(fs.realpathSync(file)!==file)throw Error('UnownedRootSource');
  return bounded(file).toString('utf8');
}
export function prepareSourcePackets(root,
packets){
  root=fs.realpathSync(root);
  const rows=new Map(),
  bindings=[];
  for(const packet of packets){
    const full=path.resolve(packet),
    bytes=bounded(full),
    manifest=JSON.parse(bytes);
    bindings.push({
      path:full,
      sha256:hash(bytes)}
    );
    if(!Array.isArray(manifest.sources))throw Error('MissingSourceRows');
    for(const row of manifest.sources){
      if(!ownerPath(row.owner)||typeof row.after!=='string'||!(row.before===null||typeof row.before==='string'))throw Error('UnboundSourceRow');
      const beforeHash=row.beforeSHA256??row.beforeSha256??null,
      afterHash=row.afterSHA256??row.afterSha256;
      if((row.before===null?null:hash(row.before))!==beforeHash||hash(row.after)!==afterHash)throw Error('PacketSourceIdentityMismatch');
      if(row.candidate&&hash(bounded(row.candidate))!==afterHash)throw Error('PhysicalCandidateDrift');
      const existing=rows.get(row.owner);
      if(existing&&(existing.afterSHA256!==afterHash||existing.beforeSHA256!==beforeHash))throw Error('ConflictingSourceOwners');
      const current=physical(root,
      row.owner),
      currentHash=current===null?null:hash(current);
      let status=currentHash===afterHash?'already-current':currentHash===beforeHash?'ready':'requires-source-owned-rebase';
      rows.set(row.owner,
      {
        owner:row.owner,
        before:row.before,
        after:row.after,
        beforeSHA256:beforeHash,
        afterSHA256:afterHash,
        currentRootSHA256:currentHash,
        status,
        packet:full}
      );
    }
  }
  return {
    schemaVersion:1,
    root,
    packets:bindings,
    sources:[...rows.values()].sort((a,
    b)=>a.owner.localeCompare(b.owner)),
    ready:[...rows.values()].every(row=>row.status!=='requires-source-owned-rebase'),
    suppliedPatchAutonomousCredit:0,
    usageTokens:null,
    currencyCost:null,
    qualification:'frozen packet integrity and physical preimages only; no authoring or operation authority'}
  ;
}

// Sparse closure evidence never grants module execution or source-write authority.
const runtimeMember = value => typeof value === 'string'
  && /^[A-Za-z0-9_.\/-]+$/u.test(value) && !path.isAbsolute(value)
  && !value.split('/').some(part => part === '..' || part === '');
const verifiedRuntimeClosures = new WeakSet();
const pureBuiltins = new Set(['node:assert/strict', 'node:crypto', 'node:path', 'node:url']);
function literalSpecifier(tree) {
  if (tree?.$ !== 'leaf' || tree.kind !== 'string'
      || !/^(['"])[A-Za-z0-9_.:/-]+\1$/u.test(tree.text)) {
    throw Error('UnsupportedRuntimeSpecifier');
  }
  return tree.text.slice(1, -1);
}
function runtimeEdges(source) {
  const edges = [];
  function visit(trees) {
    for (let i = 0; i < trees.length; i += 1) {
      const tree = trees[i], next = trees[i + 1];
      if (tree.$ === 'template') {
        for (const part of tree.parts) if (part.$ === 'interp') visit(part.trees);
        continue;
      }
      if (tree.$ === 'group') { visit(tree.trees); continue; }
      if (tree.kind !== 'identifier') continue;
      if (['require', 'eval', 'Function', 'fetch', 'importScripts',
          'process', 'globalThis', 'self'].includes(tree.text)) {
        throw Error('UnsupportedRuntimeResourceAccess');
      }
      if (tree.text === 'import' && next?.text !== '.') {
        if (next?.$ === 'group' && next.delim === 'paren') {
          if (next.trees.length !== 1) {
            throw Error('UnsupportedDynamicRuntimeImport');
          }
          edges.push({kind: 'module', specifier: literalSpecifier(next.trees[0])});
        } else if (next?.kind === 'string') {
          edges.push({kind: 'module', specifier: literalSpecifier(next)});
        } else {
          let cursor = i + 1;
          while (cursor < trees.length && trees[cursor].text !== ';'
              && trees[cursor].text !== 'from') cursor += 1;
          if (trees[cursor]?.text !== 'from') throw Error('UnsupportedRuntimeImport');
          edges.push({kind: 'module', specifier: literalSpecifier(trees[cursor + 1])});
        }
      } else if (tree.text === 'from') {
        edges.push({kind: 'module', specifier: literalSpecifier(next)});
      } else if (tree.text === 'new' && next?.text === 'URL') {
        const group = trees[i + 2];
        if (group?.$ !== 'group' || group.delim !== 'paren'
            || group.trees.slice(1).map(item => item.text).join('') !== ',import.meta.url') {
          throw Error('UnsupportedRuntimeAsset');
        }
        edges.push({kind: 'asset', specifier: literalSpecifier(group.trees[0])});
      }
    }
  }
  visit(tokenizeRuntime(source));
  return edges;
}
export function collectSourceRuntimeClosure(root, declaration, candidates = []) {
  root = fs.realpathSync(root);
  if (declaration?.schema !== 'source-runtime-closure/v1'
      || !Array.isArray(declaration.files) || !Array.isArray(declaration.entrypoints)
      || declaration.files.length === 0 || declaration.files.length > 256
      || declaration.entrypoints.length === 0) throw Error('UnsupportedUnclosedRuntime');
  const rows = new Map(), reachable = new Set();
  let totalBytes = 0;
  for (const row of declaration.files) {
    if (!runtimeMember(row.path) || !['module', 'asset'].includes(row.kind)
        || rows.has(row.path) || !/^[a-f0-9]{64}$/u.test(row.sha256)) {
      throw Error('UnboundRuntimeMember');
    }
    const source = physical(root, row.path);
    if (source === null || hash(bounded(path.join(root, row.path))) !== row.sha256) throw Error('RuntimeSourceDrift');
    const bytes = bounded(path.join(root, row.path));
    totalBytes += bytes.length;
    if (totalBytes > 8 * 1024 * 1024) throw Error('RuntimeClosureByteBound');
    rows.set(row.path, {...row, bytes: bytes.length});
  }
  function edge(owner, dependency) {
    if (dependency.specifier.startsWith('node:')) {
      if (dependency.kind !== 'module' || !pureBuiltins.has(dependency.specifier)) {
        throw Error('UnsupportedRuntimeBuiltin');
      }
      return;
    }
    if (!dependency.specifier.startsWith('./') && !dependency.specifier.startsWith('../')) {
      throw Error('UnsupportedRuntimePackage');
    }
    const target = path.posix.normalize(path.posix.join(path.posix.dirname(owner), dependency.specifier));
    if (!runtimeMember(target) || !rows.has(target)
        || rows.get(target).kind !== dependency.kind) throw Error('UnclosedRuntimeDependency');
    visit(target);
  }
  function visit(owner) {
    if (!runtimeMember(owner) || !rows.has(owner)) throw Error('UnclosedRuntimeEntrypoint');
    if (reachable.has(owner)) return;
    reachable.add(owner);
    const row = rows.get(owner);
    if (row.kind === 'module') {
      for (const dependency of runtimeEdges(physical(root, owner))) edge(owner, dependency);
    }
  }
  for (const entrypoint of declaration.entrypoints) {
    if (rows.get(entrypoint)?.kind !== 'module') throw Error('UnclosedRuntimeEntrypoint');
    visit(entrypoint);
  }
  for (const candidate of candidates) {
    if (!ownerPath(candidate.owner) || typeof candidate.after !== 'string') {
      throw Error('UnboundRuntimeCandidate');
    }
    if (candidate.owner.endsWith('.mjs')) {
      for (const dependency of runtimeEdges(candidate.after)) edge(candidate.owner, dependency);
    }
  }
  if (reachable.size !== rows.size) throw Error('UndeclaredRuntimeReachability');
  const closure = Object.freeze({schema: 'qualified-source-runtime-closure/v1', root,
    entrypoints: Object.freeze([...declaration.entrypoints]),
    files: Object.freeze([...rows.values()].map(row => Object.freeze(row))), totalBytes,
    authority: 'not-granted', ordinaryInstalledAcceptance: false});
  verifiedRuntimeClosures.add(closure);
  return closure;
}
export function stageSourceRuntimeClosure(closure, output) {
  if (!verifiedRuntimeClosures.has(closure)) {
    throw Error('UnqualifiedRuntimeClosure');
  }
  output = path.resolve(output);
  const parent = path.dirname(output);
  if (!fs.existsSync(parent) || fs.realpathSync(parent) !== parent
      || output === closure.root || output.startsWith(closure.root + path.sep)) {
    throw Error('UnownedRuntimeOutput');
  }
  if (fs.existsSync(output)) throw Error('FreshRuntimeOutputRequired');
  for (const row of closure.files) {
    if (!runtimeMember(row.path) || hash(bounded(path.join(closure.root, row.path))) !== row.sha256
        || physical(closure.root, row.path) === null) throw Error('RuntimeSourceDrift');
  }
  fs.mkdirSync(output);
  for (const row of closure.files) {
    const target = path.join(output, row.path);
    fs.mkdirSync(path.dirname(target), {recursive: true});
    fs.writeFileSync(target, bounded(path.join(closure.root, row.path)), {flag: 'wx'});
    if (hash(bounded(target)) !== row.sha256) throw Error('RuntimeCopyDrift');
  }
  return {files: closure.files.length, totalBytes: closure.totalBytes, authority: 'not-granted'};
}

function exclusiveJSON(file,
value){
  const fd=fs.openSync(file,
  'wx');
  try{
    fs.writeFileSync(fd,
    JSON.stringify(value,
    null,
    2)+'\n');
    fs.fsyncSync(fd);
  }
  finally{
    fs.closeSync(fd);
  }
}
export async function integrateSourcePackets(root,
packets,
output,
{
  apply=false, runtimeClosure=null}
={
}
){
  output=path.join(fs.realpathSync(path.dirname(path.resolve(output))),
  path.basename(output));
  if(output===fs.realpathSync(root)||output.startsWith(fs.realpathSync(root)+path.sep))throw Error('OutputMustBeOutsideRepository');
  fs.mkdirSync(output);
  const plan=prepareSourcePackets(root,
  packets);
  exclusiveJSON(path.join(output,
  'admission.json'),
  plan);
  if(!apply||!plan.ready)return {
    ready:plan.ready,
    applied:false,
    refusedOwners:plan.sources.filter(row=>row.status==='requires-source-owned-rebase').map(row=>row.owner)}
  ;
  let closure;
  try {
    closure = collectSourceRuntimeClosure(plan.root, runtimeClosure, plan.sources);
    for (const entrypoint of ['experiments/js_dogfood/drive.mjs',
      'js/agentic/node-host.mjs', 'js/server/worker-host.mjs', 'js/agentic/planner.mjs']) {
      if (!closure.entrypoints.includes(entrypoint)) throw Error('UnclosedRuntimeEntrypoint');
    }
  } catch (error) {
    exclusiveJSON(path.join(output, 'runtime-refusal.json'), {reason: error.message,
      qualification: 'Unknown runtime closure; no directory copies or module execution',
      authority: 'not-granted', applied: false});
    return {ready: plan.ready, applied: false, runtimeClosure: 'Unknown', reason: error.message};
  }
  exclusiveJSON(path.join(output, 'runtime-closure.json'), closure);
  const runtime = path.join(output, 'runtime');
  stageSourceRuntimeClosure(closure, runtime);
  const load = rel => import(pathToFileURL(path.join(runtime, rel)).href);
  const {
    drive}
  =await load('experiments/js_dogfood/drive.mjs');
  const {
    installNodeHost}
  =await load('js/agentic/node-host.mjs');
  const {
    WorkerHost}
  =await load('js/server/worker-host.mjs');
  const {
    planChatStep}
  =await load('js/agentic/planner.mjs');
  await installNodeHost(new WorkerHost());
  const receipts=[];
  for(let index=0;
  index<plan.sources.length;
  index++){
    const row=plan.sources[index],
    target=path.join(runtime,
    row.owner);
    fs.mkdirSync(path.dirname(target),
    {
      recursive:true}
    );
    if(row.status==='already-current'){
      if(!fs.existsSync(target))fs.copyFileSync(path.join(plan.root,
      row.owner),
      target);
      if(hash(bounded(target))!==row.afterSHA256)throw Error('ScratchCurrentIdentityDrift');
      receipts.push({
        owner:row.owner,
        status:'already-current',
        autonomousCredit:0}
      );
      continue;
    }
    if(!fs.existsSync(target)&&row.before!==null)fs.copyFileSync(path.join(plan.root,
    row.owner),
    target);
    if(fs.realpathSync(path.dirname(target))!==path.dirname(target))throw Error('UnownedScratchSourceParent');
    const before=fs.existsSync(target)?bounded(target).toString('utf8'):null;
    if((before===null?null:hash(before))!==row.beforeSHA256)throw Error('ScratchPreimageDrift');
    const task=before===null?`Create file ${row.owner} containing «${row.after}»`:`In ${row.owner} replace «${before}» with «${row.after}»`;
exclusiveJSON(path.join(output,`request-${index}.json`),
{
owner:row.owner,
task,
taskBytes:Buffer.byteLength(task),
taskSHA256:hash(task),
origin:'supplied-reviewed-general-capability-source',
autonomousCredit:0}
);
let result;
try{
result=await drive(planChatStep,
runtime,
task,
{
steps:12}
);
}
catch(error){
result={
failure:{
name:error.name,
message:error.message,
stack:error.stack}
}
;
}
const observed=fs.existsSync(target)?hash(bounded(target)):null,
accepted=observed===row.afterSHA256;
exclusiveJSON(path.join(output,`result-${index}.json`),
{
owner:row.owner,
result,
observedSHA256:observed,
accepted,
usageTokens:null,
currencyCost:null}
);
receipts.push({
owner:row.owner,
accepted,
observedSHA256:observed}
);
if(!accepted){
exclusiveJSON(path.join(output,
'failed-summary.json'),
{
receipts,
applied:false}
);
return {
applied:false,
failedOwner:row.owner}
;
}
}
exclusiveJSON(path.join(output,
'applied-summary.json'),
{
receipts,
applied:true,
runtime,
ordinaryInstalledAcceptance:false,
suppliedPatchAutonomousCredit:0}
);
return {
applied:true,
runtime,
ordinaryInstalledAcceptance:false}
;
}
if(import.meta.url===pathToFileURL(path.resolve(process.argv[1]??'.')).href){
const [root,
output,
...rest]=process.argv.slice(2),
apply=rest.includes('--apply');
console.log(JSON.stringify(await integrateSourcePackets(root,
rest.filter(x=>x!=='--apply'),
output,
{
apply}
)));
}
