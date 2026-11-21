#!/usr/bin/env node
// Reuse only this workflow run's three focused native contract executables.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {chmodSync,closeSync,copyFileSync,existsSync,lstatSync,mkdirSync,openSync,readFileSync,readSync,writeFileSync} from 'node:fs';
import {join,resolve,relative,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {nativeInputIdentity} from './native-release-source.mjs';

export const CONTRACT_TARGETS=Object.freeze(['coding_contracts','reasoning_contracts','interface_contracts']);
const RECEIPT='native-contract-receipt.json';
const RECIPE=Object.freeze({profile:'test',features:'default',targets:CONTRACT_TARGETS});
const git=(cwd,args)=>execFileSync('git',args,{cwd,encoding:'utf8',maxBuffer:32*1024*1024}).trim();
function digestFile(file,gitObject=false) {
  const before=lstatSync(file);assert(before.isFile()&&!before.isSymbolicLink(),'regular physical file required: '+file);
  const hash=createHash(gitObject?'sha1':'sha256');
  if(gitObject)hash.update('blob '+before.size+'\0');
  const fd=openSync(file,'r'),buffer=Buffer.allocUnsafe(1024*1024);let total=0;
  try{for(let count;(count=readSync(fd,buffer,0,buffer.length,null))>0;){total+=count;assert(total<=before.size);hash.update(buffer.subarray(0,count));}}
  finally{closeSync(fd);}
  const after=lstatSync(file);
  for(const key of ['dev','ino','size','mtimeMs','ctimeMs'])assert.equal(after[key],before[key],'file changed during hashing');
  assert.equal(total,before.size);return {bytes:total,...(gitObject?{object:hash.digest('hex')}:{sha256:hash.digest('hex')})};
}

/** Bind the checked-out sparse input set; excluded history is not certified as physical source. */
export function contractPhysicalIdentity(cwd) {
  const rows=git(cwd,['ls-tree','-r','-z','HEAD']).split('\0').filter(Boolean);
  const flags=new Map(execFileSync('git',['ls-files','-t','-z'],{cwd,encoding:'utf8',maxBuffer:32*1024*1024}).split('\0').filter(Boolean).map(line=>[line.slice(2),line[0]]));
  const digest=createHash('sha256');let files=0,excluded=0;
  for(const row of rows) {
    const match=/^(\d+) blob ([a-f0-9]{40})\t([\s\S]+)$/u.exec(row);assert(match,'committed blob required');
    const [,mode,object,name]=match;assert.notEqual(mode,'120000','symlink source is unsupported');
    assert(flags.has(name),'tracked input missing from index: '+name);
    assert.doesNotMatch(name,/[\r\n]/u);
    if(!existsSync(join(cwd,name))) {
      assert.equal(flags.get(name),'S','missing materialized input: '+name);
      assert(!name.startsWith('rust/'),'native source cannot be excluded: '+name);
      digest.update('excluded\0'+name+'\0'+object+'\0');excluded++;continue;
    }
    const checked=digestFile(join(cwd,name),true);assert.equal(checked.object,object,'uncommitted physical input: '+name);
    digest.update('physical\0'+name+'\0'+object+'\0');files++;
  }
  const untracked=git(cwd,['ls-files','--others','--exclude-standard','--','rust/tests']);
  assert.equal(untracked,'','untracked native contract input');
  return {sha256:digest.digest('hex'),files,excluded};
}

export function contractIdentity(cwd,environment=process.env) {
  const compiler=execFileSync('rustc',['-vV'],{cwd,encoding:'utf8'}).trim();
  assert.match(compiler,/^host: \S+$/mu,'actual compiler host required');
  assert(!environment.CARGO_TARGET_DIR,'focused artifacts require the default target directory');
  const overrides=Object.fromEntries(Object.entries(environment).filter(([key])=>/^CARGO_PROFILE_[A-Z0-9_]+$/u.test(key)).sort(([a],[b])=>a.localeCompare(b,'en')));
  return {commit:git(cwd,['rev-parse','HEAD']),tree:git(cwd,['rev-parse','HEAD^{tree}']),workspace:resolve(cwd),
    run:environment.GITHUB_RUN_ID??'',attempt:environment.GITHUB_RUN_ATTEMPT??'',platform:process.platform,architecture:process.arch,
    compiler,recipe:RECIPE,rustFlags:environment.RUSTFLAGS??'',encodedRustFlags:environment.CARGO_ENCODED_RUSTFLAGS??'',
    target:environment.CARGO_BUILD_TARGET??'',profileOverrides:overrides,nativeInputs:nativeInputIdentity(cwd),
    physicalInputs:contractPhysicalIdentity(cwd)};
}

export function selectContractExecutables(messages,cwd) {
  const selected={};
  for(const line of messages.trim().split(/\r?\n/u)) {
    if(!line)continue;const row=JSON.parse(line);
    if(row.reason==='compiler-message'&&row.message?.rendered)process.stderr.write(row.message.rendered);
    if(row.reason!=='compiler-artifact'||!row.executable)continue;
    const name=row.target?.name;
    const target=CONTRACT_TARGETS.includes(name)&&row.target.kind.includes('test')?name:
      name==='formal-ai'&&row.target.kind.includes('bin')&&!row.profile.test?'formal-ai':null;
    if(target===null)continue;
    const file=resolve(row.executable),inside=relative(resolve(cwd,'rust/target/debug'),file);
    assert(inside&&!inside.startsWith('..')&&!isAbsolute(inside),'compiler executable outside debug target');
    assert(!selected[target],'duplicate compiler executable: '+target);selected[target]=file;
  }
  assert.deepEqual(Object.keys(selected).sort(),[...CONTRACT_TARGETS,'formal-ai'].sort());
  return selected;
}

export function sealContractArtifact(directory,identity,executables) {
  mkdirSync(directory,{recursive:true});const files={};
  assert.deepEqual(Object.keys(executables).sort(),[...CONTRACT_TARGETS,'formal-ai'].sort());
  for(const name of [...CONTRACT_TARGETS,'formal-ai']) {
    const destination=join(directory,name);assert(!existsSync(destination),'artifact already exists');
    const before=digestFile(executables[name]);copyFileSync(executables[name],destination);
    assert.deepEqual(digestFile(destination),before);files[name]=before;
  }
  const receipt={schema:'native-contract-artifact/v1',identity,files};
  writeFileSync(join(directory,RECEIPT),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});return receipt;
}

