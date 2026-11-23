import '../../../experiments/issue-1580/buildkit-image-identity.test.mjs';
import '../../../experiments/issue-1580/compressed-secrets.test.mjs';
import '../../../experiments/issue-1580/secretlint-locked-install.test.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import {subprocessBoundaryAuditor} from '../../../scripts/lib/subprocess-boundary-audit.mjs';
const root=process.env.FORMAL_AI_BOUNDARY_RUNTIME || process.cwd();
const auditor=await subprocessBoundaryAuditor(root);
test.after(()=>auditor.close());
test('Rust actual constructors, imports, typed children and fields refuse raw blocking',()=>{
  const source='use std::process::{Command as Process, Child}; struct Guard { child: Child } impl Guard { fn stop(&mut self) { self.child.wait(); } } fn run(child: &mut Child) { let mut command = Process::new("git"); command.args(["status"]); command.output(); child.wait_with_output(); std::process::Command::new("x").status(); }';
  assert.deepEqual(auditor.rust(source).violations.map(value=>value.operation),['wait','output','wait_with_output','status']);
});
test('local constructor aliases and spawned child provenance survive chains',()=>{
  const result=auditor.rust('use std::process::Command; fn run() { let mut command=Command::new("x"); let alias=command; let mut child=alias.spawn().unwrap(); child.wait(); }');
  assert.deepEqual(result.violations.map(value=>value.operation),['wait']);assert.equal(result.unresolved.length,0);
  assert.deepEqual(auditor.rust('use std::process::Command; fn run() { let mut child=Command::new("x").spawn().expect("spawn"); child.wait_with_output(); }').violations.map(value=>value.operation),['wait_with_output']);
});
test('eight original domain status boundaries remain non-process',()=>{
  // Each site is located by a source anchor that occurs exactly once in its
  // file, not by a pinned line number: a pinned number went stale when an
  // unrelated edit above it moved the site (issue #1569, dreaming.rs 761 -> 778).
  const sites=[['cli_solve.rs','trace.status(COMMIT_STAGE) != Some'],['cli_memory.rs','"status": error.status()'],['skill_ledger.rs','status: gate.status()'],['dreaming.rs','format!("status={}", candidate.status())'],['algorithm_discovery.rs','Some(self.status())'],['algorithm_discovery.rs','!= candidate.status()'],['external_benchmarks/mod.rs','trace.status(BENCHMARK_COMMIT_STAGE) == Some'],['coding/concept_discovery.rs','Some(need.status())']];
  for(const [filename,anchor] of sites){
    const source=fs.readFileSync(path.join(root,'rust/src',filename),'utf8');
    const lines=source.split('\n').flatMap((text,index)=>text.includes(anchor)?[index+1]:[]);
    assert.equal(lines.length,1,filename+' must hold the anchor '+JSON.stringify(anchor)+' exactly once');
    const [line]=lines;const result=auditor.rust(source);
    assert.equal(result.violations.some(value=>value.line===line),false,filename+':'+line);
    assert.ok(result.unresolved.some(value=>value.line===line),filename+':'+line+' must actually be observed');
  }
});
test('terminal allowance requires the exact two terminal predicates and consequence',()=>{
 const prefix='use std::process::Command; fn run() { let mut command=Command::new("x"); ';
 const condition='std::io::stdin().is_terminal() && std::io::stdout().is_terminal()';
 assert.equal(auditor.rust(prefix+'if '+condition+' { command.status(); } }').allowed.length,1);
 assert.equal(auditor.rust(prefix+'if '+condition+' || true { command.status(); } }').violations.length,1);
 assert.equal(auditor.rust(prefix+'if '+condition+' {} else { command.status(); } }').violations.length,1);
});
test('JS imported and namespace synchronous child APIs refuse unbounded calls',()=>{
 const source='import {spawnSync as execute} from "node:child_process"; import * as processApi from "child_process"; execute("x", []); processApi.execFileSync("x", []); const {execSync: shell}=await import("node:child_process"); shell("x");';
 assert.equal(auditor.javascript(source).violations.length,3);
});
test('JS finite ignore startup policy keeps SIGKILL and literal deadline mandatory',()=>{
 const prefix='import {spawnSync} from "node:child_process"; ';
 const source='spawnSync("python3", ["-c","pass"], {stdio:"ignore",timeout:1000,killSignal:"SIGKILL"});';
 assert.equal(auditor.javascript(prefix+source).allowed.length,1);
 const forged='import {execSync as execute} from "node:child_process"; execute("sleep forever", [], {stdio:"ignore",timeout:1000,killSignal:"SIGKILL"});';
 assert.equal(auditor.javascript(forged).violations.length,1);
 assert.equal(auditor.javascript(forged).allowed.length,0);
 for(const mutation of [source.replace('1000','1001'),source.replace('SIGKILL','SIGTERM'),source.replace('"ignore"','"pipe"'),source.replace('timeout:1000','timeout:duration'),source.replace('stdio:"ignore"','...other,stdio:"ignore"')])assert.equal(auditor.javascript(prefix+mutation).violations.length,1);
});
test('opaque macro call shapes retain process provenance without parsing macro data as Rust',()=>{
 const result=auditor.rust('use std::process::Child; fn run(child: &mut Child) { json!({"status": domain.status(), "blocked": child.wait(), "text":"child.wait()"}); other!(child.wait_with_output(), /* child.output() */ domain.status()); }');
 assert.deepEqual(result.violations.map(value=>value.operation),['wait','wait_with_output']);
 assert.equal(result.unresolved.length,2);
 assert.ok(result.violations.every(value=>value.coverage==='opaque-macro-token-call-shape'));
});
test('comments, strings, domain methods and bounded calls do not masquerade as child APIs',()=>{
 assert.equal(auditor.javascript('const message="spawnSync(x)"; domain.status(); // execSync(x)\nboundedCommandSync("x");').violations.length,0);
 assert.equal(auditor.rust('fn run() { let message="Command::new(\\"x\\").output()"; /* child.wait(); */ domain.status(); }').violations.length,0);
});


