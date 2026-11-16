// Issue #1562: list the completed tool calls of a hive-mind solve.log
// (Agent CLI JSON events, one field per log line). Usage:
// node experiments/issue-1562/tool-calls-of-log.mjs <solve.log>
import { readFileSync } from 'node:fs';

const lines = readFileSync(process.argv[2], 'utf8').split('\n').map(line => line.replace(/^\[[^\]]+\] \[[A-Z]+\] ?/u, ''));
let buffer = null;
for (const line of lines) {
  if (line === '{') { buffer = [line]; continue; }
  if (!buffer) continue;
  buffer.push(line);
  if (line !== '}') continue;
  let event = null;
  try { event = JSON.parse(buffer.join('\n')); } catch { event = null; }
  buffer = null;
  if (event?.type !== 'tool_use' || event.part?.state?.status !== 'completed' && event.part?.state?.status !== 'error') continue;
  const { tool, state } = event.part;
  console.log(`- ${tool} ${JSON.stringify(state.input).slice(0, 400)}`);
  console.log(`  => ${state.status} ${JSON.stringify(state.output ?? state.error ?? '').slice(0, 300)}`);
}
