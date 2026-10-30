// Prepare a complete review without treating optional credentials as publication authority.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {boundedCommandSync} from '../js/server/bounded-command.mjs';

export function dependencyPublication(paths, layer) {
  assert.ok(['app', 'token', 'default'].includes(layer), 'known credential layer required');
  assert.ok(Array.isArray(paths) && paths.length <= 10000);
  for (const file of paths) assert.ok(typeof file === 'string' && file.length > 0 && !file.includes('\0'));
  const workflows = paths.filter(file => file.startsWith('.github/workflows/'));
  const publish = layer !== 'default' || workflows.length === 0;
  return {publish, reason: publish ? 'publication-candidate' : 'default-token-cannot-write-workflows',
    workflowPaths: workflows, publicationApplied: false};
}

export function prepareDependencyReview({cwd, layer, directory}) {
  const git = (args, maxBuffer) => {
    const result = boundedCommandSync('git', args, {cwd, timeout: 30000, maxBuffer});
    assert.ok(!result.error && result.status === 0 && result.cleanup_complete,
      'bounded local dependency diff failed: ' + JSON.stringify({status: result.status,
        signal: result.signal, error: result.error?.code, timedOut: result.timed_out,
        cleanupComplete: result.cleanup_complete}));
    return result.stdout;
  };
  const names = git(['diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--name-only', '-z', 'HEAD', '--'], 1024 * 1024);
  const paths = new TextDecoder('utf-8', {fatal: true}).decode(names).split('\0').filter(Boolean);
  const decision = dependencyPublication(paths, layer);
  const patch = git(['diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--binary', 'HEAD', '--'], 8 * 1024 * 1024);
  const review = {schema: 'dependency-update-review/v1', ...decision, changedPaths: paths,
    patchBytes: patch.length, patchSha256: createHash('sha256').update(patch).digest('hex')};
  fs.mkdirSync(directory, {recursive: true});
  fs.writeFileSync(path.join(directory, 'update.patch'), patch, {flag: 'wx'});
  fs.writeFileSync(path.join(directory, 'review.json'), JSON.stringify(review, null, 2) + '\n', {flag: 'wx'});
  return review;
}

export function main(environment = process.env) {
  assert.ok(environment.DEPENDENCY_REVIEW_DIRECTORY);
  const review = prepareDependencyReview({cwd: process.cwd(), layer: environment.AUTOMATION_LAYER,
    directory: environment.DEPENDENCY_REVIEW_DIRECTORY});
  if (environment.GITHUB_OUTPUT) fs.appendFileSync(environment.GITHUB_OUTPUT, 'publish=' + review.publish + '\n');
  if (environment.GITHUB_STEP_SUMMARY) fs.appendFileSync(environment.GITHUB_STEP_SUMMARY,
    '\n### Dependency update review\n\n' + (review.publish
      ? 'A complete patch is retained; the selected credential may attempt PR publication.\n'
      : 'Publication deferred: the default token cannot write workflow files. The complete patch is retained in the dependency-update-review artifact; no partial update is published.\n'));
  console.log(JSON.stringify(review));
  return review;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
