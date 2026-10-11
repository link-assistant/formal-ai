import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {once} from 'node:events';
import {classifyFailures,recheckAll,ACCEPTED_STATUS} from './recheck-broken-links.mjs';
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
