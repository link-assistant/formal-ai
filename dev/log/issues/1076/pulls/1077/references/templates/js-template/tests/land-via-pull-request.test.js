import { describe, it, expect } from 'test-anywhere';

import {
  findOpenPullRequest,
  landViaPullRequest,
  mergePullRequestWithRetry,
  pullRequestBranchName,
} from '../scripts/land-via-pull-request.mjs';

const silentLogger = { log() {}, error() {} };

/**
 * Record every invocation and answer from a scripted queue of results.
 * @param {Array<{match: RegExp, result: object}>} script
 */
function makeRunner(script) {
  const calls = [];
  const runner = async (command, args) => {
    const line = `${command} ${args.join(' ')}`;
    calls.push(line);
    const entry = script.find((candidate) => candidate.match.test(line));
    return entry ? entry.result : { code: 0, stdout: '', stderr: '' };
  };
  runner.calls = calls;
  return runner;
}

describe('land-via-pull-request', () => {
  it('names the branch per run so no ref is ever force-pushed or deleted', () => {
    // A `no destruction` rule targeting ~ALL blocks force pushes and deletions,
    // so a retried run must never reuse the previous run's branch (link-foundation/js-ai-driven-development-pipeline-template#143).
    expect(
      pullRequestBranchName({ label: '2.13.5', runId: '32589574378' })
    ).toBe('release/2.13.5-32589574378');
    expect(pullRequestBranchName({ label: '2.13.5', runId: '1' })).not.toBe(
      pullRequestBranchName({ label: '2.13.5', runId: '2' })
    );
  });

  it('slugifies labels that are not valid ref components', () => {
    expect(
      pullRequestBranchName({ label: 'preview images!', runId: '7' })
    ).toBe('release/preview-images-7');
  });

  it('retries the merge while GitHub is still computing mergeability', async () => {
    let attempts = 0;
    const runner = async () => {
      attempts += 1;
      return attempts < 3
        ? { code: 1, stdout: '', stderr: 'Pull request is not mergeable' }
        : { code: 0, stdout: '', stderr: '' };
    };

    const result = await mergePullRequestWithRetry({
      runner,
      url: 'https://example.invalid/pr/1',
      delayMs: 0,
      sleepFn: async () => {},
      logger: silentLogger,
    });

    expect(result).toEqual({ merged: true, attempt: 3 });
  });

  it('gives up with the real command error when the merge never succeeds', async () => {
    const runner = async () => ({
      code: 1,
      stdout: '',
      stderr: 'Pull request is not mergeable',
    });

    let thrown;
    try {
      await mergePullRequestWithRetry({
        runner,
        url: 'https://example.invalid/pr/1',
        maxAttempts: 2,
        delayMs: 0,
        sleepFn: async () => {},
        logger: silentLogger,
      });
    } catch (error) {
      thrown = error;
    }

    expect(thrown?.name).toBe('CommandFailedError');
    expect(thrown?.message).toContain('not mergeable');
  });

  it('merges with --merge only, since allowed_merge_methods may exclude squash', async () => {
    const runner = makeRunner([]);
    await mergePullRequestWithRetry({
      runner,
      url: 'https://example.invalid/pr/1',
      logger: silentLogger,
    });
    expect(runner.calls[0]).toBe(
      'gh pr merge https://example.invalid/pr/1 --merge'
    );
  });

  it('pushes, opens, merges, and fast-forwards the checkout', async () => {
    const runner = makeRunner([
      { match: /gh pr list/, result: { code: 0, stdout: '\n' } },
      {
        match: /gh pr create/,
        result: { code: 0, stdout: 'https://example.invalid/pr/9\n' },
      },
    ]);

    const result = await landViaPullRequest({
      runner,
      label: '1.2.3',
      runId: '77',
      mergeDelayMs: 0,
      sleepFn: async () => {},
      logger: silentLogger,
    });

    expect(result.head).toBe('release/1.2.3-77');
    expect(result.url).toBe('https://example.invalid/pr/9');
    expect(runner.calls[0]).toBe(
      'git push origin HEAD:refs/heads/release/1.2.3-77'
    );
    expect(runner.calls[1]).toContain(
      'gh pr list --head release/1.2.3-77 --base main --state open'
    );
    expect(runner.calls[2]).toContain(
      'gh pr create --base main --head release/1.2.3-77 --title 1.2.3 --body'
    );
    expect(runner.calls[3]).toBe(
      'gh pr merge https://example.invalid/pr/9 --merge'
    );
    // The checkout must end on the merged base branch so the publish steps in
    // the same job see the merged tree.
    expect(runner.calls.slice(4)).toEqual([
      'git fetch origin main',
      'git reset --hard origin/main',
    ]);
  });

  it('reuses an open pull request so a re-run creates no duplicate', async () => {
    const runner = makeRunner([
      {
        match: /gh pr list/,
        result: { code: 0, stdout: 'https://example.invalid/pr/9\n' },
      },
    ]);

    await landViaPullRequest({
      runner,
      label: '1.2.3',
      runId: '77',
      mergeDelayMs: 0,
      sleepFn: async () => {},
      logger: silentLogger,
    });

    expect(runner.calls.some((call) => call.startsWith('gh pr create'))).toBe(
      false
    );
  });

  it('reports no open pull request when the lookup itself fails', async () => {
    const runner = async () => ({ code: 1, stdout: '', stderr: 'boom' });
    expect(
      await findOpenPullRequest({
        runner,
        head: 'release/1.2.3-77',
        base: 'main',
        logger: silentLogger,
      })
    ).toBe('');
  });
});
