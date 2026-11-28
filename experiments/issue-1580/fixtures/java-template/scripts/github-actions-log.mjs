import { randomBytes } from 'node:crypto';

/**
 * Print contributor-controlled text without letting the GitHub Actions runner
 * interpret it as a workflow command.
 *
 * @param {unknown} text - Text that must be treated as data.
 */
export function printUntrusted(text) {
  if (!process.env.GITHUB_ACTIONS) {
    console.log(text);
    return;
  }

  // The resume token becomes a runner command while processing is stopped.
  // It must therefore be unpredictable. Hex is also always a legal token.
  const token = randomBytes(16).toString('hex');
  console.log(`::stop-commands::${token}`);
  console.log(text);
  console.log(`::${token}::`);
}
