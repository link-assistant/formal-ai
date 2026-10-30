import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {once} from 'node:events';
import fs from 'node:fs';
import {extractFailures} from './check-web-archive.mjs';
import {classifyFailures,recheckAll,ACCEPTED_STATUS} from './recheck-broken-links.mjs';
test('main badge timeout exception excludes only the decorative image, not its destination',()=>{
 const url='https://img.shields.io/crates/v/formal-ai?label=crates.io&style=flat';
 const report=`## Errors per input\n\n### Errors in README.md\n\n* [408] <${url}> | Rejected status code: 408 Request Timeout\n`;
 assert.equal(classifyFailures(extractFailures(report)).unanswered[0].url,url);
 const patterns=fs.readFileSync(new URL('../.lycheeignore',import.meta.url),'utf8')
  .split(/\r?\n/u).map(line=>line.trim()).filter(line=>line&&!line.startsWith('#')).map(line=>new RegExp(line));
 const ignored=value=>patterns.some(pattern=>pattern.test(value));
 assert.equal(ignored(url),true);
 for(const checked of ['https://crates.io/crates/formal-ai','https://img.shields.io/crates/v/other?label=crates.io&style=flat',url+'/missing','https://img.shields.io/docsrs/formal-ai?label=docs.rs&style=flat'])assert.equal(ignored(checked),false,checked);
});
test('real transient 408 requires a later accepted response within unchanged attempt rules',async()=>{
 let requests=0;
 const server=http.createServer((request,response)=>{requests++;response.writeHead(requests===1?408:200);response.end();});
 server.listen(0,'127.0.0.1');await once(server,'listening');
 try {
  const url='http://127.0.0.1:'+server.address().port+'/badge';
  assert.equal(classifyFailures([{url,marker:'408',detail:'Rejected status code: 408'}]).unanswered.length,1);
  const result=(await recheckAll([url],{waitMs:1})).get(url);
  assert.equal(result.outcome,'alive');assert.equal(result.attempts,2);assert.equal(requests,2);
 } finally {server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('persistent real 408 stays broken and real 404 is final',async()=>{
 let requests=0;
 const server=http.createServer((request,response)=>{requests++;response.writeHead(request.url==='/missing'?404:408);response.end();});
 server.listen(0,'127.0.0.1');await once(server,'listening');
 try {
  const base='http://127.0.0.1:'+server.address().port;
  const results=await recheckAll([base+'/timeout',base+'/missing'],{waitMs:1});
  assert.equal(results.get(base+'/timeout').outcome,'retryable');assert.equal(results.get(base+'/timeout').attempts,3);
  assert.equal(results.get(base+'/missing').outcome,'rejected');assert.equal(results.get(base+'/missing').attempts,1);assert.equal(requests,4);
  assert.equal(classifyFailures([{url:base,marker:'404',detail:'Rejected status code: 408'}]).answered.length,1);
  assert.equal(ACCEPTED_STATUS,'200..=204,429,500..=599');
 } finally {server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
