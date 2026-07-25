import { expect, test } from 'bun:test';
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

  expect(transcript.frame_count).toBe(3);
  expect(transcript.sequence).toContain('User: Find archive folder on my desktop');
  expect(transcript.sequence).toContain('Tool: find "$HOME/Desktop" -type d');
  expect(transcript.sequence).toContain('Result: /tmp/Desktop/archive');
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