export function verifyContractArtifact(directory,identity) {
  const receipt=JSON.parse(readFileSync(join(directory,RECEIPT),'utf8'));
  assert.equal(receipt.schema,'native-contract-artifact/v1');assert.deepEqual(receipt.identity,identity);
  assert.deepEqual(Object.keys(receipt.files).sort(),[...CONTRACT_TARGETS,'formal-ai'].sort());
  for(const name of [...CONTRACT_TARGETS,'formal-ai'])assert.deepEqual(digestFile(join(directory,name)),receipt.files[name],'artifact bytes changed: '+name);
  return receipt;
}

/** Accept only the existing focused runner's Cargo invocations. */
export function contractInvocation(args) {
  assert.deepEqual(args.slice(0,6),['test','--manifest-path','rust/Cargo.toml','--locked','--no-fail-fast','--test']);
  const target=args[6];assert(CONTRACT_TARGETS.includes(target),'unsupported native contract target');
  const separator=args.indexOf('--',7);assert(separator===7||separator===8,'unsupported cargo arguments');
  const filter=separator===8?args[7]:null,tail=args.slice(separator+1);
  if(filter!==null)assert(target==='reasoning_contracts'&&['benchmarks::','issue_1133_hive_mind_three_runs::'].includes(filter),'unsupported family filter');
  const standard=['--test-threads=1','--nocapture'];
  const list=['--list','--format','terse'];
  const skips=['--skip','benchmarks::','--skip','issue_1133_hive_mind_three_runs::'];
  const same=(left,right)=>JSON.stringify(left)===JSON.stringify(right);
  const exact=tail[0]==='--exact'&&tail.length>=4&&same(tail.slice(-2),standard)&&filter===null;
  if(exact){const names=tail.slice(1,-2);assert(names.length&&new Set(names).size===names.length&&names.every(name=>/^([A-Za-z0-9_]+::)+[A-Za-z0-9_]+$/u.test(name)),'invalid exact contract cases');}
  else assert(same(tail,list)||same(tail,standard)||(target==='reasoning_contracts'&&filter===null&&(same(tail,[...skips,...list])||same(tail,[...skips,...standard]))),'unsupported libtest arguments');
  return {target,args:[...(filter?[filter]:[]),...tail]};
}

export function runContractArtifact(directory,identity,cargoArguments,cwd) {
  const invocation=contractInvocation(cargoArguments);verifyContractArtifact(directory,identity);
  const binary=resolve(cwd,'rust/target/debug/formal-ai');mkdirSync(resolve(cwd,'rust/target/debug'),{recursive:true});
  copyFileSync(join(directory,'formal-ai'),binary);chmodSync(binary,0o755);
  const executable=resolve(directory,invocation.target);chmodSync(executable,0o755);
  const result=spawnSync(executable,invocation.args,{cwd,stdio:'inherit'});
  if(result.error)throw result.error;assert.equal(result.signal,null,'native contract process was signaled');
  assert(Number.isInteger(result.status));return result.status;
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
  const [mode,directory,...args]=process.argv.slice(2),cwd=process.cwd();assert(directory);
  const identity=contractIdentity(cwd);
  if(mode==='build') {
    assert.equal(process.env.GITHUB_ACTIONS,'true','native contract compilation is hosted-only');
    assert.match(identity.run,/^\d+$/u,'actual workflow run required');
    assert.match(identity.attempt,/^\d+$/u,'actual workflow attempt required');
    assert.equal(args.length,0);mkdirSync(directory,{recursive:true});
    const built=spawnSync('cargo',['test','--manifest-path','rust/Cargo.toml','--locked','--no-run','--message-format=json',...CONTRACT_TARGETS.flatMap(target=>['--test',target])],{cwd,encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','inherit']});
    writeFileSync(join(directory,'cargo-messages.jsonl'),built.stdout??'');
    if(built.error)throw built.error;
    if(built.status!==0){process.stderr.write(built.stdout??'');process.exit(built.status??1);}
    sealContractArtifact(directory,identity,selectContractExecutables(built.stdout,cwd));
    assert.deepEqual(contractIdentity(cwd),identity,'inputs changed during compilation');
  } else if(mode==='cargo')process.exitCode=runContractArtifact(directory,identity,args,cwd);
  else assert.fail('unsupported mode: '+mode);
}
