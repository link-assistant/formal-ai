// Replay the native specification's observed dispatch case without compiling Rust.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';
const jobs=YAML.parse(readFileSync('.github/workflows/external-benchmarks.yml','utf8')).jobs;
function selected(id,event,ref,shard_size) {
  const expression=jobs[id].if.trim().replace(/^\$\{\{\s*|\s*\}\}$/gu,'');
  return Boolean(new Script(expression).runInNewContext({github:{event_name:event,ref},inputs:{shard_size}}, {timeout:100}));
}
test('single and sharded benchmark dispatches are exclusive and main-only', () => {
  for (const size of [0,32]) {
    assert.equal(selected('external-benchmarks','workflow_dispatch','refs/heads/main',size),size===0);
    assert.equal(selected('shard-plan','workflow_dispatch','refs/heads/main',size),size!==0);
    for (const [event,ref] of [['pull_request','refs/pull/1/merge'],['workflow_dispatch','refs/heads/feature']]) {
      assert.equal(selected('external-benchmarks',event,ref,size),false);
      assert.equal(selected('shard-plan',event,ref,size),false);
    }
  }
  assert.equal(selected('external-benchmarks','schedule','refs/heads/main',0),true);
  assert.equal(selected('shard-plan','schedule','refs/heads/main',0),false);
});
