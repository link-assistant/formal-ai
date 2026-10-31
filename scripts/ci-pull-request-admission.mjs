// Fast, conservative PR admission. Production retains its full Rust classifier.
import assert from 'node:assert/strict';
import {appendFileSync, existsSync, readFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
import {isDocumentationPath} from './lib/documentation-paths.mjs';

export const isDocumentation = isDocumentationPath;
// Classify conservatively: extra lightweight checks are cheaper than missed changes.
export function admission(paths) {
  assert(Array.isArray(paths) && paths.every(path => typeof path === 'string'));
  const relevant = paths.filter(path => !['experiments/', 'dev/log/', 'docs/case-studies/'].some(prefix => path.startsWith(prefix)));
  const code = relevant.some(path => !isDocumentation(path));
  return Object.fromEntries(['rs-changed', 'toml-changed', 'docs-changed', 'workflow-changed',
    'pipeline-changed', 'agentic-routing-changed', 'any-code-changed', 'js-changed',
    'ts-changed', 'rust-changed', 'release-pending'].map(key => [key,
      key === 'release-pending' ? false : key === 'docs-changed' ? relevant.some(isDocumentation) : code]));
}
export function versionIn(text) {
  const section = /^\[package\]\s*\n([\s\S]*?)(?=^\[|$(?![\s\S]))/mu.exec(text);
  const version = /^version\s*=\s*"([^"]+)"/mu.exec(section?.[1] ?? '');
  assert(version, 'readable package version required; refusing unknown manifest');
  return version[1];
}
export function checkVersions(head, base) {
  assert.equal(versionIn(head), versionIn(base), 'Use a changelog fragment instead of changing the crate version');
}
function git(args) {
  return execFileSync('git', args, {encoding: 'utf8', timeout: 15000, maxBuffer: 4 * 1024 * 1024});
}
export function changedPaths(event, before, invoke = git) {
  if (event === 'pull_request') {
    const parents = invoke(['rev-list', '--parents', '-n', '1', 'HEAD']).trim().split(/\s+/u);
    assert(parents.length === 3, 'PR admission requires the checked-out synthetic merge commit');
    return invoke(['diff', '--no-renames', '--name-only', '-z', parents[1], parents[2]]).split('\0').filter(Boolean);
  }
  // Bot dispatch checks the whole branch against the latest default branch.
  const base = event === 'workflow_dispatch' ? 'origin/main' : before;
  assert(base && !/^0+$/u.test(base), 'known comparison base required');
  return invoke(['diff', '--no-renames', '--name-only', '-z', base, 'HEAD']).split('\0').filter(Boolean);
}
function main(mode) {
  if (mode === 'detect') {
    const result = admission(changedPaths(process.env.GITHUB_EVENT_NAME, process.env.GITHUB_EVENT_BEFORE));
    for (const [key, value] of Object.entries(result)) {
      const line = `${key}=${value}\n`;
      if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, line);
      process.stdout.write(line);
    }
  } else {
    assert.equal(mode, 'version');
    if (process.env.GITHUB_EVENT_NAME !== 'pull_request') return;
    if (['changelog-manual-release-', 'changeset-release/', 'release/', 'automated-release/']
      .some(prefix => process.env.GITHUB_HEAD_REF?.startsWith(prefix))) return;
    const base = process.env.GITHUB_BASE_REF || 'main';
    assert(/^[a-zA-Z0-9_./-]+$/u.test(base) && !base.startsWith('-'), 'valid base branch required');
    git(['fetch', 'origin', `${base}:refs/remotes/origin/${base}`, '--depth=1']);
    const head = ['Cargo.toml', 'rust/Cargo.toml'].find(existsSync);
    assert(head, 'head manifest required');
    let manifest;
    for (const path of ['Cargo.toml', 'rust/Cargo.toml']) {
      try { manifest = git(['show', `origin/${base}:${path}`]); break; } catch { /* Try the other maintained layout. */ }
    }
    assert(manifest, 'base manifest required');
    checkVersions(readFileSync(head, 'utf8'), manifest);
    console.log('Package version unchanged.');
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main(process.argv[2]);
