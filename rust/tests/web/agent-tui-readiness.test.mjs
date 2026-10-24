import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {promptAfterFor} from '../../../experiments/agent_cli_e2e/issue_819_tui/client-contract.mjs';

const fixture = JSON.parse(fs.readFileSync(new URL(
  '../../../experiments/agent_cli_e2e/issue_819_tui/fixtures/codex-startup.json', import.meta.url)));

test('the actual Codex loading frame cannot submit a prompt before the selected model is ready', () => {
  const ready = promptAfterFor('codex');
  assert.equal(fixture.loadingFrame.includes('formal-ai default'), true);
  assert.equal(ready.test(fixture.loadingFrame), false);
  assert.equal(ready.test(fixture.loadingFrame + fixture.readyFrame), true);
});

test('fragmented ANSI output does not match early or accept a different model', () => {
  const ready = promptAfterFor('codex');
  assert.equal(ready.test('model: loading\u001b[2Kformal-ai default'), false);
  assert.equal(ready.test('model: formal-ai-other'), false);
  for (let length = 0; length < 'formal-ai'.length; length++) {
    assert.equal(ready.test('model: \u001b[22m' + 'formal-ai'.slice(0, length)), false);
  }
  assert.equal(ready.test('model: \u001b[22mformal-ai\u001b[2m'), true);
});

function replayPromptGate(after) {
  return new Promise((resolve, reject) => {
    const executable = fileURLToPath(new URL(
      '../../../experiments/agent_cli_e2e/issue_819_tui/codex-startup-fixture.mjs', import.meta.url));
    const child = spawn(process.execPath, [executable], {stdio: ['pipe', 'pipe', 'pipe']});
    let output = '', error = '', confirmed = false, submitted = false;
    const timeout = setTimeout(() => child.kill('SIGKILL'), 6000);
    child.stdout.on('data', chunk => {
      output += chunk;
      if (output.length > 16384) {child.kill('SIGKILL'); return;}
      if (!confirmed && output.includes('Press enter to continue')) {
        confirmed = true; child.stdin.write('\r');
      }
      if (!submitted && (after instanceof RegExp ? after.test(output) : output.includes(after))) {
        submitted = true; child.stdin.write('Find archive folder on my desktop\r');
      }
    });
    child.stderr.on('data', chunk => {error = (error + chunk).slice(-4096);});
    child.once('error', reject);
    child.once('close', (status, signal) => {
      clearTimeout(timeout); resolve({status, signal, output, error});
    });
  });
}

test('the observed footer gate loses a real child-process submission during loading', async () => {
  const result = await replayPromptGate('formal-ai default');
  assert.equal(result.status, 1);
  assert.match(result.error, /Prompt arrived while model was loading/u);
  assert.equal(result.output.includes('Submitted:'), false);
});

test('selected-model readiness submits successfully through the same delayed interactive child', async () => {
  const result = await replayPromptGate(promptAfterFor('codex'));
  assert.equal(result.status, 0); assert.equal(result.signal, null);
  assert.match(result.output, /Submitted: Find archive folder on my desktop/u);
  assert.equal(result.error, '');
});
