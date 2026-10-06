// The JavaScript server's Telegram code execution (js/server/telegram-execution.mjs,
// js/server/execution-box.mjs) mirrors rust/src/telegram_runtime.rs and
// rust/src/execution_box/mod.rs, case for case with
// rust/tests/unit/issue_1138_telegram_execution.rs.

import { spawnSync } from 'node:child_process';
import test from 'node:test';
import assert from 'node:assert/strict';

import {
  BoxError,
  backendFromConfiguration,
  rustDebugString,
  shellWordsSplit,
} from '../../../js/server/execution-box.mjs';
import { observedEvidence, sha256Hex } from '../../../js/server/execution-evidence.mjs';
import { stableId } from '../../../js/server/ids.mjs';
import {
  TELEGRAM_EXECUTION_HARD_LIMIT_MS,
  executeTelegramCodeRequest,
  executeTelegramCodeRequestFromEnvironment,
  localizedResponse,
} from '../../../js/server/telegram-execution.mjs';

const PROMPTS = [
  ['en', 'Run this and tell me exactly what it prints: print(sum(range(1, 11)))'],
  ['ru', 'Запусти это и скажи точно, что оно печатает: print(sum(range(1, 11)))'],
  ['hi', 'इसे चलाओ और मुझे ठीक-ठीक बताओ कि यह क्या छापता है: print(sum(range(1, 11)))'],
  ['zh', '运行这个并准确告诉我它打印了什么：print(sum(range(1, 11)))'],
  ['es', 'Ejecuta esto y dime exactamente qué imprime: print(sum(range(1, 11)))'],
];

const HAS_PYTHON = !spawnSync('python3', ['-c', 'pass'], { stdio: 'ignore' }).error;
const HOST = { kind: 'host_sandbox' };

function configurationError(...args) {
  try {
    backendFromConfiguration(...args);
  } catch (error) {
    assert.ok(error instanceof BoxError);
    return error.debug();
  }
  assert.fail('configuration was accepted');
}

test('without a backend Telegram refuses honestly in five languages', async () => {
  for (const [language, prompt] of PROMPTS) {
    const outcome = await executeTelegramCodeRequest(prompt, language, null, 1000);
    assert.equal(outcome.kind, 'refused', language);
    assert.equal(outcome.answer, localizedResponse('code_execution_refused', language));
    assert.ok(!outcome.answer.includes('55'), language);
  }
});

test('an empty environment is a refusal and prose is never executed', async () => {
  const outcome = await executeTelegramCodeRequestFromEnvironment(PROMPTS[0][1], 'en', {});
  assert.equal(outcome.kind, 'refused');
  const prose = await executeTelegramCodeRequest('Explain what print(sum(range(1, 11))) means.', 'en', HOST, 1000);
  assert.deepEqual(prose, { kind: 'not_requested' });
  assert.equal(TELEGRAM_EXECUTION_HARD_LIMIT_MS, 600_000);
});

test('invalid configuration is an observed failure with the Rust Debug rendering', async () => {
  const outcome = await executeTelegramCodeRequestFromEnvironment(PROMPTS[0][1], 'en', {
    FORMAL_AI_EXECUTION_BACKEND: 'nope',
  });
  assert.equal(outcome.kind, 'failed');
  assert.equal(
    outcome.answer,
    'Execution failed in `configuration`. Nothing is reported as verified.\n\nVerbose observation:\n```text\n' +
      'InvalidConfiguration { detail: "unknown execution backend: nope" }\n```',
  );
  const silent = await executeTelegramCodeRequestFromEnvironment('hello there', 'en', {
    FORMAL_AI_EXECUTION_BACKEND: 'nope',
  });
  assert.deepEqual(silent, { kind: 'not_requested' });
});