// These execute the actual maintained CLIs in hosted web-test jobs.
import { boundedCommand } from '../../../js/server/bounded-command.mjs';

async function hostedCheck(relative, argumentsValue = []) {
  const result = await boundedCommand(process.execPath,
    [path.join(root, relative), ...argumentsValue], {
      cwd: root, timeout: 60000, maxBuffer: 1024 * 1024,
    });
  assert.equal(result.error, undefined, result.stderr.toString());
  assert.equal(result.status, 0, result.stdout.toString());
  assert.equal(result.cleanup_complete, true);
  return result.stdout.toString();
}

test('hosted subprocess boundary scan covers maintained source directories', async () => {
  const output = await hostedCheck('scripts/check-bounded-subprocess-boundaries.mjs');
  const receipt = JSON.parse(output.trim());
  assert.equal(receipt.violations, 0);
  assert.ok(receipt.files > 0);
  assert.match(receipt.coverage, /no macro-expansion/u);
});

test('hosted capture source projection remains mechanically current', async () => {
  const output = await hostedCheck('scripts/generate-bounded-command-capture.mjs', ['--check']);
  assert.match(output, /bounded command capture projection verified/u);
});

 test('dynamic child-process imports retain provenance under both Babel AST representations',()=>{
  for(const createImportExpressions of [false,true]){
    const options={createImportExpressions};
    const fixture="import {spawnSync as execute} from \"node:child_process\"; import * as processApi from \"child_process\"; execute(\"x\", []); processApi.execFileSync(\"x\", []); const {execSync: shell}=await import(\"node:child_process\"); shell(\"x\");";
    assert.equal(auditor.javascript(fixture,'source.mjs',options).violations.length,3);
    for(const declaration of ['const api=await import("node:child_process");', 'const {execFileSync: execute}=await import("child_process");']){
      const call=declaration.includes('{')?'execute("x", []);':'api.execFileSync("x", []);';
      assert.equal(auditor.javascript(declaration+call,'source.mjs',options).violations.length,1);
    }
    assert.equal(auditor.javascript('const {execSync: shell}=await import("unrelated"); shell("x");','source.mjs',options).violations.length,0);
    assert.equal(auditor.javascript('const {spawnSync: run}=await import("node:child_process"); run("x", [], {stdio:"ignore",timeout:1000,killSignal:"SIGKILL"});','source.mjs',options).allowed.length,1);
  }
});


