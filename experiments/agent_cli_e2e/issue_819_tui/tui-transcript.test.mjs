import { expect, test } from 'bun:test';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  captureTuiTranscript,
  renderedMarkerOccurrences,
} from './tui-transcript.mjs';

const directory = fileURLToPath(new URL('.', import.meta.url));

test('command-stream renders, deduplicates, and unrolls complete TUI frames', async () => {
  const transcript = await captureTuiTranscript({
    command: 'node tui-fixture.mjs',
    cwd: directory,
    stopMarker: '/tmp/Desktop/archive',
  });

  expect(transcript.frame_count).toBeGreaterThan(0);
  expect(transcript.stop_marker_seen).toBe(true);
  expect(transcript.sequence).toContain('User: Find archive folder on my desktop');
  expect(transcript.sequence).toContain('Tool: find "$HOME/Desktop" -type d');
  expect(transcript.sequence).toContain('Result: /tmp/Desktop/archive');
  expect(
    transcript.sequence.filter(
      (line) => line === 'User: Find archive folder on my desktop',
    ),
  ).toHaveLength(1);
  expect(
    transcript.sequence.indexOf('User: Find archive folder on my desktop'),
  ).toBeLessThan(
    transcript.sequence.indexOf('Tool: find "$HOME/Desktop" -type d'),
  );
  expect(
    transcript.sequence.indexOf('Tool: find "$HOME/Desktop" -type d'),
  ).toBeLessThan(transcript.sequence.indexOf('Result: /tmp/Desktop/archive'));
});

test('command-stream sends scheduled input through the PTY', async () => {
  const transcript = await captureTuiTranscript({
    command: 'node tui-input-fixture.mjs',
    cwd: directory,
    interactions: [
      {
        after: 'Choose reports',
        inputs: ['1', '2', '3', '\t', '\r'],
        delayMs: 5,
      },
    ],
    stopMarker: 'Submitted: 123',
  });

  expect(transcript.interaction_count).toBe(1);
  expect(transcript.sequence).toContain('Submitted: 123');
});

test('rendered marker counting survives a hard-wrapped final result', () => {
  const marker = '/tmp/Desktop/Archive/hive-control-center';
  const frame = [
    `Tool: ${marker}`,
    'Final: closest matching name is at /tmp/Desktop/Archive/hive-control-',
    'center',
  ].join('\n');

  expect(renderedMarkerOccurrences(frame, marker)).toBe(2);
});

test('capture timeout terminates the PTY and preserves partial evidence', async () => {
  const artifactDirectory = await mkdtemp(join(tmpdir(), 'formal-ai-tui-'));
  const outputPath = join(artifactDirectory, 'transcript.json');
  const startedAt = performance.now();

  try {
    await expect(
      captureTuiTranscript({
        command: 'node tui-timeout-fixture.mjs',
        cwd: directory,
        stopMarker: 'result arrived',
        outputPath,
        timeoutMs: 500,
      }),
    ).rejects.toThrow('TUI capture timed out after 500ms');
    expect(performance.now() - startedAt).toBeLessThan(2_000);

    const transcript = JSON.parse(await readFile(outputPath, 'utf8'));
    expect(transcript.timed_out).toBe(true);
    expect(transcript.stop_marker_seen).toBe(false);
    expect(transcript.sequence).toContain(
      'Waiting for a result that never arrives...',
    );
  } finally {
    await rm(artifactDirectory, { recursive: true, force: true });
  }
});