test('backend configuration mirrors backend_from_configuration', () => {
  assert.equal(backendFromConfiguration(null, null, null), null);
  assert.equal(backendFromConfiguration(' ', '', '  '), null);
  assert.deepEqual(backendFromConfiguration(null, null, ' host_sandbox '), HOST);
  assert.deepEqual(backendFromConfiguration('docker', "$ --isolated docker -- 'a b'", 'ignored'), {
    kind: 'start_runner',
    program: '$',
    arguments: ['--isolated', 'docker', '--', 'a b'],
    isolation: 'docker',
  });
  assert.deepEqual(backendFromConfiguration(null, null, 'box:konard/box-python:2.4.0'), {
    kind: 'box',
    image: 'konard/box-python:2.4.0',
  });
  assert.deepEqual(backendFromConfiguration(null, null, 'box-language:python'), {
    kind: 'box',
    image: 'konard/box-python:2.4.0',
  });
  assert.equal(
    configurationError('docker', null, null),
    'InvalidConfiguration { detail: "start isolation and runner must be configured together" }',
  );
  assert.equal(
    configurationError('podman', '$', null),
    'InvalidConfiguration { detail: "unsupported start isolation: podman" }',
  );
  assert.equal(configurationError('docker', '# only', null), 'InvalidConfiguration { detail: "the start runner is empty" }');
  assert.equal(configurationError('docker', '"open', null), 'InvalidConfiguration { detail: "missing closing quote" }');
  assert.equal(
    configurationError(null, null, 'box:-bad'),
    'InvalidConfiguration { detail: "container image is not a safe Docker reference" }',
  );
  assert.equal(
    configurationError(null, null, 'box-language:cobol'),
    'InvalidConfiguration { detail: "no box-language contract for: cobol" }',
  );
  assert.match(configurationError(null, null, 'box-language:c'), /^InvalidConfiguration \{ detail: "no dedicated box-c image/);
});

test('shell words and Debug strings match the Rust renderings', () => {
  assert.deepEqual(shellWordsSplit('a "b \\$c \\d" e\\ f #g'), ['a', 'b $c \\d', 'e f']);
  assert.deepEqual(shellWordsSplit('x\\'), ['x\\']);
  // Observed from rustc: `{:?}` of this string.
  assert.equal(
    rustDebugString('नहीं é  x​\u007f\u001b\0\'"\\\t'),
    '"नही\\u{902} e\\u{301} \\u{a0}x\\u{200b}\\u{7f}\\u{1b}\\0\'\\"\\\\\\t"',
  );
});

test('evidence ids are the FNV stable id of the Rust fingerprint', () => {
  const evidence = observedEvidence('python3 x.py', ['python3', 'x.py'], 0, Buffer.from('55\n'), 'command_exit', 'local_process');
  const sha = sha256Hex(Buffer.from('55\n'));
  assert.equal(
    evidence.evidence_id,
    stableId('evidence', ['python3 x.py', 'python3\u001ex.py', '0', sha, '3', 'command_exit', 'local_process'].join('\u001f')),
  );
});

test('a host sandbox reports observed output and evidence', { skip: !HAS_PYTHON }, async () => {
  for (const [language, prompt] of PROMPTS) {
    const outcome = await executeTelegramCodeRequest(prompt, language, HOST, 5000);
    assert.equal(outcome.kind, 'observed', `${language}: ${outcome.answer}`);
    assert.ok(outcome.answer.includes('```text\n55\n```'), language);
    assert.equal(outcome.evidence.exit_code, 0);
    assert.equal(outcome.evidence.for_need, 'telegram_code_execution');
    assert.equal(outcome.evidence.produced_by, 'telegram_execution_box');
    assert.ok(outcome.answer.includes(outcome.evidence.evidence_id));
    assert.equal(outcome.ladder, null);
  }
  const outcome = await executeTelegramCodeRequestFromEnvironment('run this: print(1+1)', 'en', {
    FORMAL_AI_EXECUTION_BACKEND: 'host_sandbox',
  });
  assert.equal(outcome.kind, 'observed');
  assert.match(outcome.answer, /^Execution status: observed in `host_sandbox` \(exit 0, \d+ ms; deadline 60000 ms\)\.\n\nOutput:\n```text\n2\n```\n\nEvidence: `evidence_[0-9a-f]{16}`$/);
});

test('a failing program and a timeout are failures with the observed log', { skip: !HAS_PYTHON }, async () => {
  const failing = await executeTelegramCodeRequest('run this: print(1); raise SystemExit(3)', 'en', HOST, 5000);
  assert.equal(failing.kind, 'failed');
  assert.match(failing.answer, /```text\ntimed_out=false elapsed_ms=\d+ deadline_ms=5000 exit=Some\(3\)\n1\n\n```$/);
  const slow = await executeTelegramCodeRequest(
    'run this: import time, sys; print("started", flush=True); time.sleep(30)',
    'en',
    HOST,
    100,
  );
  assert.equal(slow.kind, 'failed');
  assert.match(slow.answer, /timed_out=true elapsed_ms=\d+ deadline_ms=100 exit=None\nstarted\n/);
});

test('a start runner receives the script on stdin', { skip: !HAS_PYTHON }, async () => {
  const outcome = await executeTelegramCodeRequestFromEnvironment('run this: print(6*7)', 'en', {
    FORMAL_AI_START_ISOLATION: 'docker',
    FORMAL_AI_START_RUNNER: 'env',
  });
  assert.equal(outcome.kind, 'observed', outcome.answer);
  assert.ok(outcome.answer.includes('`start_runner:docker`'));
  assert.ok(outcome.answer.includes('```text\n42\n```'));
  assert.deepEqual(outcome.evidence.argv, ['env', 'python3', '-']);
});

test('a missing runner program is a BoxError::Observed failure', async () => {
  const outcome = await executeTelegramCodeRequestFromEnvironment('run this: print(1)', 'en', {
    FORMAL_AI_START_ISOLATION: 'docker',
    FORMAL_AI_START_RUNNER: 'formal-ai-no-such-runner',
  });
  assert.equal(outcome.kind, 'failed');
  assert.ok(outcome.answer.includes('Observed { detail: "No such file or directory (os error 2)" }'), outcome.answer);
});

test('the {N} ladder records each rung before the plain run', { skip: !HAS_PYTHON }, async () => {
  const outcome = await executeTelegramCodeRequest('run this with 8: print("{N}")', 'en', HOST, 5000);
  assert.equal(outcome.kind, 'observed', outcome.answer);
  assert.equal(outcome.ladder.rungs.length, 1);
  assert.equal(outcome.ladder.rungs[0].n, 8n);
  assert.match(outcome.ladder.verbose_log, /^N=8 timed_out=false elapsed_ms=\d+ deadline_ms=5000 exit=Some\(0\) output="8\\n"\n$/);
  // As natively, the unsubstituted source then runs once more and is what the answer shows.
  assert.ok(outcome.answer.includes('```text\n{N}\n```'));
});
