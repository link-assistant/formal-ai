import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync, realpathSync, readdirSync, mkdirSync }  from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { spawnWithIsolatedTestCache } from '../../../scripts/lib/test-cache-isolation.mjs';
import { executeBatches } from '../../../scripts/lib/coverage-execution.mjs';

function fixture() {
  const directory = mkdtempSync(join(realpathSync(tmpdir()), 'cache-isolation-contract-'));
  const program = join(directory, 'fake.mjs');
  writeFileSync(program, [
    "import fs from 'node:fs';",
    "const cache = process.env.FORMAL_AI_SOURCE_CACHE_DIR;",
    "fs.writeFileSync(process.env.OBSERVATION, JSON.stringify({cache, secondary:process.env.FORMAL_AI_CACHE_DIR,",
    "args:process.argv.slice(2),cwd:process.cwd(),profile:process.env.LLVM_PROFILE_FILE,pid:process.pid}));",
    "fs.writeFileSync(cache+'/body', 'private runtime source');",
    "if(process.env.MODE==='wait'){console.log('ready');setInterval(()=>{},1000);}",
    "else if(process.env.MODE==='signal'){process.kill(process.pid,'SIGTERM');}",
    "else if(process.env.MODE==='coverage'){",
    "console.log('test selected ... ok');console.log('test result: ok. 1 passed; 0 failed; 0 ignored;');}",
    "else {process.stdout.write('output');process.stderr.write('diagnostic');process.exitCode=7;}",
  ].join('\n'));
  return { directory, program, observation:join(directory,'observed.json'),
    close:()=>rmSync(directory,{recursive:true,force:true}) };
}

function environment(item, mode) {
  return { ...process.env, OBSERVATION:item.observation, MODE:mode,
    FORMAL_AI_CACHE_DIR:'forbidden-existing-cache', FORMAL_AI_SOURCE_CACHE_DIR:'forbidden-source-cache',
    LLVM_PROFILE_FILE:join(item.directory,'profile-%p') };
}

async function observedClose(child) {
  let stdout = '', stderr = '';
  child.stdout?.on('data', chunk => { stdout += chunk; });
  child.stderr?.on('data', chunk => { stderr += chunk; });
  const [code, signal] = await once(child, 'close');
  return {code,signal,stdout,stderr};
}

test('isolated child preserves argv/cwd/streams/status/profile and cleans its owned cache', async () => {
  const item = fixture();
  try {
    const env = environment(item, 'exit');
    const child = spawnWithIsolatedTestCache(process.execPath, [item.program,'space argument','--exact'],
      {cwd:item.directory,env,stdio:['ignore','pipe','pipe']});
    const result = await observedClose(child);
    const receipt = JSON.parse(readFileSync(item.observation));
    assert.deepEqual(result,{code:7,signal:null,stdout:'output',stderr:'diagnostic'});
    assert.deepEqual(receipt.args,['space argument','--exact']);
    assert.equal(receipt.cwd,item.directory);
    assert.equal(receipt.profile,env.LLVM_PROFILE_FILE);
    assert.equal(receipt.pid,child.pid);
    assert.equal(receipt.cache,receipt.secondary);
    assert.notEqual(receipt.cache,env.FORMAL_AI_SOURCE_CACHE_DIR);
    assert.equal(existsSync(receipt.cache),false);
    assert.equal(env.FORMAL_AI_CACHE_DIR,'forbidden-existing-cache');
  } finally { item.close(); }
});

test('coverage execution uses the same isolated environment and real completion oracle', async () => {
  const item = fixture();
  try {
    // A Node script shim is explicitly synthetic; no native binary is executed.
    const shim = join(item.directory,'executable');
    writeFileSync(shim,'#!'+process.execPath+'\n'+readFileSync(item.program,'utf8'),{mode:0o755});
    const result = await executeBatches([{target:'unit',names:['selected'],seconds:1}],
      new Map([['unit',shim]]),{env:environment(item,'coverage'),cwd:item.directory});
    const receipt = JSON.parse(readFileSync(item.observation));
    assert.equal(result.succeeded,true);
    assert.equal(result.completed,1);
    assert.equal(result.batches[0].processId,receipt.pid);
    assert.equal(receipt.args.at(-1),'selected');
    assert.equal(receipt.cache,receipt.secondary);
    assert.equal(existsSync(receipt.cache),false);
  } finally { item.close(); }
});