test('capture generation checks and repairs the native embedded consumer together with canonical source',async t=>{
  const fs=await import('node:fs'),path=await import('node:path'),os=await import('node:os');
  const {generateBoundedCommandCapture}=await import('../../../scripts/generate-bounded-command-capture.mjs');
  const repository=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-capture-mirror-'));
  t.after(()=>fs.rmSync(repository,{recursive:true,force:true}));
  fs.mkdirSync(path.join(repository,'js/server'),{recursive:true});
  fs.writeFileSync(path.join(repository,'js/server/bounded-command.mjs'),
    'export async function boundedCommand() {\n  // Preserve actual source comments.\n  return "fixture";\n}\n');
  const outputs=generateBoundedCommandCapture({repository});
  assert.equal(outputs.length,2);assert(fs.readFileSync(outputs[0]).equals(fs.readFileSync(outputs[1])));
  assert(fs.readFileSync(outputs[0],'utf8').includes('Preserve actual source comments.'));
  generateBoundedCommandCapture({repository,check:true});
  fs.writeFileSync(outputs[1],'stale native body\n');
  assert.throws(()=>generateBoundedCommandCapture({repository,check:true}),/projection drift.*rust/);
  generateBoundedCommandCapture({repository});
  generateBoundedCommandCapture({repository,check:true});
  assert(fs.readFileSync(outputs[0]).equals(fs.readFileSync(outputs[1])));
});


test('readable capture projection is valid canonical notation and round-trips quotes, escapes and UTF-8',async t=>{
  const {generateBoundedCommandCapture}=await import('../../../scripts/generate-bounded-command-capture.mjs');
  const {Parser}=await import('links-notation');
  const {installNodeHost}=await import('../../../js/agentic/node-host.mjs');
  const {WorkerHost}=await import('../../../js/server/worker-host.mjs');
  const {parseLinoRoot}=await import('../../../js/agentic/write_lino.mjs');
  const os=await import('node:os'),repository=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-capture-notation-'));
  t.after(()=>fs.rmSync(repository,{recursive:true,force:true}));
  const source=['export async function boundedCommand() {',
        `  // Unicode: Привет 世界; "double" and 'single' quotes.`,
    '  return "literal \\n and \\t";', '}'].join('\n');
  fs.mkdirSync(path.join(repository,'js/server'),{recursive:true});
  fs.writeFileSync(path.join(repository,'js/server/bounded-command.mjs'),source);
  const [output]=generateBoundedCommandCapture({repository});
  const notation=fs.readFileSync(output,'utf8');
  assert.doesNotThrow(()=>new Parser().parse(notation));
  await installNodeHost(new WorkerHost({WebAssembly:{
    compile(){throw Error('LocalWasmExecutionForbidden');},
    instantiate(){throw Error('LocalWasmExecutionForbidden');}}}));
  const tree=parseLinoRoot(notation).children[0];
  const restored=tree.children.filter(node=>node.name==='line').map(node=>node.value??node.id).join('\n');
  assert.equal(restored,source.replace('export ',''));
});


