// Captured main failure lacked both server identity and retained child output.
import test from 'node:test';
import assert from 'node:assert/strict';
import {EventEmitter} from 'node:events';
import {retainServerDiagnostics} from '../../../scripts/check-server-parity.mjs';
function child(){return {stdout:new EventEmitter(),stderr:new EventEmitter(),exitCode:null,signalCode:null};}
test('runtime failures retain both channels and actual terminal identity',()=>{
 const process=child(),snapshot=retainServerDiagnostics(process,'rust');
 process.stdout.emit('data',Buffer.from('listening\n'));
 const bytes=Buffer.from('thread panicked: 雪🙂\n');
 process.stderr.emit('data',bytes.subarray(0,bytes.length-3));process.stderr.emit('data',bytes.subarray(bytes.length-3));
 process.signalCode='SIGABRT';const result=snapshot();
 assert.equal(result.label,'rust');assert.equal(result.signal,'SIGABRT');assert.equal(result.exitCode,null);
 assert.equal(result.stdout,'listening\n');assert.equal(result.stderr,bytes.toString());
 assert.equal(result.stderrDroppedBytes,0);assert.equal(result.stderrBytes,bytes.length);
});
test('diagnostic memory remains bounded while observed and dropped byte counts remain explicit',()=>{
 const process=child(),snapshot=retainServerDiagnostics(process,'js',8);
 process.stderr.emit('data',Buffer.from('0123456789abcdef'));process.stderr.emit('data',Buffer.from('gh'));
 process.exitCode=17;const result=snapshot();assert.equal(result.stderr,'abcdefgh');
 assert.equal(result.stderrBytes,18);assert.equal(result.stderrDroppedBytes,10);assert.equal(result.exitCode,17);
 assert.equal(result.stdout,'');assert.equal(result.stdoutDroppedBytes,0);
});
test('invalid diagnostic budgets are rejected',()=>{
 for(const limit of [0,-1,Infinity,1048577])assert.throws(()=>retainServerDiagnostics(child(),'rust',limit));
});

import http from 'node:http';
import {readFileSync} from 'node:fs';
import {sendParityRequest,loadCorpus} from '../../../scripts/check-server-parity.mjs';
test('original CI request receives a delayed Node HTTP response without client TCP EOF',async context=>{
 const request=loadCorpus(readFileSync('rust/tests/fixtures/server-parity/requests.lino','utf8')).requests.find(entry=>entry.id==='chat_standing_requirement');
 assert.ok(request);
 const answer=JSON.stringify({observed:true}),bodies=[];
 const server=http.createServer(async(incoming,response)=>{
  const chunks=[];for await(const chunk of incoming)chunks.push(chunk);
  bodies.push(Buffer.concat(chunks).toString('utf8'));
  await new Promise(resolve=>setTimeout(resolve,30));
  response.writeHead(200,{'content-type':'application/json','content-length':Buffer.byteLength(answer),'connection':'close'});
  response.end(answer);
 });
 await new Promise((resolve,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',resolve);});
 context.after(async()=>{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));});
 const response=await sendParityRequest('http://127.0.0.1:'+server.address().port,request,'JavaScript');
 assert.equal(response.status,200);assert.equal(response.body,answer);assert.deepEqual(bodies,[request.body]);
});
