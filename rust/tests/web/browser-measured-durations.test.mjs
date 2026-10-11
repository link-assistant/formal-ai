import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import { browserPlan, contentDigest, completedTestSummary, collectBrowserShards } from '../../../scripts/lib/browser-coverage-shards.mjs';
import { measuredFileDurations, deriveBrowserDurations, browserDurationStatus, readBrowserDurationPackets } from '../../../scripts/lib/browser-measured-durations.mjs';

const repository = new URL('../../../', import.meta.url).pathname;
const reporter = path.join(repository, 'scripts/browser-duration-reporter.mjs');
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'browser-measured-duration-'));
const identity = { source: 'local-fixture-source', run: 'local-fixture-run', attempt: '1',
  sourceDigest: 'local-production-fixture', durationReporterDigest: contentDigest(fs.readFileSync(reporter)) };
const inventory = [];
fs.mkdirSync(path.join(directory, 'rust/tests/web'), {recursive:true});
fs.mkdirSync(path.join(directory, 'js'));
fs.writeFileSync(path.join(directory, 'js/source.mjs'), 'export function warm(){return 1;}\nexport function cold(){return 0;}\n');
for(let index=0;index<6;index+=1){
  const relative=`rust/tests/web/measured-${index}.test.mjs`;
  const source=`import test from 'node:test';\nimport assert from 'node:assert/strict';\nimport {warm} from '../../../js/source.mjs';\ntest('actual file ${index}',()=>assert.equal(warm(),1));\n`;
  fs.writeFileSync(path.join(directory,relative),source);
  inventory.push({path:relative,sha256:contentDigest(source),bytes:Buffer.byteLength(source)});
}
const plan=browserPlan(inventory);
const packets=plan.shards.map((selected,index)=>{
  const coveragePath=path.join(directory,`coverage-${index}.info`);
  const durationPath=path.join(directory,`durations-${index}.jsonl`);
  const environment={...process.env};delete environment.NODE_TEST_CONTEXT;
  const result=spawnSync(process.execPath,['--test','--experimental-test-coverage',
    '--test-reporter=lcov',`--test-reporter-destination=${coveragePath}`,
    '--test-reporter=spec','--test-reporter-destination=stdout',
    `--test-reporter=${reporter}`,`--test-reporter-destination=${durationPath}`,
    ...selected.map(relative=>path.join(directory,relative))],{cwd:directory,env:environment,encoding:'utf8'});
  assert.equal(result.status,0,result.stderr);assert.equal(result.signal,null);
  const coverage=fs.readFileSync(coveragePath),durations=fs.readFileSync(durationPath);
  const stdout=Buffer.from(result.stdout),stderr=Buffer.from(result.stderr);
  return{coverage,durations,stdout,stderr,receipt:{...identity,shard:index+1,total:6,planDigest:plan.digest,
    selected,exitCode:result.status,signal:result.signal,sourceUnchanged:true,
    summary:completedTestSummary(result.stdout),coverageDigest:contentDigest(coverage),
    stdoutDigest:contentDigest(stdout),stderrDigest:contentDigest(stderr),
    durationsDigest:contentDigest(durations),executionRoot:directory,nodeVersion:process.version}};
});
const clone=()=>packets.map(packet=>({...packet,receipt:structuredClone(packet.receipt)}));

