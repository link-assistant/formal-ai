import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'test-anywhere';

function readWorkflow(filePath) {
  return readFileSync(filePath, 'utf8').replaceAll('\r\n', '\n');
}

const workflowsWorkflow = readWorkflow('.github/workflows/workflows.yml');
const releaseWorkflow = readWorkflow('.github/workflows/release.yml');
const exampleAppWorkflow = readWorkflow('.github/workflows/example-app.yml');
const zizmorConfig = readFileSync('.github/zizmor.yml', 'utf8');

describe('workflow linting job', () => {
  // This job is what reports the shellcheck findings asserted below.
  it('runs actionlint from the Docker image that bundles shellcheck', () => {
    expect(workflowsWorkflow).toContain('uses: docker://rhysd/actionlint:');
    expect(workflowsWorkflow).toContain('shellcheck');
  });

  it('runs zizmor against the repository configuration', () => {
    expect(workflowsWorkflow).toContain('uses: zizmorcore/zizmor-action@');
    expect(workflowsWorkflow).toContain('config: .github/zizmor.yml');
    expect(zizmorConfig).toContain('unpinned-uses');
  });

  it('bounds both jobs with a timeout backstop', () => {
    const timeouts = workflowsWorkflow.match(/^ {4}timeout-minutes: \d+$/gm);

    expect(timeouts).toEqual([
      '    timeout-minutes: 10',
      '    timeout-minutes: 10',
    ]);
  });
});

describe('shell blocks under shellcheck', () => {
  // SC2046: an array passes one argument per digest explicitly, so the list
  // does not depend on word splitting a command substitution.
  it('builds the manifest digest list as an array', () => {
    expect(releaseWorkflow).toContain(
      'mapfile -t digests < <(printf "${IMAGE}@sha256:%s\\n" *)'
    );
    expect(releaseWorkflow).toContain('"${digests[@]}"');
    expect(releaseWorkflow).not.toContain('$(printf "${IMAGE}@sha256:%s " *)');
  });

  // SC2034: the retry counter is used, so the log shows which attempt a flaky
  // packaging step is on.
  it('reports the packaging attempt number', () => {
    expect(exampleAppWorkflow).toContain(
      'echo "Waiting for the packaged app (attempt ${attempt}/30)"'
    );
  });
});

describe('third-party actions', () => {
  // zizmor unpinned-uses: only the publishers listed in .github/zizmor.yml may
  // be referenced by tag; everything else is pinned to a full commit hash.
  it('pins actions outside the trusted publishers to a commit hash', () => {
    const trusted =
      /^(actions|github|docker|astral-sh|lycheeverse|zizmorcore|changesets)\//;
    const unpinned = [];

    for (const [filePath, workflow] of Object.entries({
      '.github/workflows/release.yml': releaseWorkflow,
      '.github/workflows/example-app.yml': exampleAppWorkflow,
      '.github/workflows/workflows.yml': workflowsWorkflow,
      '.github/workflows/links.yml': readWorkflow(
        '.github/workflows/links.yml'
      ),
      '.github/workflows/security.yml': readWorkflow(
        '.github/workflows/security.yml'
      ),
    })) {
      for (const match of workflow.matchAll(/^\s*(?:- )?uses: (\S+)$/gm)) {
        const [, reference] = match;

        // Local composite actions and Docker references carry no upstream ref.
        if (
          reference.startsWith('./') ||
          reference.startsWith('docker://') ||
          trusted.test(reference)
        ) {
          continue;
        }

        if (!/@[0-9a-f]{40}$/.test(reference)) {
          unpinned.push(`${filePath}: ${reference}`);
        }
      }
    }

    expect(unpinned).toEqual([]);
  });
});
