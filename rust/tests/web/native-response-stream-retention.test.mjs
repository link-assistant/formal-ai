import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,readFileSync,chmodSync,existsSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {createRawResponseStreams} from '../../../scripts/lib/native-response-capture.mjs';
const sinkUrl=new URL('../../../scripts/lib/native-response-capture.mjs',import.meta.url).href;
const executionUrl=new URL('../../../scripts/lib/coverage-execution.mjs',import.meta.url).href;
test('raw stream sinks preserve chunk bytes and reject text without claiming completion',()=>{
 const directory=mkdtempSync(join(tmpdir(),'native-raw-sink-'));
 try {
  const streams=createRawResponseStreams(directory);
  streams.onStdout(Buffer.from([0,255,13]));streams.onStdout(new Uint8Array([10,128]));
  streams.onStderr(Buffer.from([254,0,10]));
  assert.deepEqual(streams.read(),{stdout:Buffer.from([0,255,13,10,128]),stderr:Buffer.from([254,0,10])});
  assert.throws(()=>streams.onStdout('decoded text'),/raw byte chunk required/);
  assert.equal(existsSync(join(directory,'report.json')),false);
 } finally {rmSync(directory,{recursive:true,force:true});}
});
test('actual interrupted controller retains observed binary stdout and stderr before process completion',async()=>{
 const directory=mkdtempSync(join(tmpdir(),'native-raw-interruption-'));
 const stdout=Buffer.from([0,255,13,10,128]),stderr=Buffer.from([254,0,13,10]);
 let controller;let emitterPid;
 try {
  const emitter=join(directory,'emitter.mjs');
  writeFileSync(emitter,`#!/usr/bin/env node\nimport fs from 'node:fs';\nfs.writeFileSync(${JSON.stringify(join(directory,'emitter.pid'))},String(process.pid));\nprocess.stdout.write(Buffer.from(${JSON.stringify([...stdout])}));\nprocess.stderr.write(Buffer.from(${JSON.stringify([...stderr])}));\nsetTimeout(()=>process.exit(0),10000);\n`);chmodSync(emitter,0o755);
  const launcher=join(directory,'controller.mjs');
  writeFileSync(launcher,`import fs from 'node:fs';\nimport {createRawResponseStreams} from ${JSON.stringify(sinkUrl)};\nimport {executeBatches} from ${JSON.stringify(executionUrl)};\nconst streams=createRawResponseStreams(${JSON.stringify(directory)});\nlet announced=false;\nfunction retain(channel,bytes){streams[channel](bytes);const raw=streams.read();if(!announced&&raw.stdout.length===${stdout.length}&&raw.stderr.length===${stderr.length}){announced=true;process.stdout.write('RAW_READY\\n');}}\nconst execution=await executeBatches([{target:'unit',names:['unchanged_fixture'],weight:1}],new Map([['unit',${JSON.stringify(emitter)}]]),{cwd:${JSON.stringify(directory)},threads:1,concurrency:1,nocapture:true,onStdout:bytes=>retain('onStdout',bytes),onStderr:bytes=>retain('onStderr',bytes)});\nfs.writeFileSync(${JSON.stringify(join(directory,'report.json'))},JSON.stringify(execution));\n`);
  controller=spawn(process.execPath,[launcher],{stdio:['ignore','pipe','pipe']});
  const closed=once(controller,'close');let errors='';controller.stderr.on('data',bytes=>errors+=bytes.toString());
  await new Promise((resolve,reject)=>{
   let observed='';const timer=setTimeout(()=>reject(new Error('raw readiness timed out: '+errors)),5000);
   controller.stdout.on('data',bytes=>{observed+=bytes.toString();if(observed.includes('RAW_READY\n')){clearTimeout(timer);resolve();}});
   controller.once('error',error=>{clearTimeout(timer);reject(error);});
   controller.once('exit',()=>{clearTimeout(timer);reject(new Error('controller exited before interruption: '+errors));});
  });
  emitterPid=Number(readFileSync(join(directory,'emitter.pid'),'utf8'));
  assert.deepEqual(readFileSync(join(directory,'stdout.bin')),stdout);
  assert.deepEqual(readFileSync(join(directory,'stderr.bin')),stderr);
  assert.equal(controller.kill('SIGTERM'),true);
  const [exitCode,signal]=await closed;
  assert.ok(signal!==null||exitCode!==0,'interruption must not be observed as successful completion');
  assert.deepEqual(readFileSync(join(directory,'stdout.bin')),stdout);
  assert.deepEqual(readFileSync(join(directory,'stderr.bin')),stderr);
  assert.equal(existsSync(join(directory,'report.json')),false,'partial raw bytes do not certify a completed original test');
 } finally {
  if(controller&&controller.exitCode===null&&controller.signalCode===null)controller.kill('SIGKILL');
  if(emitterPid)try{process.kill(emitterPid,'SIGKILL');}catch{}
  rmSync(directory,{recursive:true,force:true});
 }
});
