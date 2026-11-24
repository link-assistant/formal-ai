import test from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {looksLikeHostname as crateHostname,normalizeUrlCandidate as crateUrl} from '../../../js/agentic/crate/solver_handlers_web_search.mjs';
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

// The native online/deny fixture uses this ordinary dotted IPv4 operand.
for(const address of ['127.0.0.1','10.2.3.4','192.168.0.255','0.0.0.0'])test('a fetch request preserves its IPv4 operand: '+address,async()=>{
 const url='http://'+address+':43127/source';
 const prompt='Fetch '+url;
 assert.equal(crateUrl(url),url);
 assert.equal(await host.run('extractHttpFetchUrl(__prompt,normalizePrompt(__prompt))',{__prompt:prompt}),url);
});
for(const address of ['999.999.999.999','256.0.0.1','127.0.1','127.00.0.1'])test('an invalid dotted numeric address is refused: '+address,async()=>{
 assert.equal(crateHostname(address),false);
 assert.equal(await host.run('looksLikeHostname(__address)',{__address:address}),false);
});

// Keep the domain-label boundary identical in both maintained JavaScript paths.
for (const [address, expected] of [
 ['a.example',true], ['a-b.example',true], ['A1.B2',true], ['1.example',true],
 ['example',false], ['.example.org',false], ['example.org.',false],
 ['example..org',false], ['example.c',false], ['-a.example',false],
 ['a-.example',false], ['a.-example',false], ['a.example-',false],
 ['a_b.example',false], ['café.example',false], ['',false],
]) test('domain-label boundary remains equivalent: '+JSON.stringify(address),async()=>{
 assert.equal(crateHostname(address),expected);
 assert.equal(await host.run('looksLikeHostname(__address)',{__address:address}),expected);
});
