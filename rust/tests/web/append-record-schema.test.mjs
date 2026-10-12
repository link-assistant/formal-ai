import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import {
  APPEND_CONTRACT, appendReceiptValid, generalChangePlanRecordHeader,
  generalChangePlanRecordHeaderFrom,
} from '../../../js/agentic/append_contract.mjs';
import { composeGeneralChangePlan, planLinksNotation } from '../../../js/agentic/general_planner.mjs';
before(async () => installNodeHost(new WorkerHost()));
const valid = [
  'agentic_tool_capabilities',
  '  record-schema "general-change-plan"',
  '    header "general_change_plan"',
  '',
].join('\n');
test('declared record header has one authoritative root, record, and single-line header', () => {
  assert.equal(generalChangePlanRecordHeaderFrom(valid), 'general_change_plan\n');
  const invalid = [
    null, '', valid.replace('agentic_tool_capabilities', 'foreign_root'),
    valid + 'foreign_root\n', valid.replace('general-change-plan', 'foreign-record'),
    valid.replace('    header "general_change_plan"\n', ''),
    valid + '  record-schema "general-change-plan"\n    header "general_change_plan"\n',
    valid + '    header "general_change_plan"\n',
    valid.replace('general_change_plan', ''),
    valid.replace('general_change_plan', 'general-change-plan'),
    valid.replace('general_change_plan', 'General_change_plan'),
    valid.replace('general_change_plan', 'λ_record'),
  ];
  for (const input of invalid) assert.equal(generalChangePlanRecordHeaderFrom(input), null, String(input));
  assert.equal(generalChangePlanRecordHeader(), 'general_change_plan\n');
  assert.equal(generalChangePlanRecordHeaderFrom(readText('data/seed/agentic-tool-capabilities.lino')),
    generalChangePlanRecordHeader());
  const plan = composeGeneralChangePlan('Write «marker» to proof.txt.');
  assert.ok(plan);
  assert.ok(planLinksNotation(plan).startsWith(generalChangePlanRecordHeader()));
});
function argumentsFor() {
  return { path: '.formal-ai/general-change-plan.lino', record_id: '  id "selected"',
    content: 'general_change_plan\n  id "selected"\n  goal "preserve λ🙂"\n',
    append_request_id: '  id "selected"/1', append_mode: 'atomic_record_append' };
}
function receiptFor(args, before) {
  return { schema: APPEND_CONTRACT, complete: true, success: true, ...args,
    operation: 'already_present', before, after: before,
    before_bytes: Buffer.byteLength(before), after_bytes: Buffer.byteLength(before) };
}
test('seed-defined boundary preserves exact old header bytes and refuses partial or shifted records', () => {
  const args = argumentsFor();
  for (const following of ['', 'general_change_plan\n  id "later"\n']) {
    assert.equal(appendReceiptValid(receiptFor(args, args.content + following), args), true);
  }
  for (const following of ['general_change_plan', 'general_change_planX\n',
    '\ngeneral_change_plan\n', 'general_change_plan\r\n', 'foreign_record\n']) {
    assert.equal(appendReceiptValid(receiptFor(args, args.content + following), args), false, following);
  }
  assert.equal(appendReceiptValid(receiptFor(args, 'prefix' + args.content), args), false);
  assert.equal(appendReceiptValid(receiptFor(args, args.content + args.content), args), false);
  assert.equal(appendReceiptValid({ ...receiptFor(args, args.content), after: args.content + 'changed' }, args), false);
});
