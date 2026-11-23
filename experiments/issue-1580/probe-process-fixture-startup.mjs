// Replay the real Node-to-Python launch topology with finite, bounded fixtures.
import {spawnSync} from 'node:child_process';
const wrapper = process.argv[2];
if (!wrapper) throw new Error('a wrapper is required');
for (const mode of ['original', 'no-startup', 'clean-home', 'no-stdin']) {
  const start = performance.now();
  const observed = spawnSync('python3', [
    'experiments/issue-1580/probe-process-fixture-startup.py', wrapper, mode,
  ], {encoding: 'utf8', maxBuffer: 65536});
  console.log(JSON.stringify({mode, milliseconds: performance.now() - start,
    status: observed.status, signal: observed.signal,
    stdout: observed.stdout, stderr: observed.stderr, error: String(observed.error ?? '')}));
}
