import test from 'node:test';
import assert from 'node:assert/strict';
import {checkTypeScriptRuntime} from '../../../scripts/check-typescript-runtime.mjs';

test('actual TS-source worker and planner match JS, with no owned-JS fallback', async () => {
  const result = await checkTypeScriptRuntime();
  assert(result.typeScriptInputOperations > 100);
  assert(result.uniqueTypeScriptInputs > 100);
  assert.equal(result.originalJavaScriptFallback, false);
  assert.equal(result.nativeExecution, false);
  assert.equal(result.productWasmExecution, false);
  assert.equal(result.cases.length, 4);
});
