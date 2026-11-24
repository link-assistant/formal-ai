// Publish a built browser-engine tarball once; unknown registry errors fail.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFileSync, readdirSync, appendFileSync, lstatSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { BROWSER_ENGINE_ARCHIVE_MAX_BYTES } from './release-publication-observation.mjs';

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

function checkPackedArchive(directory) {
  const tarballs = readdirSync(directory).filter((file) => file.endsWith('.tgz'));
  if (tarballs.length !== 1) throw new Error('Exactly one packed engine tarball is required');
  const [filename] = tarballs;
  const path = resolve(directory, filename);
  const stat = lstatSync(path);
  if (!stat.isFile()) throw new Error(`Packed engine archive ${JSON.stringify(filename)} must be a regular file`);
  if (stat.size === 0 || stat.size > BROWSER_ENGINE_ARCHIVE_MAX_BYTES) {
    throw new Error(`Packed engine archive ${JSON.stringify(filename)} has ${stat.size} bytes (allowed 1 to ${BROWSER_ENGINE_ARCHIVE_MAX_BYTES} bytes)`);
  }
  return { path, filename, bytes: stat.size };
}

/** Upload the packed artifact or verify that a previous attempt uploaded it. */
export function publishBrowserEngine(directory = process.cwd(), run = runNpm) {
  const { path: archive } = checkPackedArchive(directory);
  const manifest = JSON.parse(readFileSync(resolve(directory, 'package.json'), 'utf8'));
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

export function publicationStatus(decision) {
  if (decision?.publish === true) return "published";
  if (decision?.publish === false) return "already-matching";
  throw new Error("Unknown source-owned publication result");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  if (process.argv[2] === '--check-archive') {
    const { filename, bytes } = checkPackedArchive(process.cwd());
    console.log(`Packed engine archive ${JSON.stringify(filename)} has ${bytes} bytes (limit ${BROWSER_ENGINE_ARCHIVE_MAX_BYTES} bytes)`);
  } else {
    const decision = publishBrowserEngine();
    if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, 'status=' + publicationStatus(decision) + '\n');
  }
}

// A static source edge keeps the publication data observer in the declared protocol closure.
export {captureBrowserPublication} from './release-publication-observation.mjs';
