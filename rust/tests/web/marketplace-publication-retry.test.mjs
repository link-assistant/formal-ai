import assert from 'node:assert/strict';
import fs from 'node:fs';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { test } from 'node:test';
const root=process.env.MARKETPLACE_RETRY_REPOSITORY || fileURLToPath(new URL('../../../',import.meta.url));
const yaml=createRequire(join(root,'package.json'))('yaml');
const source=fs.readFileSync(process.env.MARKETPLACE_RETRY_WORKFLOW || join(root,'.github/workflows/publish-vscode.yml'),'utf8');
const workflow=yaml.parse(source);
const step=workflow.jobs['authoritative-publication'].steps.find(value=>value.name==='Install published Marketplace extension in a clean profile');
assert.ok(step);
const start=step.run.indexOf('for _ in 1 2 3 4 5; do');
assert.ok(start>=0);
const loop=step.run.slice(start);
function observe(failures,wrong=false) {
 const setup=[
  'EXPECTED_VERSION=1.2.3', 'code=mock_code', 'attempt=0',
  'mock_code() {', 'case " $* " in',
  '*" --install-extension "*) attempt=$((attempt+1)); echo "INSTALL:$attempt" >&2;',
  'if [ "$attempt" -le '+failures+' ]; then return 7; fi;;',
  '*" --list-extensions "*) echo "READBACK:$attempt" >&2;',
  'echo "link-assistant.formal-ai-vscode@'+(wrong?'9.9.9':'1.2.3')+'";;',
  '*) return 8;;', 'esac', '}', 'sleep() { echo SLEEP >&2; }', ''
 ].join('\n');
 const result=spawnSync('bash',['-e','-o','pipefail','-c',setup+loop],{encoding:'utf8',timeout:3000});
 assert.equal(result.error,undefined);
 return result;
}
test('transient Marketplace installation failure reaches the next attempt and exact version readback',()=>{
 const result=observe(1);
 assert.equal(result.status,0);
 assert.match(result.stderr,/INSTALL:2\nREADBACK:2/u);
 assert.doesNotMatch(result.stderr,/READBACK:1/u);
});
test('failed Marketplace installation never accepts a stale matching installed version',()=>{
 const result=observe(5);
 assert.equal(result.status,1);
 assert.equal((result.stderr.match(/INSTALL:/gu)||[]).length,5);
 assert.doesNotMatch(result.stderr,/READBACK/u);
});
test('wrong Marketplace version exhausts all five attempts and fails',()=>{
 const result=observe(0,true);
 assert.equal(result.status,1);
 assert.equal((result.stderr.match(/READBACK:/gu)||[]).length,5);
});
test('an immediately installable exact version succeeds without redundant attempts',()=>{
 const result=observe(0);
 assert.equal(result.status,0);
 assert.equal((result.stderr.match(/INSTALL:/gu)||[]).length,1);
});
