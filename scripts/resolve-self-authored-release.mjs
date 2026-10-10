// Release identity is a prerequisite, never proof of authored or reviewed work.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {appendFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {githubJson, resolvePackageRelease} from './resolve-package-release.mjs';
import {validatePublishedStableSource} from './native-release-trust.mjs';


/** Completed PR/check events cannot authorize the privileged authoring route. */
function productionReleaseObserver(environment, readGithub) {
  const endpoint = 'repos/' + environment.REPOSITORY + '/actions/runs/' + environment.RUN_ID;
  return path => {
    const result = readGithub(path);
    if (environment.EVENT === 'workflow_run' && path === endpoint) {
      assert.ok(['push', 'workflow_dispatch'].includes(result?.event),
        'completed producer must be a supported production release event');
    }
    return result;
  };
}

/** Resolve publication observations; injected observers grant no effect authority. */
export function resolveSelfAuthoredRelease(cwd, environment, readGithub = githubJson) {
  assert.ok(['release', 'workflow_run'].includes(environment.EVENT));
  if (environment.EVENT === 'release') assert.equal(environment.RELEASE_ACTION, 'published');
  const decision = resolvePackageRelease(environment, productionReleaseObserver(environment, readGithub));
  if (!decision.publish) return {active: false};
  const prefix = 'repos/' + environment.REPOSITORY + '/';
  const release = readGithub(prefix + 'releases/tags/' + encodeURIComponent(decision.tag));
  assert.ok(Number.isSafeInteger(release.id) && release.id > 0, 'exact release identifier required');
  const repository = readGithub(prefix);
  assert.equal(repository.full_name, environment.REPOSITORY);
  assert.equal(repository.default_branch, 'main', 'only the existing default main writer is supported');
  const defaultCommit = readGithub(prefix + 'commits/main').sha;
  const releaseCommit = readGithub(prefix + 'commits/' + encodeURIComponent(decision.tag)).sha;
  const observation = validatePublishedStableSource(cwd, {
    tag: decision.tag, release, defaultCommit, releaseCommit,
  });
  const git = arguments_ => execFileSync('git', arguments_, {cwd, encoding: 'utf8', timeout: 30000}).trim();
  assert.equal(git(['rev-parse', 'HEAD']), defaultCommit, 'checked-out author source must equal captured default source');
  assert.equal(git(['symbolic-ref', '--short', 'HEAD']), repository.default_branch);
  return {
    active: true, branch: repository.default_branch, releaseId: release.id,
    ...observation, effectAuthority: 'Unknown', reviewedContribution: 'Pending',
  };
}

export function main(environment = process.env) {
  // The CLI fixes the observer; callers cannot provide a callback through event data.
  const repository = environment.REPOSITORY;
  const decision = resolvePackageRelease(environment, productionReleaseObserver(environment, githubJson));
  if (decision.publish) {
    execFileSync('git', ['fetch', '--no-tags', 'origin',
      'refs/tags/' + decision.tag + ':refs/tags/' + decision.tag], {stdio: 'pipe', timeout: 30000});
  }
  const result = resolveSelfAuthoredRelease(process.cwd(), environment);
  const output = 'active=' + result.active + '\nbranch=' + (result.branch ?? '') + '\n'
    + 'release_id=' + (result.releaseId ?? '') + '\n';
  assert.ok(repository && environment.GITHUB_OUTPUT);
  appendFileSync(environment.GITHUB_OUTPUT, output);
  console.log(JSON.stringify(result));
  return result;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
