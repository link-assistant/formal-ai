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
  const sites=[['cli_solve.rs',462],['cli_memory.rs',77],['skill_ledger.rs',267],['dreaming.rs',761],['algorithm_discovery.rs',164],['algorithm_discovery.rs',350],['external_benchmarks/mod.rs',329],['coding/concept_discovery.rs',248]];
  for(const [filename,line] of sites){const source=fs.readFileSync(path.join(root,'rust/src',filename),'utf8');const result=auditor.rust(source);assert.equal(result.violations.some(value=>value.line===line),false,filename+':'+line);assert.ok(result.unresolved.some(value=>value.line===line),filename+':'+line+' must actually be observed');}
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
