import { expect, test } from 'bun:test';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { captureAgentTui } from 'agent-commander';

const directory = fileURLToPath(new URL('.', import.meta.url));

test('published adapter preserves ordered states and writes replay artifacts', async () => {
  const artifactDirectory = await mkdtemp(join(tmpdir(), 'formal-ai-tui-'));
  const capture = await captureAgentTui({
    tool: 'opencode',
    workingDirectory: directory,
    executable: process.execPath,
    prefixArgs: [join(directory, 'tui-fixture.mjs')],
    skipDefaultSafetyFlags: true,
    cols: 120,
    rows: 2,
    artifactDirectory,
  });

  const states = [
    'User: Find archive folder on my desktop',
    'Tool: find "$HOME/Desktop" -type d',
    'Result: /tmp/Desktop/archive',
  ];
  let previous = -1;
  for (const state of states) {
    const index = capture.transcript.indexOf(state);
    expect(index).toBeGreaterThan(previous);
    expect(capture.transcript.split(state).length - 1).toBe(1);
    previous = index;
  }
  const finalFrame = capture.frames.at(-1);
  expect(finalFrame.lines.length).toBeGreaterThan(finalFrame.screen.length);
  expect(finalFrame.screen).not.toContain(states[0]);
  expect(finalFrame.screen).toContain(states.at(-1));
  for (const artifact of [
    'transcript.txt',
    'frames.json',
    'session.cast',
    'snapshot.svg',
    'recording.svg',
  ]) {
    expect(
      (await readFile(join(artifactDirectory, artifact))).length,
    ).toBeGreaterThan(0);
  }
  await rm(artifactDirectory, { recursive: true, force: true });
});

test('published adapter sends text, control keys, and resize through its PTY', async () => {
  const capture = await captureAgentTui({
    tool: 'opencode',
    workingDirectory: directory,
    executable: process.execPath,
    prefixArgs: [join(directory, 'tui-input-fixture.mjs')],
    skipDefaultSafetyFlags: true,
    cols: 24,
    rows: 6,
    interactions: [
      {
        after: 'Choose reports',
        text: '123',
        key: 'TAB',
      },
      { key: 'ENTER' },
      { after: 'waiting-resize', resize: { cols: 40, rows: 10 } },
    ],
  });

  expect(capture.interactionCount).toBe(3);
  expect(capture.transcript).toContain('Submitted: 123');
  expect(capture.transcript).toContain('resized:40x10');
});
