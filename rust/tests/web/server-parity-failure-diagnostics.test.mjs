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
