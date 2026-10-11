// Issue #1166 R1166-3 parity: the executor's CI-workflow decision reads the
// formalized request in both roots. `requestedIn` (js/agentic/ci_workflow.mjs)
// mirrors `ci_workflow::requested_in`, which delegates to
// `intent_formalization::request_demands(objective, CiWorkflow)`; the cases
// are rust/tests/unit/issue_1166_obligation_routing.rs
// `ci_workflow_demand_is_read_from_the_obligation_graph`, over the same
// canonical fixture.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { requestedIn } from '../../../js/agentic/ci_workflow.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

test('ci_workflow_demand_is_read_from_the_obligation_graph', () => {
  const canonical = readFileSync(
    new URL('../fixtures/issue-1166/hello-world-kotlin-en.txt', import.meta.url),
    'utf8',
  );
  const demands = [
    canonical,
    'Print exactly "add a GitHub Actions workflow"',
    'Write a Kotlin program',
  ].map(requestedIn);
  assert.deepEqual(demands, [true, false, false]);
});
