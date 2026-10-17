import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {fileReadTaskFor} from '../../../js/agentic/file_read.mjs';
import {agenticMessage} from '../../../js/agentic/messages.mjs';
const fixture=JSON.parse(fs.readFileSync(new URL('../fixtures/agentic-file-read-cues.json',import.meta.url)));
await installNodeHost(new WorkerHost({WebAssembly:{
  instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');},
}}));
test('every shared read cue preserves its exact text, including leading spaces',()=>{
  for(const {key,text} of fixture.cues)assert.equal(agenticMessage(key),text,key);
});
test('production file-read routing preserves the captured original containment semantics',()=>{
  for(const {prompt,expected} of fixture.cases)assert.deepEqual(JSON.parse(JSON.stringify(fileReadTaskFor(prompt))),expected,prompt);
});
test('native message metadata and JavaScript share exact source data',()=>{
  const root=new URL('../../../',import.meta.url);
  assert.equal(fs.readFileSync(new URL('data/meta/agentic-messages.lino',root),'utf8'),
    fs.readFileSync(new URL('rust/embedded/data/meta/agentic-messages.lino',root),'utf8'));
});
