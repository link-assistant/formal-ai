// Exercise the production shell boundary without resolving or building Rust.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,mkdtempSync,rmSync,copyFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';

const root=new URL('../../../',import.meta.url);
const config=readFileSync(new URL('.cargo/audit.toml',root),'utf8');
const historicalConfig=readFileSync(new URL('experiments/issue-1580/fixtures/audit-before-web-capture-upgrade.toml',root),'utf8');

function withAudit(run){
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-audit-'));
  try{
    for(const folder of ['bin','.cargo','rust','scripts','extra'])mkdirSync(join(directory,folder));
    copyFileSync(new URL('scripts/check-rust-dependencies.sh',root),join(directory,'scripts/check-rust-dependencies.sh'));
    writeFileSync(join(directory,'rust/Cargo.lock'),'# protocol fixture; no Rust dependency resolution\n');
    writeFileSync(join(directory,'extra/Cargo.lock'),'# independent committed-lock fixture\n');
    const trace=join(directory,'calls.jsonl');
    writeFileSync(join(directory,'bin/cargo'),`#!${process.execPath}\n`+String.raw`
const fs=require('node:fs');
const args=process.argv.slice(2);
fs.appendFileSync(process.env.AUDIT_TRACE,JSON.stringify(args)+'\n');
if(args[0]==='tree'){
  const spec=args[args.indexOf('--invert')+1];
  if(spec===process.env.ABSENT_CRATE)process.exit(0);
  if(!['fxhash@0.2.1','chrono@0.4.45'].includes(spec))process.exit(101);
  process.stdout.write(spec+'\n└── fixture dependency path\n');
}else if(args[0]==='audit')process.exit(Number(process.env.AUDIT_FAILURE??0));
else process.exit(99);
`,{mode:0o755});
    writeFileSync(join(directory,'bin/git'),`#!${process.execPath}\n`+String.raw`
if(process.argv[2]!=='ls-files')process.exit(99);
process.stdout.write(process.env.GIT_INVENTORY_OUTPUT??'rust/Cargo.lock\n');
process.exit(Number(process.env.GIT_INVENTORY_STATUS??0));
`,{mode:0o755});
    const invoke=({text=config,...env}={})=>{
      writeFileSync(join(directory,'.cargo/audit.toml'),text);
      writeFileSync(trace,'');
      const result=spawnSync('bash',['scripts/check-rust-dependencies.sh'],{
        cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:65536,
        env:{...process.env,PATH:join(directory,'bin')+':'+process.env.PATH,AUDIT_TRACE:trace,...env},
      });
      const calls=readFileSync(trace,'utf8').split('\n').filter(Boolean).map(line=>JSON.parse(line));
      return {...result,calls};
    };
    run(invoke);
  }finally{rmSync(directory,{recursive:true,force:true});}
}

test('blocked advisory entries require reachable exact versions and retain the strict audit', {timeout:10000},()=>{
  withAudit(invoke=>{
    const result=invoke({text:historicalConfig});
    assert.equal(result.status,0,result.stderr+result.stdout);
    assert.deepEqual(result.calls.filter(args=>args[0]==='tree').map(args=>args.at(-1)),['fxhash@0.2.1','chrono@0.4.45']);
    assert.deepEqual(result.calls.filter(args=>args[0]==='audit'),[['audit','--file','rust/Cargo.lock','--deny','warnings']]);
  });
});

test('removing the blocked Chrono version invalidates the exception before audit', {timeout:10000},()=>{
  withAudit(invoke=>{
    const result=invoke({ABSENT_CRATE:'chrono@0.4.45'});
    assert.equal(result.status,1,result.stderr);
    assert.match(result.stdout,/chrono@0.4.45.*no longer reaches the build graph/u);
    assert.equal(result.calls.some(args=>args[0]==='audit'),false);
  });
});

test('a missing advisory proof is rejected and unrelated audit failures propagate', {timeout:10000},()=>{
  withAudit(invoke=>{
    const withoutProof=config.replace(/^# RUSTSEC-2026-0543 blocked-upstream.*\n/mu,'');
    const missing=invoke({text:withoutProof});
    assert.equal(missing.status,1,missing.stderr);
    assert.match(missing.stdout,/ignores RUSTSEC-2026-0543 without a proof line/u);
    assert.equal(missing.calls.some(args=>args[0]==='audit'),false);
    const unrelated=invoke({AUDIT_FAILURE:'1'});
    assert.equal(unrelated.status,1,unrelated.stderr);
    assert.ok(unrelated.calls.some(args=>args[0]==='audit'&&args.includes('warnings')));
  });
});

for(const [name,output] of [['empty',''],['partial','extra/Cargo.lock\n']]){
  test(`failed ${name} Git inventory stops before any Cargo command`, {timeout:10000},()=>{
    withAudit(invoke=>{
      const result=invoke({GIT_INVENTORY_OUTPUT:output,GIT_INVENTORY_STATUS:'29'});
      assert.equal(result.status,29,result.stderr+result.stdout);
      assert.match(result.stderr+result.stdout,/Unable to enumerate.*lockfiles.*29/u);
      assert.deepEqual(result.calls,[]);
    });
  });
}

test('complete inventory audits every independent lock with its existing policy', {timeout:10000},()=>{
  withAudit(invoke=>{
    const result=invoke({GIT_INVENTORY_OUTPUT:'rust/Cargo.lock\nextra/Cargo.lock\n'});
    assert.equal(result.status,0,result.stderr+result.stdout);
    assert.deepEqual(result.calls.filter(args=>args[0]==='audit'),[
      ['audit','--file','rust/Cargo.lock','--deny','warnings'],
      ['audit','--file','extra/Cargo.lock','--deny','unsound','--deny','yanked'],
    ]);
  });
});

test('a committed lock missing from checkout remains a failing audit', {timeout:10000},()=>{
  withAudit(invoke=>{
    const result=invoke({GIT_INVENTORY_OUTPUT:'rust/Cargo.lock\nmissing/Cargo.lock\n'});
    assert.equal(result.status,1,result.stderr+result.stdout);
    assert.match(result.stdout,/missing\/Cargo.lock is committed but not checked out/u);
    assert.equal(result.calls.filter(args=>args[0]==='audit').length,1);
  });
});

test('the published web-capture upgrade removes fxhash and its exception while preserving strict Chrono proof', {timeout:10000},()=>{
  const lock=readFileSync(new URL('rust/Cargo.lock',root),'utf8');
  assert.equal(/^name = "fxhash"$/mu.test(lock),false,'fxhash must leave the production lockfile');
  assert.doesNotMatch(config,/RUSTSEC-2025-0057/u);
  withAudit(invoke=>{
    const result=invoke();
    assert.equal(result.status,0,result.stderr+result.stdout);
    assert.deepEqual(result.calls.filter(args=>args[0]==='tree').map(args=>args.at(-1)),['chrono@0.4.45']);
    assert.deepEqual(result.calls.filter(args=>args[0]==='audit'),[['audit','--file','rust/Cargo.lock','--deny','warnings']]);
  });
});
