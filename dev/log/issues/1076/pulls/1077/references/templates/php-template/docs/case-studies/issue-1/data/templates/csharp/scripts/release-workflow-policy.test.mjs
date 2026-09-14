import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';

const RELEASE_WORKFLOW = '.github/workflows/release.yml';

const EXPECTED_JOB_TIMEOUTS = new Map([
  ['detect-changes', 5],
  ['changeset-check', 10],
  ['lint', 20],
  ['test', 30],
  ['build', 20],
  ['release', 30],
  ['instant-release', 30],
  ['changeset-pr', 10],
]);

function readWorkflow(filePath) {
  return readFileSync(filePath, 'utf-8').replaceAll('\r\n', '\n');
}

function getJobBlocks(workflow) {
  const lines = workflow.split('\n');
  const jobsStart = lines.findIndex((line) => line === 'jobs:');
  if (jobsStart === -1) {
    return new Map();
  }

  const blocks = new Map();
  let currentName = '';
  let currentLines = [];

  for (const line of lines.slice(jobsStart + 1)) {
    const match = /^  ([a-zA-Z0-9_-]+):\s*$/.exec(line);
    if (match) {
      if (currentName) {
        blocks.set(currentName, currentLines.join('\n'));
      }
      currentName = match[1];
      currentLines = [line];
      continue;
    }

    if (currentName) {
      currentLines.push(line);
    }
  }

  if (currentName) {
    blocks.set(currentName, currentLines.join('\n'));
  }

  return blocks;
}

describe('release workflow policy', () => {
  test('does not cancel release runs on main when newer pushes arrive', () => {
    const workflow = readWorkflow(RELEASE_WORKFLOW);

    expect(workflow).toContain(
      'group: ${{ github.workflow }}-${{ github.ref }}'
    );
    expect(workflow).toContain(
      "cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}"
    );
    expect(workflow).not.toContain('cancel-in-progress: true');
  });

  test('sets explicit timeout-minutes on every job', () => {
    const workflow = readWorkflow(RELEASE_WORKFLOW);
    const jobBlocks = getJobBlocks(workflow);

    expect([...jobBlocks.keys()]).toEqual([...EXPECTED_JOB_TIMEOUTS.keys()]);

    for (const [jobName, timeoutMinutes] of EXPECTED_JOB_TIMEOUTS) {
      expect(jobBlocks.get(jobName)).toContain(
        `\n    timeout-minutes: ${timeoutMinutes}`
      );
    }
  });

  test('uses the current GitHub Action versions required by the template', () => {
    const workflow = readWorkflow(RELEASE_WORKFLOW);

    expect(workflow).toContain('uses: actions/checkout@v6');
    expect(workflow).toContain('uses: actions/upload-artifact@v7');
    expect(workflow).toContain('uses: peter-evans/create-pull-request@v8');
    expect(workflow).not.toContain('uses: actions/checkout@v4');
    expect(workflow).not.toContain('uses: actions/upload-artifact@v4');
    expect(workflow).not.toContain('uses: peter-evans/create-pull-request@v7');
  });
});
