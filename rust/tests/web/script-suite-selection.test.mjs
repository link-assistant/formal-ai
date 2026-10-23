// Exercise the actual selector without running or compiling Rust.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {spawnSync} from 'node:child_process';
test('script selection keeps standalone tests and excludes helpers compiled by either native or gated consumers',t=>{
  const root=mkdtempSync(join(tmpdir(),'formal-ai-script-selection-'));
  t.after(()=>rmSync(root,{recursive:true,force:true}));
  const write=(name,text)=>{const file=join(root,name);mkdirSync(dirname(file),{recursive:true});writeFileSync(file,text);};
  write('scripts/test-scripts.sh',readFileSync(new URL('../../../scripts/test-scripts.sh',import.meta.url)));
  write('rust/tests/unit/entry.rs','#[path = "../../../scripts/native-consumer.rs"] mod native_consumer;\n');
  write('scripts/native-consumer.rs','#[path = "native-helper.rs"] mod helper;\n#[cfg(test)] mod tests {}\n');
  write('scripts/native-helper.rs','#[cfg(test)] mod tests {}\n');
  write('scripts/gated-consumer.rs','#[path = "gated-helper.rs"] mod helper;\n#[cfg(test)] mod tests {}\n');
  write('scripts/gated-helper.rs','#[cfg(test)] mod tests {}\n');
  write('scripts/standalone.rs','#[cfg(test)] mod tests {}\n');
  write('data/meta/ci-gates/example.lino','run "rust-script --test scripts/gated-consumer.rs"\n');
  mkdirSync(join(root,'.github/workflows'),{recursive:true});
  const result=spawnSync('bash',['scripts/test-scripts.sh','--list'],{cwd:root,encoding:'utf8',timeout:10000,maxBuffer:1048576});
  assert.equal(result.error,undefined);assert.equal(result.status,0,result.stderr);
  assert.deepEqual(result.stdout.trim().split('\n'),['scripts/standalone.rs']);
});
