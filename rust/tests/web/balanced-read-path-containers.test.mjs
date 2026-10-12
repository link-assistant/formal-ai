import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { readPolicyBlocksPlan } from '../../../js/agentic/file_read/ownership.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
await installNodeHost(new WorkerHost());

test('balanced path containers deliver the exact unchanged physical Read', async () => {
  const requests = [
    ['Open (\x60Cargo.toml\x60).', 'Cargo.toml'],
    ['Open ["records.txt"].', 'records.txt'],
    ["Read {'notes.txt'}.", 'notes.txt'],
    ['Read ((\x60other.txt\x60)).', 'other.txt'],
  ];
  for (const [request, target] of requests) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'balanced-read-path-'));
    try {
      const contents = 'SOURCE π\n';
      fs.writeFileSync(path.join(directory, target), contents);
      const result = await drive(planChatStep, directory, request, { steps: 6 });
      assert(result.transcript.some(item => item.tool === 'read'));
      assert(result.answer.includes(contents.trim()));
      assert.equal(fs.readFileSync(path.join(directory, target), 'utf8'), contents);
      assert.deepEqual(fs.readdirSync(directory), [target]);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});

test('unbalanced containers, payloads and unsolved independent Needs grant no Read', async () => {
  const requests = [
    'Open (\x60Cargo.toml\x60)).',
    'Open ((\x60Cargo.toml\x60).',
    'Open (\x60Cargo.toml\x60].',
    'Open (\x60Cargo.toml\x60.',
    'Example: Open (\x60Cargo.toml\x60).',
    'Write note.txt containing "Open (\x60Cargo.toml\x60)."',
    'Do not read Cargo.toml. Open (\x60Cargo.toml\x60).',
    'Open (\x60Cargo.toml\x60) and deploy.',
    'Open (\x60Cargo.toml\x60) and juggle the moon.',
    'Open (\x60Cargo.toml\x60) and (deploy).',
    'Open (\x60Cargo.toml\x60) after permission is granted.',
  ];
  for (const request of requests) {
    const plan = await planChatStep([{ role: 'user', content: request }], ['Read']);
    assert(!(plan?.calls ?? []).some(call => call.tool === 'Read'));
  }
});


test('containers inside quoted payloads cannot introduce a Read condition', () => {
  const request = 'Write note.txt containing "Open (\x60Cargo.toml\x60) and juggle the moon."';
  const plan = { kind: 'tool_calls', calls: [{ tool: 'Read', arguments: JSON.stringify({ path: 'note.txt' }) }] };
  assert.equal(readPolicyBlocksPlan(request, plan), false);
});
