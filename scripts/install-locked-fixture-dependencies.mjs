// Recover the observed read-only registry conflict without changing locked bytes.
import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import {boundedCommand} from '../js/server/bounded-command.mjs';
const conflict = /^error: GET https:\/\/registry[.]npmjs[.]org\/[a-zA-Z0-9@._%+/-]+[.]tgz - 409\s*$/mu;
export async function installLockedFixtures({invoke = boundedCommand,
  wait = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds)),
  stdout = bytes => process.stdout.write(bytes), stderr = bytes => process.stderr.write(bytes)} = {}) {
  for (let attempt = 1; attempt <= 3; attempt++) {
    stderr(`Locked fixture dependency attempt ${attempt}/3\n`);
    const result = await invoke('bun', ['install','--frozen-lockfile','--ignore-scripts'], {
      timeout:120000, maxBuffer:2*1024*1024, onStdout:stdout, onStderr:stderr,
    });
    assert(result.cleanup_complete && !result.error && !result.timed_out && !result.signal
      && Number.isInteger(result.status),
      'fixture installer did not complete cleanly; retry refused');
    if (result.status === 0) return;
    const errors = (String(result.stdout ?? '')+'\n'+String(result.stderr ?? ''))
      .split('\n').filter(line => line.startsWith('error:'));
    assert(result.status === 1 && attempt < 3 && errors.length > 0 && errors.every(line => conflict.test(line)),
      'locked fixture installation failed; no further attempts authorized');
    await wait(attempt * 1000);
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  assert.equal(process.argv.length, 2, 'installer accepts no mutable resolution flags');
  await installLockedFixtures();
}
