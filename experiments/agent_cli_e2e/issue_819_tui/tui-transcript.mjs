import { writeFile } from 'node:fs/promises';

import xterm from '@xterm/headless';
import { $ } from 'command-stream';

const { Terminal } = xterm;

function terminalSnapshot(terminal) {
  const buffer = terminal.buffer.active;
  const lines = [];
  for (let index = 0; index < buffer.length; index += 1) {
    const line = buffer.getLine(index)?.translateToString(true).trimEnd() ?? '';
    lines.push(line);
  }
  while (lines.at(-1) === '') lines.pop();
  return lines.join('\n');
}

function writeTerminal(terminal, chunk) {
  return new Promise((resolve) => terminal.write(chunk, resolve));
}

export function unrollFrames(frames) {
  const seen = new Set();
  const sequence = [];
  for (const frame of frames) {
    for (const line of frame.split('\n')) {
      const normalized = line
        .replace(/[│┃╭╮╰╯─━┌┐└┘┆┊]+/gu, ' ')
        .replace(/\s+/gu, ' ')
        .trim();
      if (normalized && !seen.has(normalized)) {
        seen.add(normalized);
        sequence.push(normalized);
      }
    }
  }
  return sequence;
}

/**
 * Stream an actual TUI through a PTY, render every output chunk, and retain
 * only distinct terminal frames. `command` is passed as one safely quoted
 * `script -c` argument; command-stream never evaluates it as template syntax.
 */
export async function captureTuiTranscript({
  command,
  cwd,
  environment = {},
  stopMarker,
  stopMarkerOccurrences = 1,
  outputPath,
  timeoutMs = 90_000,
}) {
  const terminal = new Terminal({
    allowProposedApi: true,
    cols: 120,
    rows: 40,
    scrollback: 4000,
  });
  const runner = $({
    cwd,
    env: { ...process.env, ...environment, TERM: 'xterm-256color' },
    mirror: false,
    capture: true,
  })`script -qefc ${command} /dev/null`;
  const frames = [];
  const seenFrames = new Set();
  let raw = '';
  const timeout = setTimeout(() => runner.kill('SIGTERM'), timeoutMs);

  try {
    for await (const chunk of runner.stream()) {
      if (chunk.type === 'exit') break;
      const text = chunk.data.toString();
      raw += text;
      await writeTerminal(terminal, text);
      const frame = terminalSnapshot(terminal);
      if (frame && !seenFrames.has(frame)) {
        seenFrames.add(frame);
        frames.push(frame);
      }
      if (
        stopMarker &&
        raw.split(stopMarker).length - 1 >= stopMarkerOccurrences
      ) {
        runner.kill('SIGTERM');
        break;
      }
    }
  } finally {
    clearTimeout(timeout);
  }
  terminal.dispose();

  const transcript = {
    command,
    frame_count: frames.length,
    frames,
    sequence: unrollFrames(frames),
    stop_marker_seen:
      !stopMarker || raw.split(stopMarker).length - 1 >= stopMarkerOccurrences,
  };
  if (outputPath) {
    await writeFile(outputPath, `${JSON.stringify(transcript, null, 2)}\n`);
  }
  return transcript;
}
