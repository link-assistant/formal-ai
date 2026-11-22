import assert from 'node:assert/strict';
import test from 'node:test';
import { startCli } from '../rust/tests/web/support/structured-log-fixture.mjs';

const binary = process.env.FORMAL_AI_TEST_NATIVE_CLI;
assert.ok(binary, 'Set FORMAL_AI_TEST_NATIVE_CLI to the native executable');

test('inspect native program code and clarification replies', async (t) => {
  const server = await startCli(t, { binary, lifetime: 30_000, env: { FORMAL_AI_OFFLINE: '1' } });
  await server.ready();
  const prompts = [
    'Write a Bash script',
    'Write a Python script',
    "Write a Python script that prints 'rust'",
    'Write a Python program that copies stdin to stdout',
    'Write a Python program that counts from -6 to -4',
    'Write a Python program that implements FizzBuzz from 14 to 16',
    'Write a Python program that calculates the factorial of seven',
    'Write a Python program that will reverse the string «cat 🧩»',
    'Write a Python program that calculates the sum from 12 through 15',
    'Write a Python program that calculates the 13th Fibonacci number',
    'Write a Python program that calculates the factorial',
    'Write a Python program that implements FizzBuzz',
    'Write a Python program that reverses a string',
  ];
  for (const prompt of prompts) {
    const response = await fetch(`${server.url}/v1/chat/completions`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      signal: AbortSignal.timeout(2_000),
      body: JSON.stringify({ model: 'formal-ai', messages: [{ role: 'user', content: prompt }] }),
    });
    assert.ok(response.ok, await response.clone().text());
    const { choices } = await response.json();
    console.log(JSON.stringify({ prompt, reply: choices[0].message.content }));
  }
});
