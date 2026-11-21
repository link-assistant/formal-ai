// Finite transport probes use tiny JavaScript executables, never a local Rust build.
import test from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync,spawnSync} from 'node:child_process';
import {gunzipSync} from 'node:zlib';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,unlinkSync,chmodSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {CONTRACT_TARGETS,contractPhysicalIdentity,selectContractExecutables,sealContractArtifact,verifyContractArtifact,contractInvocation,runContractArtifact} from '../../scripts/native-contract-artifact.mjs';
function workspace(t) {
  const root=mkdtempSync(join(tmpdir(),'native-contract-transport-'));t.after(()=>rmSync(root,{recursive:true,force:true}));
  const git=args=>execFileSync('git',args,{cwd:root,encoding:'utf8',stdio:['ignore','pipe','pipe']});
  git(['init','--quiet']);git(['config','user.name','Test']);git(['config','user.email','test@example.invalid']);
  for(const [file,contents] of [['rust/Cargo.toml','[package]\nname="fixture"\n'],['rust/Cargo.lock','# finite fixture\n'],['rust/tests/unit/coding.rs','// fixture\n'],['dev/log/history.txt','excluded historical record\n']]){
    mkdirSync(resolve(root,file,'..'),{recursive:true});writeFileSync(join(root,file),contents);
  }
  git(['add','.']);git(['commit','--quiet','-m','finite fixture']);return {root,git};
}
function artifact(t) {
  const {root}=workspace(t),directory=join(root,'transport'),sources=join(root,'compiled');mkdirSync(sources);
  const executables={};
  for(const name of [...CONTRACT_TARGETS,'formal-ai']) {
    const file=join(sources,name);writeFileSync(file,'#!/usr/bin/env node\nif(process.argv.includes("fail::case"))process.exit(17);\nconsole.log(JSON.stringify(process.argv.slice(2)));\n');chmodSync(file,0o755);executables[name]=file;
  }
  const identity={commit:'a'.repeat(40),run:'123',attempt:'1',physicalInputs:contractPhysicalIdentity(root)};
  sealContractArtifact(directory,identity,executables);return {root,directory,identity,executables};
}
const invocation=(target,tail,filter=[])=>['test','--manifest-path','rust/Cargo.toml','--locked','--no-fail-fast','--test',target,...filter,'--',...tail];
test('contract sparse identity checks actual bytes and explicitly records excluded history',t=>{
  const {root,git}=workspace(t),before=contractPhysicalIdentity(root);assert.equal(before.files,4);assert.equal(before.excluded,0);
  git(['update-index','--skip-worktree','dev/log/history.txt']);unlinkSync(join(root,'dev/log/history.txt'));
  const sparse=contractPhysicalIdentity(root);assert.equal(sparse.files,3);assert.equal(sparse.excluded,1);assert.notEqual(sparse.sha256,before.sha256);
  writeFileSync(join(root,'rust/tests/unit/coding.rs'),'changed\n');assert.throws(()=>contractPhysicalIdentity(root),/uncommitted physical input/);
  git(['checkout','--','rust/tests/unit/coding.rs']);unlinkSync(join(root,'rust/tests/unit/coding.rs'));
  assert.throws(()=>contractPhysicalIdentity(root),/missing materialized input/);
  git(['update-index','--skip-worktree','rust/tests/unit/coding.rs']);assert.throws(()=>contractPhysicalIdentity(root),/native source cannot be excluded/);
});
test('untracked native tests cannot enter a compiled contract identity',t=>{
  const {root}=workspace(t);writeFileSync(join(root,'rust/tests/unit/injected.rs'),'// unexpected\n');
  assert.throws(()=>contractPhysicalIdentity(root),/untracked native contract input/);
});
test('compiler JSON selects all three actual targets and the companion binary without guessing hashes',t=>{
  const {root}=workspace(t);
  const records=[...CONTRACT_TARGETS.map(name=>({reason:'compiler-artifact',target:{name,kind:['test']},profile:{test:true},executable:join(root,'rust/target/debug/deps',name+'-hash')})),
    {reason:'compiler-artifact',target:{name:'formal-ai',kind:['bin']},profile:{test:false},executable:join(root,'rust/target/debug/formal-ai')}];
  const text=rows=>rows.map(row=>JSON.stringify(row)).join('\n');
  assert.deepEqual(Object.keys(selectContractExecutables(text(records),root)).sort(),[...CONTRACT_TARGETS,'formal-ai'].sort());
  assert.throws(()=>selectContractExecutables(text(records.slice(1)),root));
  assert.throws(()=>selectContractExecutables(text([...records,records[0]]),root),/duplicate/);
  assert.throws(()=>selectContractExecutables(text(records.map((row,i)=>i===0?{...row,executable:'/tmp/foreign-executable'}:row)),root),/outside debug target/);
});
test('receipt requires the same source, run, attempt and every executable byte',t=>{
  const {directory,identity}=artifact(t);verifyContractArtifact(directory,identity);
  for(const field of ['commit','run','attempt'])assert.throws(()=>verifyContractArtifact(directory,{...identity,[field]:'different'}));
  writeFileSync(join(directory,'coding_contracts'),'changed');assert.throws(()=>verifyContractArtifact(directory,identity),/artifact bytes changed/);
});
test('companion binary tampering fails before any test executable runs',t=>{
  const {root,directory,identity}=artifact(t);writeFileSync(join(directory,'formal-ai'),'changed');
  assert.throws(()=>runContractArtifact(directory,identity,invocation('coding_contracts',['--exact','pass::case','--test-threads=1','--nocapture']),root),/artifact bytes changed/);
});
test('adapter preserves the selected libtest arguments and refuses undeclared Cargo invocations',()=>{
  for(const target of CONTRACT_TARGETS) {
    assert.deepEqual(contractInvocation(invocation(target,['--list','--format','terse'])),{target,args:['--list','--format','terse']});
    assert.deepEqual(contractInvocation(invocation(target,['--exact','one::case','two::case','--test-threads=1','--nocapture'])).args,['--exact','one::case','two::case','--test-threads=1','--nocapture']);
  }
  for(const prefix of ['benchmarks::','issue_1133_hive_mind_three_runs::'])assert.equal(contractInvocation(invocation('reasoning_contracts',['--test-threads=1','--nocapture'],[prefix])).args[0],prefix);
  assert.throws(()=>contractInvocation(invocation('unit',['--list','--format','terse'])),/unsupported native/);
  assert.throws(()=>contractInvocation(invocation('coding_contracts',['--ignored'])),/unsupported libtest/);
  assert.throws(()=>contractInvocation(invocation('coding_contracts',['--exact','one::case','one::case','--test-threads=1','--nocapture'])),/invalid exact/);
  assert.throws(()=>contractInvocation(invocation('coding_contracts',['--list','--format','terse']).filter(arg=>arg!=='--locked')));
});
test('verified finite executable failures propagate and the bound companion is restored',t=>{
  const {root,directory,identity}=artifact(t);
  const status=runContractArtifact(directory,identity,invocation('coding_contracts',['--exact','fail::case','--test-threads=1','--nocapture']),root);
  assert.equal(status,17);assert.deepEqual(readFileSync(join(root,'rust/target/debug/formal-ai')),readFileSync(join(directory,'formal-ai')));
});