try{
 test('actual Node coverage events produce exact measured source-owned file durations and unchanged raw LCOV',()=>{
  const table=deriveBrowserDurations(plan,identity,packets);
  assert.equal(deriveBrowserDurations(plan,identity,[...packets].reverse()),table);
  const merged=collectBrowserShards(plan,identity,packets);
  assert.deepEqual(merged,Buffer.concat(packets.map(packet=>packet.coverage)));
  for(const record of inventory){
    const packet=packets.find(packet=>packet.receipt.selected.includes(record.path));
    const seconds=measuredFileDurations(packet.durations,directory,packet.receipt.selected).get(record.path);
    assert.ok(Number.isFinite(seconds)&&seconds>=0);
    assert.ok(table.includes(`test ${JSON.stringify(record.path)}\n    source-sha256 ${JSON.stringify(record.sha256)}\n    seconds ${seconds}`));
  }
  assert.equal(browserDurationStatus(plan,identity,packets).status,'Measured');
 });
 for(const [name,mutate] of [
  ['missing shard',selected=>selected.pop()],
  ['duplicate shard',selected=>{selected[1]=selected[0];}],
  ['stale source',selected=>{selected[0].receipt.source='foreign';}],
  ['stale run',selected=>{selected[0].receipt.run='foreign';}],
  ['stale source digest',selected=>{selected[0].receipt.sourceDigest='foreign';}],
  ['stale plan',selected=>{selected[0].receipt.planDigest='foreign';}],
  ['failed coverage process',selected=>{selected[0].receipt.exitCode=1;}],
  ['mutated source',selected=>{selected[0].receipt.sourceUnchanged=false;}],
  ['missing legacy timing data',selected=>{delete selected[0].durations;}],
  ['counterfeit timing data',selected=>{selected[0].durations=Buffer.from('counterfeit\n');}],
  ['mixed measurement runtime',selected=>{selected[0].receipt.nodeVersion='v99.0.0';}],
  ['foreign reporter source',selected=>{selected[0].receipt.durationReporterDigest='foreign';}],
 ])test(`${name} remains Unknown without a replacement timing table`,()=>{
   const selected=clone();mutate(selected);const result=browserDurationStatus(plan,identity,selected);
   assert.equal(result.status,'Unknown');assert.equal(Object.hasOwn(result,'table'),false);
 });
 for(const [name,change] of [
  ['missing file duration',rows=>rows.splice(0,1)],
  ['duplicate file duration',rows=>rows.splice(0,0,rows[0])],
  ['foreign file duration',rows=>{rows[0].file=path.join(directory,'foreign.test.mjs');}],
  ['failed file duration',rows=>{rows[0].passed=false;}],
  ['skipped file duration',rows=>{rows[0].skipped=true;}],
  ['invalid duration',rows=>{rows[0].milliseconds=-1;}],
  ['incomplete stream',rows=>rows.pop()],
 ])test(`${name} is refused even with a matching recomputed byte digest`,()=>{
   const selected=clone();const rows=selected[0].durations.toString().trimEnd().split('\n').map(JSON.parse);
   change(rows);selected[0].durations=Buffer.from(rows.map(JSON.stringify).join('\n')+'\n');
   selected[0].receipt.durationsDigest=contentDigest(selected[0].durations);
   assert.equal(browserDurationStatus(plan,identity,selected).status,'Unknown');
 });
 test('raw duration artifact member census rejects legacy and extra members',()=>{
  const artifacts=path.join(directory,'artifacts');fs.mkdirSync(artifacts);
  const folder=path.join(artifacts,'shard-one');fs.mkdirSync(folder);
  const packet=packets[0];
  for(const [name,bytes]of [['coverage.info',packet.coverage],['stdout.txt',packet.stdout],['stderr.txt',packet.stderr],['durations.jsonl',packet.durations],['receipt.json',JSON.stringify(packet.receipt)]])fs.writeFileSync(path.join(folder,name),bytes);
  assert.equal(readBrowserDurationPackets(artifacts).length,1);
  fs.writeFileSync(path.join(folder,'extra.txt'),'unproved');assert.throws(()=>readBrowserDurationPackets(artifacts));fs.rmSync(path.join(folder,'extra.txt'));
  fs.rmSync(path.join(folder,'durations.jsonl'));assert.throws(()=>readBrowserDurationPackets(artifacts));
 });
 test('truncated duration bytes cannot certify completion',()=>{
  assert.throws(()=>measuredFileDurations(packets[0].durations.subarray(0,-1),directory,plan.shards[0]));
 });
}finally{
 process.on('exit',()=>fs.rmSync(directory,{recursive:true,force:true}));
}
