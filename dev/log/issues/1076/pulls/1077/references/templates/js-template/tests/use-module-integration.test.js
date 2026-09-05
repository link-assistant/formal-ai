/* eslint local/no-changelog-comments: "off" */

/**
 * End-to-end guard for the use-m interop shim.
 *
 * The unit tests in use-module.test.js pin the shapes we normalise; this file
 * loads `command-stream` through the real, unpinned use-m on the same Node
 * version the release jobs run (`node-version: '24.x'`) and asserts `$` is
 * callable. Without it the interop breakage only surfaces on `main`, inside a
 * job that pushes tags and publishes to npm.
 *
 * The test needs network access. When the fetch of use.js or the package
 * install fails, it logs the reason and passes, so offline development and
 * sandboxed runs are not blocked by an unreachable CDN.
 *
 * It also skips on Windows: use-m imports the resolved file by its bare
 * absolute path, which the ESM loader rejects there with
 * ERR_UNSUPPORTED_ESM_URL_SCHEME ("On Windows, absolute paths must be valid
 * file:// URLs"). That is an upstream loader bug in use-m, independent of the
 * namespace shape this shim normalises, and the Linux and macOS runs of this
 * same test still cover the interop.
 */

import { describe, it, expect } from 'test-anywhere';

import { loadCommandStream, USE_M_URL } from '../scripts/use-module.mjs';

async function hasNetwork() {
  try {
    const response = await fetch(USE_M_URL, { method: 'HEAD' });
    return response.ok;
  } catch {
    return false;
  }
}

describe('use-m loads command-stream on this Node version', () => {
  it('exposes a callable $ from command-stream', async () => {
    if (process.platform === 'win32') {
      console.log(
        'Skipping: use-m imports resolved paths without a file:// scheme, ' +
          'which the Windows ESM loader rejects (ERR_UNSUPPORTED_ESM_URL_SCHEME).'
      );
      return;
    }

    if (!(await hasNetwork())) {
      console.log(
        `Skipping: ${USE_M_URL} is unreachable, so use-m cannot be evaluated.`
      );
      return;
    }

    let commandStream;
    try {
      commandStream = await loadCommandStream();
    } catch (error) {
      if (
        /fetch|network|ENOTFOUND|EAI_AGAIN|ETIMEDOUT|registry/i.test(
          error.message
        )
      ) {
        console.log(`Skipping: ${error.message}`);
        return;
      }
      throw error;
    }

    console.log(`Loaded command-stream on ${process.version}`);
    expect(typeof commandStream.$).toBe('function');
  });
});
