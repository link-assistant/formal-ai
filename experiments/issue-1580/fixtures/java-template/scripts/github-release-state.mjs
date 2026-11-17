import { spawnSync } from 'node:child_process';
import { printUntrusted } from './github-actions-log.mjs';

/** Return exists, missing, or unknown; only gh's explicit not-found response is missing. */
export function githubReleaseState(tag, repository = process.env.GITHUB_REPOSITORY) {
  const args = ['release', 'view', tag];
  if (repository) args.push('--repo', repository);
  const result = spawnSync('gh', args, { encoding: 'utf8' });
  if (result.status === 0) return 'exists';
  if (result.status === 1 && result.stderr?.trim() === 'release not found') return 'missing';

  console.warn(`::warning::Could not determine whether GitHub release ${tag} exists; skipping recovery.`);
  if (process.env.RELEASE_DEBUG === '1') {
    printUntrusted(result.error?.message || result.stderr || `gh exited with status ${result.status}`);
  }
  return 'unknown';
}

/** Keep versions safe for tag names, command arguments, and GitHub output files. */
export function validateReleaseVersion(version) {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid release version in project configuration`);
  }
  return version;
}
