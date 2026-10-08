// Publish a built browser-engine tarball once; unknown registry errors fail.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

/** Run npm without a shell, returning JSON for a registry lookup. */
export function runNpm(argumentsList, directory) {
  return execFileSync('npm', argumentsList, {
    cwd: directory, encoding: 'utf8', timeout: 120000, stdio: ['ignore', 'pipe', 'pipe'],
  });
}

/** Determine whether immutable registry bytes match the prepared tarball. */
export function publicationDecision(specification, integrity, lookup) {
  let existing;
  try {
    existing = JSON.parse(lookup(['view', specification, 'dist.integrity', '--json']));
  } catch (error) {
    let registryError;
    try { registryError = JSON.parse(String(error.stdout ?? '')).error; } catch { /* transport failure */ }
    if (registryError?.code === 'E404') return { publish: true };
    throw error;
  }
  if (existing === integrity) return { publish: false };
  throw new Error('Published package integrity differs from this release tarball');
}

/** Upload the packed artifact or verify that a previous attempt uploaded it. */
export function publishBrowserEngine(directory = process.cwd(), run = runNpm) {
  const manifest = JSON.parse(readFileSync(resolve(directory, 'package.json'), 'utf8'));
  const tarballs = readdirSync(directory).filter((file) => file.endsWith('.tgz'));
  if (tarballs.length !== 1) throw new Error('Exactly one packed engine tarball is required');
  const archive = resolve(directory, tarballs[0]);
  const integrity = 'sha512-' + createHash('sha512').update(readFileSync(archive)).digest('base64');
  const specification = manifest.name + '@' + manifest.version;
  const decision = publicationDecision(specification, integrity, (argumentsList) => run(argumentsList, directory));
  if (decision.publish) {
    process.stdout.write(run(['publish', archive, '--provenance', '--access', 'public', '--ignore-scripts'], directory));
  } else {
    process.stdout.write(specification + ' already published with matching integrity\n');
  }
  return decision;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) publishBrowserEngine();
