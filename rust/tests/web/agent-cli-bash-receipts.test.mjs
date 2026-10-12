import test from 'node:test';
import assert from 'node:assert/strict';
import {executeShellReceipt,encodeExecutionReceipt} from '../../../scripts/lib/agent-cli-bash-receipt.mjs';
const quote=(value)=>process.platform==='win32'?'"'+value.replaceAll('"','\\"')+'"':"'"+value.replaceAll("'","'\\''")+"'";
const command=(body)=>quote(process.execPath)+' -e '+quote(body);
const run=(body,timeout=2000)=>executeShellReceipt({command:command(body),cwd:process.cwd(),timeout,abort:new AbortController().signal});
test('actual process receipt preserves authored failure prose and real zero status',async()=>{
 const receipt=await run("process.stdout.write('failed is authored text\\n')");
 assert.deepEqual(JSON.parse(encodeExecutionReceipt(receipt)),{
  schema:'bash-execution-receipt/v1',command:command("process.stdout.write('failed is authored text\\n')"),cwd:process.cwd(),exit_code:0,signal:null,
  stream_complete:true,complete:true,truncated:false,timed_out:false,aborted:false,
  stdout:'failed is authored text\n',stderr:'',stdout_bytes:24,stderr_bytes:0,retained_code_units:24,output_limit_code_units:30000,
 });
});
test('actual nonzero exit and stderr survive receipt encoding',async()=>{
 const receipt=JSON.parse(encodeExecutionReceipt(await run("process.stdout.write('same bytes');process.stderr.write('actual error');process.exitCode=9")));
 assert.equal(receipt.exit_code,9);assert.equal(receipt.stdout,'same bytes');assert.equal(receipt.stderr,'actual error');assert.equal(receipt.complete,true);
});
test('large real process output is bounded while original length and incomplete witness remain',async()=>{
 const receipt=await run("process.stdout.write('x'.repeat(33392))");
 assert.equal(receipt.exit_code,0);assert.equal(receipt.stdout_bytes,33392);assert.equal(receipt.stdout,'x'.repeat(30000));
 assert.equal(receipt.truncated,true);assert.equal(receipt.complete,false);assert.equal(receipt.stream_complete,true);
});
test('real split UTF8 chunks preserve exact source text',async()=>{
 const receipt=await run("let b=Buffer.from('😀漢');process.stdout.write(b.subarray(0,1));setTimeout(()=>process.stdout.write(b.subarray(1)),20)");
 assert.equal(receipt.stdout,'😀漢');assert.equal(receipt.stdout_bytes,7);assert.equal(receipt.complete,true);
});
test('actual timeout never advertises complete bytes',async()=>{
 const receipt=await run("setTimeout(()=>{},10000)",30);
 assert.equal(receipt.timed_out,true);assert.equal(receipt.complete,false);assert.equal(receipt.stream_complete,true);
});
test('actual abort never advertises complete bytes',async()=>{
 const controller=new AbortController();setTimeout(()=>controller.abort(),30);
 const receipt=await executeShellReceipt({command:command("setTimeout(()=>{},10000)"),cwd:process.cwd(),timeout:2000,abort:controller.signal});
 assert.equal(receipt.aborted,true);assert.equal(receipt.complete,false);assert.equal(receipt.stream_complete,true);
});
