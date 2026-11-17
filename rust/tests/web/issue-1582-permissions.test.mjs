// Shared language regressions for issue #1582; supplied, zero authorship credit.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const fixtures=JSON.parse(readFileSync(new URL('../../../data/parity/issue-1582-permissions.json',import.meta.url),'utf8'));
const examples=JSON.parse(readFileSync(new URL('../../../data/parity/issue-1582-permission-answers.json',import.meta.url),'utf8'));
const host=new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}});
for(const row of fixtures)test(row.language+': '+row.prompt,{timeout:15000},async()=>{
  if(row.guidance){
    const result=await host.solve(row.prompt);
    assert.equal(result.intent,'capabilities');
    assert.equal(result.content,examples[row.language]);
    assert(result.evidence.includes('language:'+row.language),'detect the language without forcing it');
    assert(result.evidence.includes('permission:guidance'));
    assert.match(result.content,/FORMAL_AI_OFFLINE/u);
    assert.match(result.content,/\/permissions/u);
  }else{
    await host.boot();
    const answer=await host.run('tryFeatureCapabilityStatus(__prompt, normalizePrompt(__prompt), __language, {})',{__prompt:row.prompt,__language:row.language});
    assert.equal(answer?.evidence.includes('permission:guidance')??false,false,'a quoted permission phrase does not own the request');
  }
});

test('the generated TypeScript recognizer executes the same shared cases',{timeout:15000},async()=>{
  const source=readFileSync(new URL('../../../ts/worker/formal_ai_worker_capabilities_and_runtime_rules.ts',import.meta.url),'utf8');
  const helper=source.match(/function internetAccessHelpRequested[\s\S]*?(?=function tryFeatureCapabilityStatus)/u)?.[0];
  assert(helper,'the TypeScript projection must include the shared request recognizer');
  await host.boot();
  await host.run(helper);
  for(const row of fixtures){
    const accepted=await host.run('internetAccessHelpRequested(__prompt, normalizePrompt(__prompt))',{__prompt:row.prompt});
    assert.equal(accepted,row.guidance,row.prompt);
  }
});
