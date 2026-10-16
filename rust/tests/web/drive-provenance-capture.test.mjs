import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { provenancePrimitives } from '../../../js/provenance/session.mjs';
import { encodeTyped, decodeTyped } from '../../../js/server/structured-log.mjs';
import { blake3Hex } from '../../../js/crypto/blake3.mjs';
before(async () => {
  await installNodeHost(new WorkerHost());
});
const primitives = provenancePrimitives({
  encodeTyped,
  decodeTyped
});
function readEvidence(directory) {
  const lines = fs.readFileSync(path.join(directory, 'chain.lino'), 'utf8').trim().split('\n');
  const anchor = decodeTyped(fs.readFileSync(path.join(directory, 'anchor.lino'), 'utf8').trim());
  primitives.verifyChain(lines, anchor);
  const records = lines.map(decodeTyped);
  for (const record of records) {
    const evidence = record.payload.evidence;
    if (!evidence) continue;
    const data = decodeTyped(fs.readFileSync(path.join(directory, evidence.file), 'utf8').trim());
    const bytes = Buffer.from(data.value, 'base64');
    assert.equal(blake3Hex(bytes), evidence.blake3);
    assert.equal(bytes.length, evidence.size);
  }
  return {
    lines,
    anchor,
    records
  };
}
test('real maintained planner causally captures exact input/output and executor-time file bytes', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drive-provenance-')),
    workspace = path.join(root, 'work'),
    evidence = path.join(root, 'proof');
  fs.mkdirSync(workspace);
  try {
    const result = await drive(planChatStep, workspace, 'Create file output.mjs containing export const value = 13;', {
      steps: 12,
      provenance: {
        directory: evidence
      }
    });
    assert.equal(result.stop, 'final', JSON.stringify(result));
    assert.equal(fs.readFileSync(path.join(workspace, 'output.mjs'), 'utf8'), 'export const value = 13;');
    const {
      lines,
      anchor,
      records
    } = readEvidence(result.provenance.directory);
    const edits = records.filter(x => x.kind === 'edit');
    assert.equal(edits.length, 1);
    assert.equal(edits[0].payload.preimage, null);
    assert.equal(edits[0].payload.postimage.blake3, blake3Hex(fs.readFileSync(path.join(workspace, 'output.mjs'))));
    assert.ok(records.some(x => x.kind === 'message_out' && x.hash === edits[0].payload.messageOut));
    assert.ok(records.some(x => x.kind === 'message_in'));
    assert.equal(records.at(-1).payload.reason, 'final');
    assert.ok(records.filter(x => x.kind === 'thinking').every(x => ['captured-fields', 'unavailable'].includes(x.payload.status)));
    const tampered = lines.slice();
    tampered.splice(1, 1);
    assert.throws(() => primitives.verifyChain(tampered, anchor));
    assert.equal(result.provenance.authorship, 'not-established');
  } finally {
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('budget and thrown planner failures preserve terminal chain evidence', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drive-provenance-failure-')),
    workspace = path.join(root, 'work'),
    evidence = path.join(root, 'proof');
  fs.mkdirSync(workspace);
  try {
    const result = await drive(planChatStep, workspace, 'unknown', {
      steps: 0,
      provenance: {
        directory: evidence
      }
    });
    assert.equal(result.stop, 'steps');
    assert.equal(readEvidence(result.provenance.directory).records.at(-1).payload.reason, 'steps');
    await assert.rejects(drive(async () => {
      throw Error('test failure');
    }, workspace, 'unknown', {
      provenance: {
        directory: evidence
      }
    }), /test failure/);
    const sessions = fs.readdirSync(evidence).map(p => path.join(evidence, p));
    assert.ok(sessions.some(p => readEvidence(p).records.some(x => x.kind === 'failure')));
  } finally {
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('refuses provenance evidence inside solved workspace', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drive-provenance-boundary-'));
  try {
    await assert.rejects(drive(planChatStep, root, 'unknown', {
      provenance: {
        directory: path.join(root, 'logs')
      }
    }), /outside_workspace/);
    assert.deepEqual(fs.readdirSync(root), []);
  } finally {
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
test('event sink observes exact chosen plans, actual receipts and one outer terminal', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drive-events-'));
  try {
    const events = [];
    const result = await drive(planChatStep, root, 'Create file output.mjs containing export const value = 17;', {
      steps: 12,
      onEvent: event => events.push(event)
    });
    assert.equal(result.stop, 'final');
    assert.equal(events.filter(x => x.kind === 'terminal').length, 1);
    assert.ok(events.some(x => x.kind === 'plan'));
    assert.ok(events.some(x => x.kind === 'tool'));
    assert.equal(events.at(-1).result, result);
    const exception = [];
    await assert.rejects(drive(async () => {
      throw Error('event fixture');
    }, root, 'unknown', {
      onEvent: event => exception.push(event)
    }), /event fixture/);
    assert.equal(exception.filter(x => x.kind === 'exception').length, 1);
    const failed = await drive(planChatStep, root, 'unknown', {
      steps: 0,
      onEvent: () => {
        throw Error('logger unavailable');
      }
    });
    assert.equal(failed.stop, 'steps');
    assert.equal(failed.eventSinkFailures, 1);
  } finally {
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
import { verifyDriveEvidence } from '../../../js/provenance/verify-drive.mjs';
test('retained evidence verifier detects altered output and sidecar bytes without claiming authorship', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'drive-verify-')),
    workspace = path.join(root, 'work'),
    evidence = path.join(root, 'proof');
  fs.mkdirSync(workspace);
  try {
    const result = await drive(planChatStep, workspace, 'Create file output.mjs containing export const value = 19;', {
      steps: 12,
      provenance: {
        directory: evidence
      }
    });
    const directory = result.provenance.directory;
    const verified = verifyDriveEvidence(directory, workspace, result.provenance.anchor);
    assert.equal(verified.anchorTrust, 'caller-pinned');
    assert.equal(verified.authorship, 'not-established');
    assert.deepEqual(verified.files, [{
      path: 'output.mjs',
      status: 'matches-observation'
    }]);
    fs.appendFileSync(path.join(workspace, 'output.mjs'), ' ');
    assert.equal(verifyDriveEvidence(directory, workspace).files[0].status, 'foreign');
    const payload = fs.readdirSync(directory).find(x => /^\d+\.lino$/u.test(x));
    const original = fs.readFileSync(path.join(directory, payload), 'utf8');
    const value = decodeTyped(original.trim());
    value.value = Buffer.from('edited memory evidence').toString('base64');
    fs.writeFileSync(path.join(directory, payload), encodeTyped(value) + '\n');
    assert.throws(() => verifyDriveEvidence(directory, workspace), /evidence_hash/);
  } finally {
    fs.rmSync(root, {
      recursive: true,
      force: true
    });
  }
});