test('CLI forwards termination to its child, cleans cache, and retains signal disposition', async () => {
  const item = fixture();
  try {
    const wrapper = resolve('scripts/lib/test-cache-isolation.mjs');
    const child = spawn(process.execPath,[wrapper,process.execPath,item.program],
      {env:environment(item,'wait'),stdio:['ignore','pipe','pipe']});
    await once(child.stdout,'data');
    const closed = once(child,'close');
    child.kill('SIGTERM');
    const [code,signal] = await closed;
    const receipt = JSON.parse(readFileSync(item.observation));
    assert.equal(code,null);
    assert.equal(signal,'SIGTERM');
    assert.equal(existsSync(receipt.cache),false);
  } finally { item.close(); }
});

test('child-originated signal is preserved by CLI and owned cache is removed', async () => {
  const item = fixture();
  try {
    const wrapper = resolve('scripts/lib/test-cache-isolation.mjs');
    const child = spawn(process.execPath,[wrapper,process.execPath,item.program],
      {env:environment(item,'signal'),stdio:['ignore','pipe','pipe']});
    const result = await observedClose(child);
    const receipt = JSON.parse(readFileSync(item.observation));
    assert.equal(result.signal,'SIGTERM');
    assert.equal(result.code,null);
    assert.equal(existsSync(receipt.cache),false);
  } finally { item.close(); }
});

test('full-suite routes both execution branches through isolation and leaves listing untouched', () => {
  const source = readFileSync('scripts/run-prebuilt-tests.sh','utf8');
  assert.equal(source.split('node scripts/lib/test-cache-isolation.mjs "dist/tests/$target"').length-1,2);
  assert.match(source,/"dist\/tests\/\$target" --list --format terse/);
});

test('spawn failure still cleans the allocated cache and reports the actual error', async () => {
  const before = new Set(readdirSync(realpathSync(tmpdir())).filter(name => name.startsWith('formal-ai-test-cache-')));
  const child = spawnWithIsolatedTestCache('/nonexistent/formal-ai-fake-harness',[],{stdio:'pipe'});
  let error;
  child.on('error',observed => { error = observed; });
  await new Promise(resolve => child.once('close',resolve));
  assert.equal(error.code,'ENOENT');
  const after = readdirSync(realpathSync(tmpdir())).filter(name => name.startsWith('formal-ai-test-cache-'));
  assert.deepEqual(after.filter(name => !before.has(name)),[]);
});

test('actual full-suite shell uses isolated fake executables for every target', async () => {
  const item = fixture();
  try {
    mkdirSync(join(item.directory,'scripts/lib'),{recursive:true});
    mkdirSync(join(item.directory,'dist/tests'),{recursive:true});
    writeFileSync(join(item.directory,'scripts/install-rust-script.sh'),'#!/bin/sh\nexit 0\n');
    writeFileSync(join(item.directory,'scripts/lib/test-cache-isolation.mjs'),
      readFileSync('scripts/lib/test-cache-isolation.mjs'));
    writeFileSync(join(item.directory,'run.sh'),readFileSync('scripts/run-prebuilt-tests.sh'));
    for(const target of ['unit','integration','source']) {
      writeFileSync(join(item.directory,'dist/tests',target),[
        '#!'+process.execPath,
        "import fs from 'node:fs';",
        "const cache=process.env.FORMAL_AI_SOURCE_CACHE_DIR;",
        "fs.appendFileSync(process.env.OBSERVATION,JSON.stringify({cache,secondary:process.env.FORMAL_AI_CACHE_DIR,args:process.argv.slice(2)})+'\\n');",
        "fs.writeFileSync(cache+'/runtime-body','fake runtime source');",
      ].join('\n'),{mode:0o755});
    }
    const child = spawn('bash',['run.sh'],{cwd:item.directory,
      env:{...environment(item,'shell'),SHARD_TOTAL:'1',TEST_SUITE:'full'},stdio:['ignore','pipe','pipe']});
    const result = await observedClose(child);
    assert.equal(result.code,0,result.stderr);
    const records=readFileSync(item.observation,'utf8').trim().split('\n').map(line=>JSON.parse(line));
    assert.equal(records.length,3);
    assert.equal(new Set(records.map(record=>record.cache)).size,3);
    for(const record of records) {
      assert.equal(record.cache,record.secondary);
      assert.equal(existsSync(record.cache),false);
      assert.ok(record.args.includes('issue_1138_no_silent_unknown'));
    }
  } finally { item.close(); }
});
