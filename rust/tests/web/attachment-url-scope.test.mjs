import test from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const host=new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}});
for(const filename of ['tiny.png','invoice.pdf','data.custom-format','example.com'])test('attachment metadata is not a requested web operand: '+filename,async()=>{
 const prompt='Read attached image\n\nAttached files:\n1. '+filename+' (image/png, 68 B)\nOCR text: https://quoted.example.org/content';
 assert.equal(await host.run('firstUrlCandidate(__prompt)',{__prompt:prompt}),null);
 assert.equal(await host.run('extractHttpFetchUrl(__prompt,normalizePrompt(__prompt))',{__prompt:prompt}),null);
});
test('an explicit request URL before the attachment envelope remains authoritative',async()=>{
 const prompt='Fetch https://docs.example.org/guide\n\nAttached files:\n1. tiny.png (image/png, 68 B)';
 const result=await host.run('firstUrlCandidate(__prompt)',{__prompt:prompt});assert.equal(result.url,'https://docs.example.org/guide');
});
test('attachment boundaries retain case and CRLF compatibility without an extension catalog',async()=>{
 const prompt='Read attached file\r\n\r\n  ATTACHED FILES:  \r\n1. unknown.anything (application/octet-stream, 1 B)';
 assert.equal(await host.run('firstUrlCandidate(__prompt)',{__prompt:prompt}),null);
 assert.equal((await host.run('firstUrlCandidate("Read example.org")')).url,'https://example.org');
});
