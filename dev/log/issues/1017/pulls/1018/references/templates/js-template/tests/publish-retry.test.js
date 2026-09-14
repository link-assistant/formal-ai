import { describe, it, expect } from 'test-anywhere';

import {
  isAlreadyPublishedError,
  publishWithRetry,
  waitForVersionOnRegistry,
} from '../scripts/publish-retry.mjs';

const noSleep = async () => {};

describe('waitForVersionOnRegistry', () => {
  it('returns true as soon as the version becomes visible', async () => {
    let checks = 0;
    const found = await waitForVersionOnRegistry({
      verify: async () => ++checks >= 3,
      sleepFn: noSleep,
    });
    expect(found).toBe(true);
    expect(checks).toBe(3);
  });

  it('keeps polling when verification throws (E404 propagation lag)', async () => {
    let checks = 0;
    const found = await waitForVersionOnRegistry({
      verify: async () => {
        if (++checks < 3) {
          throw new Error('npm error code E404');
        }
        return true;
      },
      sleepFn: noSleep,
    });
    expect(found).toBe(true);
  });

  it('returns false after exhausting the bounded attempts', async () => {
    let checks = 0;
    const found = await waitForVersionOnRegistry({
      verify: async () => {
        checks++;
        return false;
      },
      attempts: 4,
      sleepFn: noSleep,
    });
    expect(found).toBe(false);
    expect(checks).toBe(4);
  });

  it('uses exponential backoff capped at maxDelay', async () => {
    const delays = [];
    await waitForVersionOnRegistry({
      verify: async () => false,
      attempts: 5,
      initialDelay: 2000,
      maxDelay: 8000,
      sleepFn: async (ms) => {
        delays.push(ms);
      },
    });
    expect(delays).toEqual([2000, 4000, 8000, 8000, 8000]);
  });
});

describe('isAlreadyPublishedError', () => {
  it('detects publish conflicts', () => {
    expect(
      isAlreadyPublishedError(
        'You cannot publish over the previously published versions: 1.0.0.'
      )
    ).toBe(true);
    expect(isAlreadyPublishedError('npm error code EPUBLISHCONFLICT')).toBe(
      true
    );
    expect(isAlreadyPublishedError('npm error code E401')).toBe(false);
    expect(isAlreadyPublishedError(undefined)).toBe(false);
  });
});

describe('publishWithRetry', () => {
  it('never republishes when only verification lags', async () => {
    let publishes = 0;
    let checks = 0;
    const result = await publishWithRetry({
      publish: async () => {
        publishes++;
        if (publishes > 1) {
          return {
            success: false,
            error: new Error(
              'cannot publish over the previously published versions'
            ),
            output: 'cannot publish over the previously published versions',
          };
        }
        return { success: true, error: null, output: 'success' };
      },
      verify: async () => {
        if (++checks < 3) {
          throw new Error('npm error code E404');
        }
        return true;
      },
      sleepFn: noSleep,
    });

    expect(result.success).toBe(true);
    expect(result.publishAttempts).toBe(1);
    expect(publishes).toBe(1);
  });

  it('fails without republishing when verification polling is exhausted', async () => {
    let publishes = 0;
    const result = await publishWithRetry({
      publish: async () => {
        publishes++;
        return { success: true, error: null, output: 'success' };
      },
      verify: async () => false,
      verifyOptions: { attempts: 3 },
      sleepFn: noSleep,
    });

    expect(result.success).toBe(false);
    expect(publishes).toBe(1);
    expect(result.error.verificationFailed).toBe(true);
  });

  it('retries the publish command when the publish itself failed', async () => {
    let publishes = 0;
    const result = await publishWithRetry({
      publish: async () => {
        publishes++;
        if (publishes < 3) {
          return {
            success: false,
            error: new Error('network error'),
            output: 'network error',
          };
        }
        return { success: true, error: null, output: 'success' };
      },
      verify: async () => true,
      sleepFn: noSleep,
    });

    expect(result.success).toBe(true);
    expect(publishes).toBe(3);
  });

  it('fails fast on non-retryable publish errors', async () => {
    let publishes = 0;
    const authError = new Error('ENEEDAUTH');
    authError.nonRetryable = true;
    const result = await publishWithRetry({
      publish: async () => {
        publishes++;
        return { success: false, error: authError, output: 'ENEEDAUTH' };
      },
      verify: async () => true,
      sleepFn: noSleep,
    });

    expect(result.success).toBe(false);
    expect(publishes).toBe(1);
    expect(result.error.message).toBe('ENEEDAUTH');
  });

  it('treats an already-published conflict as a cue to verify', async () => {
    let publishes = 0;
    const result = await publishWithRetry({
      publish: async () => {
        publishes++;
        return {
          success: false,
          error: new Error('npm error code EPUBLISHCONFLICT'),
          output: 'npm error code EPUBLISHCONFLICT',
        };
      },
      verify: async () => true,
      sleepFn: noSleep,
    });

    expect(result.success).toBe(true);
    expect(publishes).toBe(1);
  });
});
