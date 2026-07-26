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

export function renderedMarkerOccurrences(frame, marker) {
  const count = (value) => value.split(marker).length - 1;
  return Math.max(count(frame), count(frame.replace(/\n/gu, '')));
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
  interactions = [],
  stopMarker,
  stopMarkerOccurrences = 1,
  outputPath,
  timeoutMs = 90_000,
}) {
  const abortController = new AbortController();
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
    signal: abortController.signal,
  })`script -qefc ${command} /dev/null`;
  const frames = [];
  const seenFrames = new Set();
  const pendingInteractions = [...interactions];
  let raw = '';
  let stopMarkerSeen = false;
  let timedOut = false;
  const timeoutSentinel = Symbol('TUI capture timeout');
  let timeout;
  const deadline = new Promise((resolve) => {
    timeout = setTimeout(() => {
      timedOut = true;
      abortController.abort();
      resolve(timeoutSentinel);
    }, timeoutMs);
  });
  const beforeDeadline = (promise) => Promise.race([promise, deadline]);
  const stream = runner.stream()[Symbol.asyncIterator]();
  let stdin = null;

  try {
    if (pendingInteractions.length > 0) {
      stdin = await beforeDeadline(Promise.resolve(runner.streams.stdin));
    }
    while (!timedOut) {
      const next = await beforeDeadline(stream.next());
      if (next === timeoutSentinel || next.done) break;
      const chunk = next.value;
      if (chunk.type === 'exit') break;
      const text = chunk.data.toString();
      raw += text;
      const rendered = await beforeDeadline(writeTerminal(terminal, text));
      if (rendered === timeoutSentinel) break;
      const frame = terminalSnapshot(terminal);
      if (frame && !seenFrames.has(frame)) {
        seenFrames.add(frame);
        frames.push(frame);
      }
      stopMarkerSeen =
        stopMarkerSeen ||
        (stopMarker &&
          renderedMarkerOccurrences(frame, stopMarker) >=
            stopMarkerOccurrences);
      while (
        stdin &&
        pendingInteractions.length > 0 &&
        raw.includes(pendingInteractions[0].after)
      ) {
        const interaction = pendingInteractions.shift();
        for (const input of interaction.inputs) {
          stdin.write(input);
          if (interaction.delayMs) {
            const delayed = await beforeDeadline(
              new Promise((resolve) =>
                setTimeout(resolve, interaction.delayMs),
              ),
            );
            if (delayed === timeoutSentinel) break;
          }
        }
      }
      if (stopMarkerSeen) {
        break;
      }
    }
  } finally {
    clearTimeout(timeout);
    if (!abortController.signal.aborted) abortController.abort();
    try {
      await beforeDeadline(stream.return?.() ?? Promise.resolve());
    } finally {
      terminal.dispose();
    }
  }

  const transcript = {
    command,
    frame_count: frames.length,
    frames,
    sequence: unrollFrames(frames),
    interaction_count: interactions.length - pendingInteractions.length,
    stop_marker_seen: !stopMarker || stopMarkerSeen,
    timed_out: timedOut,
  };
  if (outputPath) {
    await writeFile(outputPath, `${JSON.stringify(transcript, null, 2)}\n`);
  }
  if (timedOut) {
    throw new Error(`TUI capture timed out after ${timeoutMs}ms`);
  }
  return transcript;
}
