import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import vm from 'node:vm';
import {createHash} from 'node:crypto';
import {createBrowserContext,plain} from './support/browser-runtime.mjs';
import {declaredWebSeedFiles} from '../../../scripts/browser-engine-cold-install.mjs';
const root = new URL('../../../',import.meta.url);
function memory() {
  const context=createBrowserContext();
  vm.runInContext(fs.readFileSync(new URL('js/memory.js',root),'utf8'),context);
  return context.FormalAiMemory;
}
test('full memory bundle preserves exact empty, newline, CRLF, Unicode and escaped seed bytes',()=>{
  const client=memory();
  for(const text of ['', '\n', '\n\n', 'first\n\nλ🙂\r\nlast\n\n', 'literal \\n and "quote"\r\t', '\0\n']) {
    const raw={'seed/renamed.lino':text};
    const bundle=client.exportFullMemory({seed:{raw},events:[{id:'original-event',content:'retained memory'}]});
    const parsed=client.importFullMemory(bundle);
    assert.equal(parsed.kind,'bundle');assert.deepEqual(plain(parsed.seedFiles),raw);
    assert.equal(parsed.events.length,1);assert.equal(parsed.events[0].content,'retained memory');
  }
});
test('every declared current web seed retains its original UTF8 bytes in a bundle',()=>{
  const client=memory();
  const names=declaredWebSeedFiles(fs.readFileSync(new URL('js/seed-files.js',root),'utf8'));
  const raw=Object.fromEntries(names.map(name=>[name,fs.readFileSync(new URL('data/'+name,root),'utf8')]));
  const parsed=client.importFullMemory(client.exportFullMemory({seed:{raw},events:[]}));
  assert.deepEqual(Object.keys(parsed.seedFiles).sort(),names.slice().sort());
  for(const name of names){
    assert.equal(parsed.seedFiles[name],raw[name],name);
    assert.equal(createHash('sha256').update(parsed.seedFiles[name]).digest('hex'),
      createHash('sha256').update(fs.readFileSync(new URL('data/'+name,root))).digest('hex'),name);
  }
});
test('legacy CRLF and legacy event-only documents keep their established reader',()=>{
  const client=memory();
  const text='formal_ai_bundle\r\n  seed_files\r\n    file "seed/legacy.lino"\r\n      legacy\r\n        value "ok"\r\n  demo_memory\r\n';
  assert.equal(client.importFullMemory(text).seedFiles['seed/legacy.lino'],'legacy\n  value "ok"');
  const events=[{id:'legacy-event',content:'unmodified old memory'}];
  const parsed=client.importFullMemory(client.exportLinksNotation(events));
  assert.equal(parsed.kind,'memory');assert.deepEqual(plain(parsed.seedFiles),{});
  assert.equal(parsed.events[0].content,events[0].content);
});
test('declared loaded inventory rejects executable, duplicate and unsafe package metadata',()=>{
  const declaration=values=>'self.FORMAL_AI_SEED_FILES = Object.freeze('+JSON.stringify(values)+');';
  assert.deepEqual(declaredWebSeedFiles(declaration(['seed/renamed.lino'])),['seed/renamed.lino']);
  for(const value of [declaration(['seed/duplicate.lino','seed/duplicate.lino']),
    declaration(['../outside.lino']),declaration(['seed/a.lino'])+' process.exit(0);',
    'self.FORMAL_AI_SEED_FILES = (()=>["seed/a.lino"])();',declaration([]),declaration([null])]){
    assert.throws(()=>declaredWebSeedFiles(value));
  }
});

test('seed transport preserves original bytes beside the unchanged expanded reader view',async()=>{
  const source='\ufeffmeaning\r\n  lexeme en "alpha" "beta"\r\n\r\n';
  const bytes=Buffer.from(source);
  const context=createBrowserContext({fetch:async()=>new Response(bytes)});
  vm.runInContext(fs.readFileSync(new URL('js/seed_loader.js',root),'utf8'),context);
  const seed=await context.FormalAiSeed.loadAll(['seed/transport-fixture.lino']);
  assert.equal(seed.sourceRaw['seed/transport-fixture.lino'],source);
  assert.deepEqual(Buffer.from(seed.sourceRaw['seed/transport-fixture.lino']),bytes);
  assert.notEqual(seed.raw['seed/transport-fixture.lino'],source);
  assert.match(seed.raw['seed/transport-fixture.lino'],/surface/u);
  const client=memory();
  const parsed=client.importFullMemory(client.exportFullMemory({seed:{raw:seed.sourceRaw},events:[]}));
  assert.equal(parsed.seedFiles['seed/transport-fixture.lino'],source);
});
test('actual worker seed dump and full memory roundtrip preserve every declared original source',async()=>{
  const {WorkerHost}=await import('../../../js/server/worker-host.mjs');
  const replies=[];
  const worker=new WorkerHost({postMessage:message=>replies.push(message)});
  const expanded=await worker.run('SEED_RAW');
  await worker.run('self.onmessage({data:{kind:"seed_dump",requestId:"source-roundtrip-control"}})');
  const dumps=replies.filter(reply=>reply.kind==='seed_dump');
  assert.equal(dumps.length,1);assert.equal(dumps[0].requestId,'source-roundtrip-control');
  const raw=plain(dumps[0].raw);
  const names=declaredWebSeedFiles(fs.readFileSync(new URL('js/seed-files.js',root),'utf8'));
  assert.deepEqual(Object.keys(raw).sort(),names.slice().sort());
  const client=memory();
  const parsed=client.importFullMemory(client.exportFullMemory({seed:{raw},events:[]}));
  for(const name of names){
    const bytes=fs.readFileSync(new URL('data/'+name,root));
    assert.deepEqual(Buffer.from(raw[name]),bytes,name);
    assert.deepEqual(Buffer.from(parsed.seedFiles[name]),bytes,name);
  }
  const changed=names.filter(name=>expanded[name]!==raw[name]);
  assert.ok(changed.length>0,'existing semantic readers still receive concise lexeme expansion');
  for(const name of changed)assert.match(expanded[name],/surface/u,name);
  assert.ok(replies.some(reply=>reply.kind==='engine_unavailable'),
    'Node source control does not execute or certify a WASM engine');
});
