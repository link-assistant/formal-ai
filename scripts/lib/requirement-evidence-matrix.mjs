// Byte-bound requirement citations are evidence inventory, never fresh acceptance proof.
import fs from 'node:fs';
import path from 'node:path';
import {
  createHash}
from 'node:crypto';
import {
  fileURLToPath}
from 'node:url';
import {
  ledgerRecords}
from '../render-progressive-plan.mjs';
import {
  requirementRows,
  isDefinitionLine}
from '../generate-requirement-status.mjs';
import {
  requirementIds}
from './requirements-register.mjs';
export const matrixDigest=bytes=>createHash('sha256').update(bytes).digest('hex');
const compare=(left,
right)=>left<right?-1:left>right?1:0;
export function requirementCitations(text) {
  text=text.replace(/https?:\/\/[^\s)`]+/gu,
  '');
  const candidates=[...text.matchAll(/(?:^|[^A-Za-z0-9_])((?:js|ts|rust|scripts|data|docs|tests|desktop|experiments|\.github)\/[A-Za-z0-9_.\/-]+(?:\{[A-Za-z0-9_,.-]+\})?)/gu)].flatMap(match=>{
    const value=match[1].replace(/[.,:]+$/u,
    '');
    const group=/^(.*)\{([^}]+)\}$/u.exec(value);
    return group?group[2].split(',').map(suffix=>group[1]+suffix):[value];
  }
  );
  return [...new Set(candidates)].sort(compare);
}
function binding(root,
relative) {
  if(path.isAbsolute(relative)||relative.split('/').some(part=>part==='..'||part==='.'||!part))return {
    path:relative,
    status:'Unsafe',
    sha256:null}
  ;
  let current=root;
  for(const component of relative.split('/')){
    current=path.join(current,
    component);
    let info;
    try{
      info=fs.lstatSync(current);
    }
    catch(error){
      if(error.code==='ENOENT')return {
        path:relative,
        status:'Missing',
        sha256:null}
      ;
      throw error;
    }
    if(info.isSymbolicLink())return {
      path:relative,
      status:'Symlink',
      sha256:null}
    ;
  }
  const info=fs.lstatSync(current);
  if(info.isDirectory())return {
    path:relative,
    status:'Directory',
    sha256:null}
  ;
  if(!info.isFile())return {
    path:relative,
    status:'Unsupported',
    sha256:null}
  ;
  const bytes=fs.readFileSync(current);
  return {
    path:relative,
    status:'Bound',
    sha256:matrixDigest(bytes),
    bytes:bytes.length}
  ;
}
export function requirementEvidenceMatrix(root,
{
  declaredIds=[],
  candidateManifests=[]}
={
}
) {
  const bindings=new Map(),
  observe=relative=>{
    if(!bindings.has(relative))bindings.set(relative,
    binding(root,
    relative));
    return bindings.get(relative);
  }
  ;
  const directoryInputs=['docs/requirements',
  'docs/requirements/assembled',
  'data/meta/requirement-status-ledger'];
  const directoryInventory=()=>directoryInputs.map(directory=>({
    directory,
    names:fs.existsSync(path.join(root,
    directory))?fs.readdirSync(path.join(root,
    directory)).sort():null}
  ));
  const structure=directoryInventory();
  for(const entry of structure)for(const name of entry.names??[])if(/\.(md|lino)$/u.test(name))observe(entry.directory+'/'+name);
  if(fs.existsSync(path.join(root,
  'docs/requirements-traceability.md')))observe('docs/requirements-traceability.md');
  if(new Set(candidateManifests).size!==candidateManifests.length)throw new Error('duplicate candidate manifest');
  const rows=requirementRows(root);
  const manifestBindings=[];
  const ledger=new Map();
 // Bind all assembled definitions and ledger shards, so admission/derived drift is visible.
 for(const directory of ['docs/requirements/assembled',
  'data/meta/requirement-status-ledger']){
    if(!fs.existsSync(path.join(root,
    directory)))continue;
    for(const name of fs.readdirSync(path.join(root,
    directory)).sort())if(/\.(md|lino)$/u.test(name)){
      observe(directory+'/'+name);
      if(directory==='data/meta/requirement-status-ledger')for(const record of ledgerRecords(fs.readFileSync(path.join(root,
      directory,
      name),
      'utf8'))){
        if(ledger.has(record.id))throw new Error('duplicate generated ledger requirement: '+record.id);
        ledger.set(record.id,
        record);
      }
    }
  }
  for(const relative of ['data/meta/requirement-status-ledger.lino',
  'docs/requirements-traceability.md'])if(fs.existsSync(path.join(root,
  relative)))observe(relative);
  const records=rows.map(row=>{
    const source=fs.readFileSync(path.join(root,
    row.shard),
    'utf8');
    observe(row.shard);
    const definitions=source.split('\n').filter(line=>isDefinitionLine(line)&&requirementIds(line)[0]===row.id);
    if(definitions.length>1)throw new Error('duplicate owning definition: '+row.id);
    const definition=definitions[0]??null;
    const citations=requirementCitations(definition??'');
    if(row.automatedTest)citations.push(row.automatedTest);
    const paths=[...new Set(citations)].sort(compare).map(observe);
    const test=row.automatedTest?observe(row.automatedTest):null;
    return {
      id:row.id,
      shard:row.shard,
      definitionSha256:definition===null?null:matrixDigest(definition),
      documentedVerdict:row.verdict,
      classification:definition===null?'Undrafted':row.verdict==='implemented'?'DeliveredClaim':row.verdict==='partial'?'Partial':row.verdict==='not-delivered'?'NotDelivered':'Closed',
      generatedLedger:ledger.get(row.id)??null,
      ledgerConsistency:!ledger.has(row.id)?'Missing':ledger.get(row.id).verdict===row.verdict&&ledger.get(row.id).test===row.automatedTest&&ledger.get(row.id).shard===row.shard?'Matches':'Mismatch',
      automatedTest:test,
      testRoot:row.automatedTest.endsWith('.rs')?'Rust':row.automatedTest?'JavaScript':'None',
      citedPaths:paths,
      freshAcceptance:'Unknown',
      freshAcceptanceReason:'Source/test citation bytes do not prove test execution or requirement completion',
      manualConfirmation:row.manual}
    ;
  }
  );
  const ids=new Set(records.map(row=>row.id));
  if(new Set(declaredIds).size!==declaredIds.length)throw new Error('duplicate declared requirement');
  for(const id of [...declaredIds].sort(compare))if(!ids.has(id))records.push({
    id,
    classification:'Undrafted',
    documentedVerdict:null,
    freshAcceptance:'Unknown',
    citedPaths:[],
    automatedTest:null,
    testRoot:'None'}
  );
  for(const name of [...candidateManifests].sort(compare)){
    const bytes=fs.readFileSync(name),
    manifest=JSON.parse(bytes);
    const sources=manifest.sources??manifest.source??manifest.changes;
    if(!Array.isArray(sources))throw new Error('unsupported candidate source manifest');
    const seen=new Set();
    const candidateSources=sources.map(source=>{
      const relative=source.path??source.target,
      after=source.afterSha256??source.afterSHA256??source.after;
      if(typeof relative!=='string'||seen.has(relative)||!/^[a-f0-9]{64}$/u.test(after))throw new Error('invalid or duplicate candidate source');
      seen.add(relative);
      const current=observe(relative);
      return {
        path:relative,
        candidateAfterSha256:after,
        current,
        status:current.sha256===after?'InstalledExactBytes':current.status==='Missing'?'ReadyOnlyAbsent':'ReadyOnlyDifferent',
        semanticCountCredit:0}
      ;
    }
    );
    manifestBindings.push({
      path:name,
      sha256:matrixDigest(bytes),
      sources:candidateSources,
      qualification:'Candidate source bytes only; no delivery or semantic count credit'}
    );
  }
  records.sort((left,
  right)=>compare(left.id,
  right.id));
  if(JSON.stringify(structure)!==JSON.stringify(directoryInventory()))throw new Error('requirement input structure changed during matrix capture');
  for(const [relative,
  original] of bindings)if(JSON.stringify(binding(root,
  relative))!==JSON.stringify(original))throw new Error('source changed during requirement matrix capture: '+relative);
  const counts={
  }
  ;
  for(const record of records)counts[record.classification]=(counts[record.classification]??0)+1;
  return {
    schema:'requirement-source-test-evidence-matrix/v1',
    scope:'Declared ledger claims and exact cited bytes; no fresh execution or completion credit',
    sourceClosure:'Cited files and canonical ledger inputs only; not a complete execution closure',
    structure,
    producerSourceSha256:matrixDigest(fs.readFileSync(fileURLToPath(import.meta.url))),
    ledgerMismatches:records.filter(record=>record.ledgerConsistency==='Mismatch').map(record=>record.id),
    declaredIds:[...declaredIds].sort(compare),
    counts,
    records,
    bindings:[...bindings.values()].sort((left,
    right)=>compare(left.path,
    right.path)),
    candidateManifests:manifestBindings,
    freshAcceptance:'Unknown',
    suppliedProgramAutonomyCredit:0}
  ;
}
export function checkRequirementEvidenceMatrix(recorded,
root,
options={
}
) {
  const current=requirementEvidenceMatrix(root,
  options);
  if(JSON.stringify(recorded)!==JSON.stringify(current))throw new Error('stale or counterfeit requirement evidence matrix');
  if(current.ledgerMismatches.length)throw new Error('generated requirement ledger disagrees with owning source');
  return current;
}
