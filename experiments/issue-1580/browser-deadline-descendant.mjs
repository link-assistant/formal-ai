// Bounded observation of the release deadline's actual descendant teardown.
// Supplied by Codex; this is not independent Formal AI reasoning evidence.
import { createHash } from 'node:crypto';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { runBoundedOperation } from '../../scripts/browser-engine-deadline.mjs';

const directory = mkdtempSync(join(tmpdir(), 'deadline-descendant-observation-'));
const cooperative = process.argv.includes('--reap-child');
let identifier;
function observe() {
  try {
    process.kill(identifier, 0);
  } catch (error) {
    if (error.code === 'ESRCH') return { exists: false, state: 'ESRCH' };
    throw error;
  }
  if (process.platform !== 'linux') return { exists: true, state: 'unknown' };
  const stat = readFileSync('/proc/' + identifier + '/stat', 'utf8');
  const fields = stat.slice(stat.lastIndexOf(')') + 2).split(' ');
  return { exists: true, state: fields[0], parent: Number(fields[1]) };
}
try {
  const environment = {
    ...process.env, CI: 'true', GITHUB_JOB: 'package',
    GITHUB_RUN_ID: '42', GITHUB_RUN_ATTEMPT: '1',
    GITHUB_WORKFLOW_SHA: 'a'.repeat(40), RUNNER_TEMP: directory,
  };
  const bytes = JSON.stringify({
    schema: 'browser-engine-job-clock/v1', started: Date.now() - 1193000,
    CI: environment.CI, GITHUB_JOB: environment.GITHUB_JOB,
    GITHUB_RUN_ID: environment.GITHUB_RUN_ID, GITHUB_RUN_ATTEMPT: environment.GITHUB_RUN_ATTEMPT,
    GITHUB_WORKFLOW_SHA: environment.GITHUB_WORKFLOW_SHA,
  });
  writeFileSync(join(directory, 'browser-engine-job-clock.json'), bytes);
  environment.ENGINE_CLOCK_SHA256 = createHash('sha256').update(bytes).digest('hex');
  const childFile = join(directory, 'child.json');
  const child = 'require("node:fs").writeFileSync(' + JSON.stringify(childFile) + ',JSON.stringify(process.pid));setInterval(()=>{},100);';
  const parent = 'const child=require("node:child_process").spawn(process.execPath,["-e",' + JSON.stringify(child) + '],{stdio:"inherit"});'
    + (cooperative ? 'process.on("SIGTERM",()=>{});child.on("exit",()=>process.exit(0));' : '')
    + 'setInterval(()=>{},100);';
  const status = runBoundedOperation('cold', [process.execPath, '-e', parent], { environment });
  if (!existsSync(childFile)) throw new Error('descendant fixture did not start');
  identifier = JSON.parse(readFileSync(childFile, 'utf8'));
  const observation = observe();
  console.log(JSON.stringify({
    schema: 'deadline-descendant-observation/v1', cooperativeReaper: cooperative,
    status, identifier, observation, independentAuthorshipCredit: 0,
  }));
  if (status !== 124) throw new Error('the unchanged timeout status must be 124');
  if (cooperative && observation.exists) throw new Error('cooperative fixture descendant was not reaped');
} finally {
  if (identifier !== undefined) {
    const remaining = observe();
    if (remaining.exists && remaining.state !== 'Z') {
      process.kill(identifier, 'SIGKILL');
      for (let attempt = 0; attempt < 100; attempt += 1) {
        const state = observe();
        if (!state.exists || state.state === 'Z') break;
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
      }
      const state = observe();
      if (state.exists && state.state !== 'Z') throw new Error('live descendant remains after cleanup');
    }
  }
  rmSync(directory, { recursive: true, force: true });
}
