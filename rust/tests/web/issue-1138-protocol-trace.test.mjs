// Issue #1138 R1138-3-5: SWE-bench, the coding ladder and self-authoring
// consume the same protocol document and write the same
// `repository-protocol.lino` evidence.
//
// The JavaScript twin (js/agentic/crate/repository_workspace.mjs) ports the
// document parser, the step applicability rule and the trace renderer; the
// runners stay native. Mirrors rust/tests/unit/ci-cd/protocol_callers.rs.

import assert from 'node:assert/strict';
import { before, describe, it } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let protocol;

before(async () => {
  await installNodeHost(new WorkerHost());
  protocol = await import('../../../js/agentic/crate/repository_workspace.mjs');
});

describe('repository protocol trace (R1138-3-5)', () => {
  it('declares the Agent CLI session stages as data, not as a separate loop', () => {
    const steps = protocol.loadProtocol();
    assert.deepEqual(
      steps.map((step) => step.id),
      ['clone', 'locate', 'read', 'serve', 'edit', 'session', 'verify', 'diff', 'commit'],
    );
    assert.deepEqual(steps.map((step) => step.order), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    const sessionOnly = steps.filter((step) => !protocol.stepAppliesTo(step, protocol.EDITOR_STRUCTURAL));
    assert.deepEqual(sessionOnly.map((step) => step.id), ['serve', 'session']);
    assert.ok(steps.every((step) => protocol.stepAppliesTo(step, protocol.EDITOR_AGENT_SESSION)));
  });

  it('renders every declared stage for every caller, in protocol order', () => {
    const steps = protocol.loadProtocol();
    const ids = steps.map((step) => step.id);
    const structural = protocol.newProtocolTrace(steps, 'solve', protocol.EDITOR_STRUCTURAL);
    const authoring = protocol.newProtocolTrace(steps, 'authoring', protocol.EDITOR_AGENT_SESSION);
    for (const id of ids) protocol.recordStage(authoring, id, 'observed');
    protocol.recordStage(authoring, 'commit', 'refused');
    protocol.recordStage(structural, 'clone', 'observed');
    protocol.recordStage(structural, 'edit', 'stopped');
    protocol.recordStage(structural, 'undeclared', 'observed');
    protocol.setTraceField(structural, 'model', 'formal-ai/0');
    protocol.setTraceField(structural, 'model', 'formal-ai/1');
    structural.open.push('a "quoted" gap');

    const structuralDocument = protocol.renderProtocolTrace(structural);
    const authoringDocument = protocol.renderProtocolTrace(authoring);
    assert.deepEqual(protocol.traceStageIds(structuralDocument), ids);
    assert.deepEqual(protocol.traceStageIds(authoringDocument), ids);
    assert.deepEqual(protocol.traceStageStatuses(structuralDocument), [
      ['clone', 'observed'], ['locate', 'not_reached'], ['read', 'not_reached'],
      ['serve', 'not_applicable'], ['edit', 'stopped'], ['session', 'not_applicable'],
      ['verify', 'not_reached'], ['diff', 'not_reached'], ['commit', 'not_reached'],
    ]);
    assert.equal(
      structuralDocument,
      [
        'repository_protocol_trace',
        '  caller "solve"',
        '  editor "structural"',
        '  model "formal-ai/1"',
        '  stage clone "observed"',
        '  stage locate "not_reached"',
        '  stage read "not_reached"',
        '  stage serve "not_applicable"',
        '  stage edit "stopped"',
        '  stage session "not_applicable"',
        '  stage verify "not_reached"',
        '  stage diff "not_reached"',
        '  stage commit "not_reached"',
        '  open "a ""quoted"" gap"',
        '',
      ].join('\n'),
    );
  });
});
