// PR #1188 dogfooding: the single-quoted payloads of a `Replace 'X' with 'Y'`
// edit request are data, not computer-use cues. A replacement naming a
// precedence "order" and a built "list" was planned as a computer-use order
// listing (plan synthesized-computer_use_resource_orders-computer_use_list_directory)
// because `instructionSurface` dropped only double-quoted spans. It now drops
// every quote pair `textOutsideQuotedSegments` reads. Twin of
// rust/tests/unit/pull_request_1188_quoted_payload_cues.rs.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { computerUsePlanAgenticStep } from '../../../js/agentic/crate/computer_use_planner.mjs';
import { planRequest } from '../../../js/agentic/crate/computer_use.mjs';
import { instructionSurface } from '../../../js/agentic/crate/computer_use_lexicon.mjs';

const AGENT_CLI_TOOLS = ['bash', 'edit', 'glob', 'grep', 'list', 'read', 'webfetch', 'websearch', 'write'];
const EDIT = "Replace 'Partially implemented' with 'Pinned: every handler in precedence order, "
  + "every subcommand (also equal to the built `formal-ai --help` list), and the parts.' in docs/r.md.";

const user = (content) => ({ role: 'user', content, tool_calls: [], tool_call_id: null, name: null });

before(async () => {
  await installNodeHost(new WorkerHost());
});

test('single-quoted edit payloads leave the instruction surface', () => {
  assert.equal(instructionSurface(EDIT), 'Replace   with   in docs/r.md.\n');
  assert.equal(instructionSurface("List the orders in 'cache' don't wait"), 'List the orders in   don\'t wait\n');
});

test('an edit whose payload names orders and a list is not a computer-use plan', () => {
  assert.equal(planRequest(EDIT), null);
  assert.equal(computerUsePlanAgenticStep([user(EDIT)], AGENT_CLI_TOOLS), null);
});

test('an unquoted computer-use request still plans', () => {
  const prompt = 'Count the sub-tasks of the customer import rewrite and save the result in `counts.md`.';
  assert.notEqual(planRequest(prompt), null);
});

// G105 (TRANSLATE, T773): a «…» payload that runs over a blank line was read
// line by line, so neither line closed its pair and the payload's "order" and
// "count" planned a computer-use count of orders.
const MULTILINE = 'Replace «old cell» with «A.\n\norder, count» in r.md.';

test('a quoted payload over several lines leaves the instruction surface whole', () => {
  assert.equal(instructionSurface(MULTILINE), 'Replace   with   in r.md.\n');
  assert.equal(planRequest(MULTILINE), null);
  assert.equal(computerUsePlanAgenticStep([user(MULTILINE)], AGENT_CLI_TOOLS), null);
});
