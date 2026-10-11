import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { readRequestEnvelope } from '../../../js/agentic/file_read/ownership.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
await installNodeHost(new WorkerHost());

test('complete quoted Read preserves original physical source and its span', async () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'quoted-read-envelope-'));
  try {
    const prompt = '"read the file alpha.txt and print its contents"';
    const contents = 'ALPHA\n';
    fs.writeFileSync(path.join(directory, 'alpha.txt'), contents);
    const envelope = readRequestEnvelope(prompt);
    assert.equal(prompt.slice(envelope.start, envelope.end), envelope.text);
    assert.equal(envelope.sourceUnit, 'utf16');
    const result = await drive(planChatStep, directory, prompt, { steps: 6 });
    assert(result.transcript.some(item => item.tool === 'read'));
    assert(result.answer.includes(contents.trim()));
    assert.equal(fs.readFileSync(path.join(directory, 'alpha.txt'), 'utf8'), contents);
    assert.deepEqual(fs.readdirSync(directory), ['alpha.txt']);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test('literal, attributed, partial and independently constrained quotes grant no Read', async () => {
  const requests = [
    'Example: "read the file alpha.txt and print its contents"',
    '"summarize the file alpha.txt and print its contents"',
    '"read the file summary.txt and print its contents"',
    'Write note.txt containing "read the file alpha.txt and print its contents"',
    '"Do not read alpha.txt. read the file alpha.txt and print its contents"',
    '"read the file alpha.txt and print its contents. Deploy the project"',
    '"read the file alpha.txt and print its contents. Do not write files"',
    '"read the file alpha.txt and print its contents',
    '"read the file alpha.txt and print its contents" and deploy',
    '```read the file alpha.txt and print its contents```',
    '"read the file alpha.txt and print its contents after permission is granted"',
  ];
  for (const request of requests) {
    assert.equal(readRequestEnvelope(request), null);
    const plan = await planChatStep([{ role: 'user', content: request }], ['Read']);
    assert(!(plan?.calls ?? []).some(call => call.tool === 'Read'));
  }
});