test('secret scanner uses one bounded npm command and retains every literal filename through NUL stdin', {timeout: 60000}, async t => {
  const os = await import('node:os');
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-secret-batches-'));
  t.after(() => fs.rmSync(temporary, {recursive: true, force: true}));
  const receipts = path.join(temporary, 'receipts.jsonl');
  // Model npm's fixed shell command and the separate filename stream.
  fs.writeFileSync(path.join(temporary, 'npx'), `#!${process.execPath}
const fs = require('node:fs');
const args = process.argv.slice(2);
const marker = args.indexOf('node');
if (marker < 0) process.exit(2);
if (args[marker + 1] !== 'scripts/check-secrets-files.mjs' || args.length !== marker + 2) process.exit(3);
const files = fs.readFileSync(0, 'utf8').split(String.fromCharCode(0)).filter(Boolean);
const quote = value => "'" + value.replaceAll("'", "'" + String.fromCharCode(92) + "''") + "'";
const bytes = Buffer.byteLength(args.map(quote).join(' '));
fs.appendFileSync(process.env.SECRET_SCAN_RECEIPTS, JSON.stringify({files, bytes}) + '\\n');
if (bytes > 120 * 1024) { console.error('mock npm spawn E2BIG'); process.exit(7); }
if (files.includes('fixture-detected-secret')) process.exit(1);
`, {mode: 0o755});
  const files = Array.from({length: 1800}, (_, index) =>
    `fixture/${index.toString().padStart(4, '0')}/a file with 'quotes' 世界 ${'a'.repeat(70)}.txt`);
  const run = argumentsValue => boundedCommand('bash',
    [path.join(root, 'scripts/check-secrets.sh'), ...argumentsValue], {
      cwd: root, timeout: 20000, maxBuffer: 1024 * 1024,
      env: {...process.env, PATH: `${temporary}${path.delimiter}${process.env.PATH}`,
        SECRET_SCAN_RECEIPTS: receipts, SECRETLINT_VERSION: '13.0.5'},
    });
  const result = await run(files);
  assert.equal(result.error, undefined, result.stderr.toString());
  assert.equal(result.status, 0, result.stderr.toString());
  assert.equal(result.cleanup_complete, true);
  const batches = fs.readFileSync(receipts, 'utf8').trim().split('\n').map(line => JSON.parse(line));
  assert.equal(batches.length, 1);
  assert.deepEqual(batches.flatMap(batch => batch.files), files);
  assert.ok(batches.every(batch => batch.bytes <= 120 * 1024));
  const finding = await run(['fixture-detected-secret']);
  assert.equal(finding.error, undefined);
  assert.notEqual(finding.status, 0, 'scanner findings must still fail the gate');
  assert.equal(finding.cleanup_complete, true);
});


test('secret scanner refuses a failed or partial Git inventory before scanning', {timeout: 30000}, async t => {
  const os = await import('node:os');
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'secret-inventory-'));
  t.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  const receipt = path.join(directory, 'scanner-called');
  fs.writeFileSync(path.join(directory, 'git'), `#!${process.execPath}
const command=process.argv[2],mode=process.env.SECRET_INVENTORY_CASE;
if(command==='rev-parse'||command==='merge-base'){console.log('fixture-commit');process.exit(0);}
if(command==='diff'||command==='ls-files'){
  if(mode==='empty')process.exit(0);
  process.stdout.write('partial inventory.txt'+String.fromCharCode(0));
  if(command==='diff'||mode==='all')process.exit(17);
  process.exit(0);
}
process.exit(18);
`, {mode: 0o755});
  fs.writeFileSync(path.join(directory, 'npx'), `#!${process.execPath}
require('node:fs').writeFileSync(process.env.SECRET_INVENTORY_RECEIPT,'called');
`, {mode: 0o755});
  for (const mode of ['all', 'uncommitted', 'base', 'empty']) {
    fs.rmSync(receipt, {force: true});
    const result = await boundedCommand('bash', [path.join(root, 'scripts/check-secrets.sh'),
      ...(['all', 'empty'].includes(mode) ? ['--all'] : [])], {
      cwd: root, timeout: 5000, maxBuffer: 65536,
      env: {...process.env, PATH: `${directory}${path.delimiter}${process.env.PATH}`,
        BASE_REF: mode === 'base' ? 'main' : '', SECRET_INVENTORY_CASE: mode,
        SECRET_INVENTORY_RECEIPT: receipt},
    });
    assert.equal(result.error, undefined, mode);
    assert.equal(result.cleanup_complete, true, mode);
    assert.equal(result.status === 0, mode === 'empty', mode+' must preserve inventory failure');
    assert.equal(fs.existsSync(receipt), false, mode+' must not scan an incomplete inventory');
    if (mode === 'empty') assert.match(result.stdout.toString(), /No files to scan/u);
    else assert.doesNotMatch(result.stdout.toString(), /No secrets found/u);
  }
});
