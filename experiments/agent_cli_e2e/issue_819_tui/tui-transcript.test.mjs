import { expect, test } from 'bun:test';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { captureAgentTui } from 'agent-commander';

import {
  promptAfterFor,
  startupInteractionsFor,
} from './client-contract.mjs';

const directory = fileURLToPath(new URL('.', import.meta.url));

test('Claude confirms only the remaining bypass-permissions prompt', () => {
  expect(startupInteractionsFor('claude')).toEqual([
    { after: 'Enter y/n:', text: 'y', key: 'ENTER' },
  ]);
});

test('Claude waits for the current interactive input prompt', () => {
  expect(promptAfterFor('claude')).toBe('$');
});

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

test('published adapter defaults to a faithful publishable replay bundle', async () => {
  const artifactDirectory = await mkdtemp(join(tmpdir(), 'formal-ai-tui-'));
  try {
    const capture = await captureAgentTui({
      tool: 'opencode',
      workingDirectory: directory,
      executable: process.execPath,
      prefixArgs: [join(directory, 'tui-fixture.mjs')],
      skipDefaultSafetyFlags: true,
      artifactDirectory,
      artifactOptions: { borderRadius: 0 },
    });

    expect(capture.asciicast.header.width).toBe(80);
    expect(capture.asciicast.header.height).toBe(30);
    const recording = await readFile(
      join(artifactDirectory, 'recording.svg'),
      'utf8',
    );
    expect(recording).toContain('rx="0"');
    expect(recording).toContain('xml:space="preserve"');
    expect(recording).toContain('textLength=');
    expect(recording).toContain('lengthAdjust="spacingAndGlyphs"');
    expect(recording).toContain('@font-face');
    expect(recording).toContain('data:font/woff2;base64,');
    expect(recording).toContain('@keyframes');
    expect(recording).toContain('steps(1, end)');
    expect(recording).not.toContain('<animate');
    for (const svg of [
      recording,
      await readFile(join(artifactDirectory, 'snapshot.svg'), 'utf8'),
    ]) {
      const textRuns = [...svg.matchAll(/<text\b[^>]*>([^<]*)<\/text>/gu)];
      expect(textRuns.length).toBeGreaterThan(0);
      for (const [, text] of textRuns) {
        expect(text).not.toMatch(/^\s|\s$/u);
      }
    }
    expect(
      (await readFile(join(artifactDirectory, 'recording.gif')))
        .subarray(0, 6)
        .toString(),
    ).toBe('GIF89a');
  } finally {
    await rm(artifactDirectory, { recursive: true, force: true });
  }
});

test('published adapter preserves partial replay artifacts after timeout', async () => {
  const artifactDirectory = await mkdtemp(join(tmpdir(), 'formal-ai-tui-'));
  try {
    let failure;
    try {
      await captureAgentTui({
        tool: 'opencode',
        workingDirectory: directory,
        executable: process.execPath,
        prefixArgs: [join(directory, 'tui-timeout-fixture.mjs')],
        skipDefaultSafetyFlags: true,
        timeoutMilliseconds: 250,
        artifactDirectory,
      });
    } catch (error) {
      failure = error;
    }

    expect(failure?.message).toContain('timed out');
    expect(failure?.capture.transcript).toContain(
      'Waiting for a result that never arrives...',
    );
    expect(
      (await readFile(join(artifactDirectory, 'recording.svg'))).length,
    ).toBeGreaterThan(0);
  } finally {
    await rm(artifactDirectory, { recursive: true, force: true });
  }
});
