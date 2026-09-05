import { describe, it, expect } from 'test-anywhere';
import { readFileSync } from 'node:fs';

const releaseWorkflow = readWorkflow('.github/workflows/release.yml');
const linksWorkflow = readWorkflow('.github/workflows/links.yml');
const packageJson = JSON.parse(readFileSync('package.json', 'utf8'));

function readWorkflow(filePath) {
  return readFileSync(filePath, 'utf8').replaceAll('\r\n', '\n');
}

function normalizeNewlines(text) {
  return text.replaceAll('\r\n', '\n');
}

function listWorkflowJobs(workflow) {
  const normalizedWorkflow = normalizeNewlines(workflow);
  const jobsStart = normalizedWorkflow.indexOf('\njobs:\n');
  const jobsBody = jobsStart === -1 ? '' : normalizedWorkflow.slice(jobsStart);
  const matches = jobsBody.matchAll(/^[ ]{2}([a-zA-Z0-9_-]+):\s*$/gm);

  return Array.from(matches, (match) => match[1]);
}

function getJobBlock(workflow, jobName) {
  const lines = normalizeNewlines(workflow).split('\n');
  const jobHeader = `  ${jobName}:`;
  const start = lines.findIndex((line) => line === jobHeader);

  if (start === -1) {
    return '';
  }

  const end = lines.findIndex(
    (line, index) => index > start && /^[ ]{2}[a-zA-Z0-9_-]+:\s*$/.test(line)
  );

  return lines.slice(start, end === -1 ? lines.length : end).join('\n');
}

function getTimeoutMinutes(workflow, jobName) {
  const block = getJobBlock(workflow, jobName);
  const timeout = block.match(/^[ ]{4}timeout-minutes:\s*(\d+)\s*$/m);

  return timeout ? Number(timeout[1]) : undefined;
}

function getStepBlock(workflow, stepName) {
  const lines = normalizeNewlines(workflow).split('\n');
  const start = lines.findIndex(
    (line) => line.trim() === `- name: ${stepName}`
  );

  if (start === -1) {
    return '';
  }

  const end = lines.findIndex(
    (line, index) => index > start && /^\s*- (name|uses|run):/.test(line)
  );

  return lines.slice(start, end === -1 ? lines.length : end).join('\n');
}

// A step declares its deadline either by wrapping a command in
// run-with-budget-warning.sh or, for `uses:` steps that cannot be wrapped,
// with a step-level timeout-minutes.
function getStepBudgetSeconds(workflow, jobName) {
  const block = getJobBlock(workflow, jobName);
  const wrapped = Array.from(
    block.matchAll(/run-with-budget-warning\.sh\s+(\d+)\s+"([^"]+)"/g),
    (match) => ({ label: match[2], seconds: Number(match[1]) })
  );
  const stepTimeouts = Array.from(
    block.matchAll(/^[ ]{8}timeout-minutes:\s*(\d+)\s*$/gm),
    (match) => ({
      label: 'step timeout-minutes',
      seconds: Number(match[1]) * 60,
    })
  );

  return [...wrapped, ...stepTimeouts];
}

describe('CI timeout policy', () => {
  it('sets timeout-minutes for every release workflow job', () => {
    const expectedTimeouts = {
      'detect-changes': 5,
      'test-compilation': 5,
      'check-file-line-limits': 5,
      'version-check': 5,
      'changeset-check': 10,
      lint: 10,
      test: 15,
      'validate-docs': 5,
      release: 30,
      'instant-release': 30,
      'docker-build': 30,
      'docker-publish-config': 10,
      'docker-publish-build': 30,
      'docker-publish': 30,
      'changeset-pr': 10,
      'pipeline-status': 5,
    };

    expect(listWorkflowJobs(releaseWorkflow).sort()).toEqual(
      Object.keys(expectedTimeouts).sort()
    );

    for (const [jobName, timeout] of Object.entries(expectedTimeouts)) {
      expect(getTimeoutMinutes(releaseWorkflow, jobName)).toBe(timeout);
    }
  });

  it('sets timeout-minutes for every link workflow job', () => {
    expect(listWorkflowJobs(linksWorkflow)).toEqual(['link-checker']);
    expect(getTimeoutMinutes(linksWorkflow, 'link-checker')).toBe(10);
  });

  it('parses workflow files checked out with Windows line endings', () => {
    const crlfWorkflow = [
      'name: CRLF fixture',
      '',
      'jobs:',
      '  first-job:',
      '    timeout-minutes: 5',
      '  second-job:',
      '    timeout-minutes: 10',
      '',
    ].join('\r\n');

    expect(listWorkflowJobs(crlfWorkflow)).toEqual(['first-job', 'second-job']);
    expect(getTimeoutMinutes(crlfWorkflow, 'second-job')).toBe(10);
  });

  it('caps individual Node.js and Bun tests at 30 seconds', () => {
    expect(packageJson.scripts.test).toBe(
      'node --test --test-timeout=30000 tests/*.test.js'
    );
    expect(releaseWorkflow).toContain('bun test --timeout 30000');
  });

  it('documents that the Bun flag bounds a test and not the suite', () => {
    const bunStep = getStepBlock(releaseWorkflow, 'Run tests (Bun)');

    expect(bunStep).toContain('per-test bound and does not bound the suite');
  });
});

// `timeout-minutes` on a job is a backstop, never the deadline: GitHub reports
// a job it kills as *cancelled*, not *failed*. Every long step therefore
// declares its own budget, and every budget must expire with room to spare
// before the backstop fires.
const MAX_BUDGET_SHARE_PERCENT = 70;

describe('CI execution budgets', () => {
  it('keeps every declared step budget under the job backstop', () => {
    const jobsWithBudgets = listWorkflowJobs(releaseWorkflow)
      .map((jobName) => ({
        jobName,
        budgets: getStepBudgetSeconds(releaseWorkflow, jobName),
        backstop: getTimeoutMinutes(releaseWorkflow, jobName),
      }))
      .filter((job) => job.budgets.length > 0);

    expect(jobsWithBudgets.map((job) => job.jobName).sort()).toEqual([
      'docker-build',
      'docker-publish-build',
      'release',
      'test',
    ]);

    const violations = [];

    for (const { jobName, budgets, backstop } of jobsWithBudgets) {
      expect(typeof backstop).toBe('number');

      const allowedSeconds = Math.floor(
        (backstop * 60 * MAX_BUDGET_SHARE_PERCENT) / 100
      );
      const totalSeconds = budgets.reduce(
        (sum, budget) => sum + budget.seconds,
        0
      );

      for (const budget of budgets) {
        if (budget.seconds > allowedSeconds) {
          violations.push(
            `${jobName}: "${budget.label}" budget ${budget.seconds}s exceeds ${allowedSeconds}s (${MAX_BUDGET_SHARE_PERCENT}% of the ${backstop}min backstop)`
          );
        }
      }

      if (totalSeconds > allowedSeconds) {
        violations.push(
          `${jobName}: budgets total ${totalSeconds}s, exceeding ${allowedSeconds}s (${MAX_BUDGET_SHARE_PERCENT}% of the ${backstop}min backstop)`
        );
      }
    }

    expect(violations).toEqual([]);
  });

  it('warns at the same share of the budget that the invariant allows', () => {
    const budgetScript = readWorkflow('scripts/run-with-budget-warning.sh');

    expect(budgetScript).toContain(
      `BUDGET_WARN_PERCENT:-${MAX_BUDGET_SHARE_PERCENT}`
    );
  });
});
