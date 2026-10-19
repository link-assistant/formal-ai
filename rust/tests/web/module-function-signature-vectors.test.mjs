import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { moduleFunctionRequest } from '../../../js/agentic/module_function.mjs';
const vectors = JSON.parse(readFileSync(new URL('../fixtures/module-function/signature-vectors.json', import.meta.url)));
test('shared declarative module signature vectors on production JS request parser', async () => {
  await installNodeHost(new WorkerHost({ WebAssembly: {
    instantiate() { throw Error('LocalWasmExecutionForbidden'); },
    compile() { throw Error('LocalWasmExecutionForbidden'); },
  } }));
  for (const row of vectors.cases) {
    const request = moduleFunctionRequest(row.prompt);
    assert.deepEqual(request?.parameters ?? null, row.expected, row.id);
    if (row.expected !== null) {
      assert.equal(request.name, 'carried', row.id);
      assert.equal(request.module, 'carried.mjs', row.id);
    }
  }
});
