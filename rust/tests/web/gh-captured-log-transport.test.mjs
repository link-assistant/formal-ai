import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {runGh} from '../../../scripts/collect-ci-failures.mjs';
const endpoint='repos/link-assistant/formal-ai/actions/jobs/121114230265/logs';
const limits={timeoutMs:2000,maxBytes:4096,captureRaw:true};
test('modern and legacy CLIs capture exact ANSI log bytes with one bounded capability probe',
 {skip:process.platform==='win32'},async()=>{
  for(const modern of [true,false]){
   const directory=fs.mkdtempSync(path.join(os.tmpdir(),'gh-log-capture-')),previous=process.env.PATH;
   try{
    const observations=path.join(directory,'calls.jsonl');
    const program=`#!${process.execPath}
const fs=require('node:fs'),args=process.argv.slice(2);
fs.appendFileSync(${JSON.stringify(observations)},JSON.stringify(args)+'\\n');
if(args[1]==='--help'){console.log(${JSON.stringify(modern?'--allow-escape-sequences':'--include --silent')});process.exit(0);}
if(${modern}&&!args.includes('--allow-escape-sequences')){console.error('the response contains terminal escape sequences');process.exit(1);}
if(!${modern}&&args.includes('--allow-escape-sequences')){console.error('unknown flag');process.exit(1);}
process.stdout.write('\\x1b[31mFinal issue URL: https://github.com/link-assistant/formal-ai/issues/1409\\x1b[0m\\n');
`;
    fs.writeFileSync(path.join(directory,'gh'),program,{mode:0o755});
    process.env.PATH=directory+path.delimiter+previous;
    for(let index=0;index<2;index++){
     const result=await runGh(['api',endpoint],limits);
     assert.equal(result.ok,true);assert.equal(result.streamComplete,true);assert.equal(result.tailComplete,true);
     assert.equal(result.text,'\x1b[31mFinal issue URL: https://github.com/link-assistant/formal-ai/issues/1409\x1b[0m\n');
    }
    const calls=fs.readFileSync(observations,'utf8').trim().split('\n').map(JSON.parse);
    assert.equal(calls.length,3,'one local help and exactly two requested API reads');
    assert.deepEqual(calls[0],['api','--help']);
    assert(calls.slice(1).every(args=>args.includes('--allow-escape-sequences')===modern));
   }finally{process.env.PATH=previous;fs.rmSync(directory,{recursive:true,force:true});}
  }
 });
test('raw output opt-in refuses mutations, metadata and arbitrary CLI commands before execution',async()=>{
 for(const args of [['api',endpoint,'--method','POST'],['api','repos/link-assistant/formal-ai/issues/1409'],['auth','token']]){
  await assert.rejects(runGh(args,limits));
 }
});
test('an unavailable capability probe fails closed without any API attempt or retry',
 {skip:process.platform==='win32'},async()=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'gh-log-no-capability-')),previous=process.env.PATH;
  const observations=path.join(directory,'calls.jsonl');
  try{
   fs.writeFileSync(path.join(directory,'gh'),`#!${process.execPath}
const fs=require('node:fs');fs.appendFileSync(${JSON.stringify(observations)},JSON.stringify(process.argv.slice(2))+'\\n');
console.error('help unavailable');process.exit(1);
`,{mode:0o755});
   process.env.PATH=directory+path.delimiter+previous;
   for(let index=0;index<2;index++){
    const result=await runGh(['api',endpoint],limits);
    assert.equal(result.ok,false);assert.equal(result.text,'');assert.match(result.error,/capability check unavailable/u);
   }
   assert.deepEqual(fs.readFileSync(observations,'utf8').trim().split('\n').map(JSON.parse),[['api','--help']]);
  }finally{process.env.PATH=previous;fs.rmSync(directory,{recursive:true,force:true});}
 });
test('complete UTF-8 logs preserve characters spanning the retained buffer boundary',
 {skip:process.platform==='win32'},async()=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'gh-log-unicode-')),previous=process.env.PATH;
  const expected='x'.repeat(63)+'🙂'+'\x1b[32mTAIL\x1b[0m\n';
  try{
   fs.writeFileSync(path.join(directory,'gh'),`#!${process.execPath}
if(process.argv[3]==='--help')console.log('--allow-escape-sequences');else process.stdout.write(${JSON.stringify(expected)});
`,{mode:0o755});
   process.env.PATH=directory+path.delimiter+previous;
   const result=await runGh(['api',endpoint],{...limits,maxBytes:128});
   assert.equal(result.ok,true);assert.equal(result.truncated,false);
   assert.equal(result.text,expected);assert.equal(result.bytesReceived,Buffer.byteLength(expected));
  }finally{process.env.PATH=previous;fs.rmSync(directory,{recursive:true,force:true});}
 });
