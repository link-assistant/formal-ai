import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {promptAfterFor, promptInteractionsFor} from '../../../experiments/agent_cli_e2e/issue_819_tui/client-contract.mjs';

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

test('submission waits for the actual cursor-separated draft and treats prompt punctuation literally', () => {
  const submit = promptInteractionsFor('codex', fixture.prompt)[1];
  assert.equal(submit.after.test(fixture.loadingFrame + fixture.readyFrame), false);
  assert.equal(submit.after.test(fixture.draftFrame), true);
  const punctuated = 'Print (a+b) [exact]? $5.0';
  assert.equal(promptInteractionsFor('codex', punctuated)[1].after.test(punctuated), true);
  assert.equal(promptInteractionsFor('codex', punctuated)[1].after.test('Print aaab exact 50'), false);
  assert.deepEqual(promptInteractionsFor('claude', 'hello'), [{after: '$', text: 'hello', key: 'ENTER'}]);
});

function replayPromptGate(after, submission = null) {
  return new Promise((resolve, reject) => {
    const executable = fileURLToPath(new URL(
      '../../../experiments/agent_cli_e2e/issue_819_tui/codex-startup-fixture.mjs', import.meta.url));
    const child = spawn(process.execPath, [executable], {stdio: ['pipe', 'pipe', 'pipe']});
    let output = '', error = '', confirmed = false, submitted = false, submitTimer;
    const timeout = setTimeout(() => child.kill('SIGKILL'), 6000);
    child.stdout.on('data', chunk => {
      output += chunk;
      if (output.length > 16384) {child.kill('SIGKILL'); return;}
      if (!confirmed && output.includes('Press enter to continue')) {
        confirmed = true; child.stdin.write('\r');
      }
      if (!submitted && (after instanceof RegExp ? after.test(output) : output.includes(after))) {
        submitted = true;
        child.stdin.write('Find archive folder on my desktop' + (submission ? '' : '\r'));
      }
      if (submitted && submission && !submitTimer && submission.after.test(output)) {
        submitTimer = setTimeout(() => child.stdin.write('\r'), submission.idleMilliseconds);
      }
    });
    child.stderr.on('data', chunk => {error = (error + chunk).slice(-4096);});
    child.stdin.on('error', error => {if (error.code !== 'EPIPE') reject(error);});
    child.once('error', reject);
    child.once('close', (status, signal) => {
      clearTimeout(timeout); clearTimeout(submitTimer); resolve({status, signal, output, error});
    });
  });
}

test('the observed footer gate loses a real child-process submission during loading', async () => {
  const result = await replayPromptGate('formal-ai default');
  assert.equal(result.status, 1);
  assert.match(result.error, /Prompt arrived while model was loading/u);
  assert.equal(result.output.includes('Submitted:'), false);
});

test('model readiness alone cannot submit Enter inside the pinned client paste-burst window', async () => {
  const result = await replayPromptGate(promptAfterFor('codex'));
  assert.equal(result.status, 1);
  assert.match(result.error, /Enter arrived inside the paste-burst window/u);
});

test('selected-model readiness and separate settled Enter submit successfully through the same child', async () => {
  const interactions = promptInteractionsFor('codex', 'Find archive folder on my desktop');
  assert.equal(interactions.length, 2); assert.equal(interactions[0].key, undefined);
  assert.equal(interactions[1].text, undefined); assert.equal(interactions[1].key, 'ENTER');
  assert.ok(interactions[1].idleMilliseconds > fixture.pasteEnterSuppressionMilliseconds);
  const result = await replayPromptGate(interactions[0].after, interactions[1]);
  assert.equal(result.status, 0); assert.equal(result.signal, null);
  assert.match(result.output, /Submitted: Find archive folder on my desktop/u);
  assert.equal(result.error, '');
});
