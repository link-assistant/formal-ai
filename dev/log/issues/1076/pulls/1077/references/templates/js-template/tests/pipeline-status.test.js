import { describe, it, expect } from 'test-anywhere';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { fileURLToPath, URL } from 'node:url';

const workflow = readFileSync(
  '.github/workflows/release.yml',
  'utf8'
).replaceAll('\r\n', '\n');
const scriptPath = fileURLToPath(
  new URL('../scripts/check-pipeline-status.sh', import.meta.url)
);
const canRunBash =
  typeof Deno === 'undefined' &&
  typeof process !== 'undefined' &&
  process.platform !== 'win32';

function listWorkflowJobs(source) {
  const jobsStart = source.indexOf('\njobs:\n');
  const jobsBody = jobsStart === -1 ? '' : source.slice(jobsStart);

  return Array.from(
    jobsBody.matchAll(/^[ ]{2}([a-zA-Z0-9_-]+):\s*$/gm),
    (match) => match[1]
  );
}

function getJobBlock(source, jobName) {
  const lines = source.split('\n');
  const start = lines.findIndex((line) => line === `  ${jobName}:`);

  if (start === -1) {
    return '';
  }

  const end = lines.findIndex(
    (line, index) => index > start && /^[ ]{2}[a-zA-Z0-9_-]+:\s*$/.test(line)
  );

  return lines.slice(start, end === -1 ? lines.length : end).join('\n');
}

function listNeededJobs(jobBlock) {
  const lines = jobBlock.split('\n');
  const needsStart = lines.findIndex((line) => line === '    needs:');

  if (needsStart === -1) {
    return [];
  }

  const neededJobs = [];

  for (const line of lines.slice(needsStart + 1)) {
    const item = line.match(/^[ ]{6}- ([a-zA-Z0-9_-]+)\s*$/);

    if (!item) {
      break;
    }

    neededJobs.push(item[1]);
  }

  return neededJobs;
}

function runGate(needs, isMain = false) {
  return spawnSync('bash', [scriptPath], {
    encoding: 'utf8',
    env: {
      ...process.env,
      NEEDS_JSON: JSON.stringify(needs),
      IS_MAIN: String(isMain),
    },
  });
}

describe('pipeline status gate', () => {
  it('observes every other release workflow job', () => {
    const jobs = listWorkflowJobs(workflow);
    const gate = getJobBlock(workflow, 'pipeline-status');

    expect(gate).toContain('    if: always()');
    expect(gate).toContain('uses: actions/setup-node@v6');
    expect(gate).toContain('NEEDS_JSON: ${{ toJSON(needs) }}');
    expect(gate).toContain(
      "IS_MAIN: ${{ github.ref == 'refs/heads/main' && github.event_name == 'push' }}"
    );
    expect(listNeededJobs(gate).sort()).toEqual(
      jobs.filter((job) => job !== 'pipeline-status').sort()
    );
  });

  it('does not cancel superseded jobs on main', () => {
    expect(workflow).not.toContain('      cancel-in-progress: true');
    expect(workflow).toContain(
      "      cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}"
    );
  });

  if (canRunBash) {
    it('passes when jobs succeeded or were skipped', () => {
      const result = runGate({
        lint: { result: 'success' },
        release: { result: 'skipped' },
      });

      expect(result.status).toBe(0);
      expect(result.stdout).toContain('Failed jobs:    <none>');
      expect(result.stdout).toContain('Cancelled jobs: <none>');
    });

    it('fails for a failed job on every ref', () => {
      const result = runGate({ lint: { result: 'failure' } });

      expect(result.status).toBe(1);
      expect(result.stdout).toContain('Pipeline failed. Failing jobs: lint');
    });

    it('fails for a cancelled job on main', () => {
      const result = runGate({ release: { result: 'cancelled' } }, true);

      expect(result.status).toBe(1);
      expect(result.stdout).toContain(
        'Pipeline has cancelled jobs on main: release'
      );
    });

    it('warns for a cancelled job on a non-default ref', () => {
      const result = runGate({ test: { result: 'cancelled' } });

      expect(result.status).toBe(0);
      expect(result.stdout).toContain('::warning::Cancelled jobs: test');
    });
  }
});
