import assert from 'node:assert/strict';
import test from 'node:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import { mapSourceNetworks } from '../../../scripts/lib/source-network-workers.mjs';

async function withWorker(body, check) {
  const directory = mkdtempSync(join(tmpdir(), 'source-network-pool-'));
  try {
    const path = join(directory, 'worker.mjs');
    writeFileSync(path, "import {parentPort} from 'node:worker_threads';\n" + body);
    await check(pathToFileURL(path));
  } finally { rmSync(directory, { recursive: true, force: true }); }
}

test('bounded pool preserves complete input order across actual asynchronous workers', async () => {
  await withWorker(`parentPort.on('message', ({index,path}) => {
    parentPort.postMessage({event:'start',path});
    setTimeout(() => {
      parentPort.postMessage({event:'finish',path});
      parentPort.postMessage({index,receipt:{path}});
    }, (12-index)*3);
  });`, async (url) => {
    const paths = Array.from({ length: 12 }, (_, index) => 'source-' + index);
    const started = [], active = new Set();
    let peak = 0;
    const receipts = await mapSourceNetworks(paths, url, {}, (message) => {
      if (message.event === 'start') { started.push(message.path); active.add(message.path); }
      else active.delete(message.path);
      peak = Math.max(peak, active.size);
    });
    assert.deepEqual(receipts.map((receipt) => receipt.path), paths);
    assert.equal(started.length, paths.length);
    assert.equal(new Set(started).size, paths.length);
    assert.ok(peak > 0 && peak <= 4);
    assert.equal(active.size, 0);
  });
});

test('worker errors, early exits and mismatched receipt identity never produce complete reports', async () => {
  for (const body of [
    "parentPort.on('message',()=>parentPort.postMessage({error:'actual parse failure'}));",
    "parentPort.on('message',()=>process.exit(0));",
    "parentPort.on('message',({index})=>parentPort.postMessage({index,receipt:{path:'wrong-source'}}));",
  ]) await withWorker(body, async (url) => {
    await assert.rejects(mapSourceNetworks(['expected-source','second-source'], url, {}));
  });
});
