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

export function prepareDependencyReview({cwd, layer, directory, branch}) {
  const gitResult = (args, maxBuffer, statuses = [0]) => {
    const result = boundedCommandSync('git', args, {cwd, timeout: 30000, maxBuffer});
    assert.ok(!result.error && statuses.includes(result.status) && result.cleanup_complete,
      'bounded dependency review command failed: ' + JSON.stringify({operation: args[0], status: result.status,
        signal: result.signal, error: result.error?.code, timedOut: result.timed_out,
        cleanupComplete: result.cleanup_complete}));
    return result;
  };
  const git = (args, maxBuffer) => gitResult(args, maxBuffer).stdout;
  const names = git(['diff', '--no-ext-diff', '--no-textconv', '--no-renames', '--name-only', '-z', 'HEAD', '--'], 1024 * 1024);
  const paths = new TextDecoder('utf-8', {fatal: true}).decode(names).split('\0').filter(Boolean);
  const decision = dependencyPublication(paths, layer);
  if (branch !== undefined) {
    assert.match(branch, /^dependencies\/latest\/[1-9][0-9]{0,19}$/u);
    decision.branch = branch;
    if (decision.publish) {
      const ref = 'refs/heads/' + branch;
      const remote = gitResult(['ls-remote', '--exit-code', '--refs', 'origin', ref], 65536, [0, 2]);
      const observed = remote.stdout.toString('utf8').trim();
      if (remote.status === 2) assert.equal(observed, '', 'missing branch returned unexpected data');
      else {
        const rows = observed.split(/\s+/u);
        assert.equal(rows.length, 2); assert.match(rows[0], /^[a-f0-9]{40}$/u); assert.equal(rows[1], ref);
        Object.assign(decision, {publish: false, reason: 'immutable-update-branch-already-exists', existingCommit: rows[0]});
      }
    }
  }
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
  assert.match(environment.GITHUB_RUN_ID ?? '', /^[1-9][0-9]{0,19}$/u, 'exact workflow run identity required');
  const review = prepareDependencyReview({cwd: process.cwd(), layer: environment.AUTOMATION_LAYER,
    directory: environment.DEPENDENCY_REVIEW_DIRECTORY, branch: 'dependencies/latest/' + environment.GITHUB_RUN_ID});
  if (environment.GITHUB_OUTPUT) fs.appendFileSync(environment.GITHUB_OUTPUT,
    'publish=' + review.publish + '\nbranch=' + review.branch + '\n');
  if (environment.GITHUB_STEP_SUMMARY) fs.appendFileSync(environment.GITHUB_STEP_SUMMARY,
    '\n### Dependency update review\n\n' + (review.publish
      ? 'A complete patch is retained; the selected credential may attempt PR publication.\n'
      : 'Publication deferred (' + review.reason + '). The complete patch is retained in the dependency-update-review artifact; no partial update, branch rewrite or duplicate PR is attempted.\n'));
  console.log(JSON.stringify(review));
  return review;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