test('all nine real runner groups reuse bound finite executables and invoke no Cargo', {timeout:30000}, t=>{
  const {root,git}=workspace(t),directory=join(root,'transport'),sources=join(root,'compiled'),bin=join(root,'bin');mkdirSync(sources);mkdirSync(bin);
  const cases={coding_contracts:['coding::one','coding::two','coding::three','coding::four'],reasoning_contracts:[
    'reasoning::one','reasoning::two','reasoning::three','reasoning::four','benchmarks::one','benchmarks::two',
    ...Array.from({length:6},(_,i)=>'issue_1133_hive_mind_three_runs::case_'+i)],interface_contracts:['interface::one','interface::two']};
  writeFileSync(join(bin,'rustc'),'#!/usr/bin/env node\nconsole.log("rustc finite-transport-fixture\\nhost: fixture-host");\n',{mode:0o755});
  writeFileSync(join(bin,'cargo'),'#!/usr/bin/env node\nconsole.error("finite transport probe: Cargo must not run");process.exit(99);\n',{mode:0o755});
  const calls=join(root,'calls.jsonl'),environment={...process.env,PATH:bin+':'+process.env.PATH,FORMAL_AI_NATIVE_CONTRACT_ARTIFACT:directory,
    NATIVE_CONTRACT_CASES:JSON.stringify(cases),NATIVE_CONTRACT_CALLS:calls};delete environment.CARGO_TARGET_DIR;
  const executables={};
  const program=`#!/usr/bin/env node
import {appendFileSync} from 'node:fs';import {basename} from 'node:path';
const target=basename(process.argv[1]),args=process.argv.slice(2);
appendFileSync(process.env.NATIVE_CONTRACT_CALLS,JSON.stringify({target,args})+'\\n');
if(args.includes('--list')){
 let names=JSON.parse(process.env.NATIVE_CONTRACT_CASES)[target];
 if(args[0]&&!args[0].startsWith('--'))names=names.filter(name=>name.includes(args[0]));
 for(let i=0;i<args.length;i++)if(args[i]==='--skip')names=names.filter(name=>!name.includes(args[i+1]));
 process.stdout.write(names.map(name=>name+': test\\n').join(''));
}
`;
  for(const name of [...CONTRACT_TARGETS,'formal-ai']){const file=join(sources,name);writeFileSync(file,program,{mode:0o755});executables[name]=file;}
  const module=new URL('../../scripts/native-contract-artifact.mjs',import.meta.url).href;
  const identity=JSON.parse(execFileSync(process.execPath,['--input-type=module','-e',`import {contractIdentity} from ${JSON.stringify(module)};process.stdout.write(JSON.stringify(contractIdentity(process.cwd())));`],{cwd:root,env:environment,encoding:'utf8'}));
  sealContractArtifact(directory,identity,executables);
  const runner=resolve('scripts/ci-native-regressions.sh');
  const before=join(root,'original-runner.sh');writeFileSync(before,gunzipSync(readFileSync(new URL('./fixtures/native-contract-artifact/original-runner.sh.gz',import.meta.url))));
  const baseline=spawnSync('bash',[before,'coding-1'],{cwd:root,env:environment,encoding:'utf8',timeout:5000});
  assert.equal(baseline.status,99,baseline.stderr);assert.match(baseline.stderr,/Cargo must not run/u);
  for(const group of ['coding-1','coding-2','reasoning-1','reasoning-2','benchmarks','interfaces','hive-1','hive-2','hive-3']){
    const result=spawnSync('bash',[runner,group],{cwd:root,env:environment,encoding:'utf8',timeout:5000,maxBuffer:65536});
    assert.equal(result.status,0,group+': '+result.stdout+result.stderr);
  }
  const rows=readFileSync(calls,'utf8').trimEnd().split('\n').map(JSON.parse),actual=[];
  for(const row of rows.filter(row=>!row.args.includes('--list'))){
    if(row.args[0]==='--exact')actual.push(...row.args.slice(1,-2).map(name=>row.target+'\t'+name));
    else actual.push(...cases[row.target].filter(name=>row.args[0].startsWith('--')||name.includes(row.args[0])).map(name=>row.target+'\t'+name));
  }
  const expected=Object.entries(cases).flatMap(([target,names])=>names.map(name=>target+'\t'+name));
  assert.deepEqual(actual.toSorted(),expected.toSorted());assert.equal(new Set(actual).size,actual.length);
  writeFileSync(join(directory,'coding_contracts'),'tampered');
  const rejected=spawnSync('bash',[runner,'coding-1'],{cwd:root,env:environment,encoding:'utf8',timeout:5000});
  assert.notEqual(rejected.status,0);assert.match(rejected.stderr,/artifact bytes changed/u);
  assert.equal(readFileSync(calls,'utf8').trimEnd().split('\n').length,rows.length,'invalid receipt must not list or execute tests');
});
