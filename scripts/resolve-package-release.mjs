// Resolve a published package release without executing a workflow's arbitrary ref.
// Automated GITHUB_TOKEN releases need the CI completion event, because their
// release event is suppressed. Only a published tag belonging to that run is used.
import { execFileSync } from 'node:child_process';
import { appendFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const RELEASE_TAG = /^v[0-9]+\.[0-9]+\.[0-9]+$/;
const COMMIT_IDENTIFIER = /^[a-f0-9]{40,64}$/;

/** Read JSON from GitHub with arguments kept separate from the command. */
export function githubJson(endpoint) {
  return JSON.parse(execFileSync('gh', ['api', endpoint], {
    encoding: 'utf8', timeout: 30000, stdio: ['ignore', 'pipe', 'pipe'],
  }));
}

/** Resolve checks, an explicit publication, or the release made by a main run. */
export function resolvePackageRelease(environment, readGithub = githubJson) {
  const event = environment.EVENT ?? '';
  const repository = environment.REPOSITORY ?? '';
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
    throw new Error('REPOSITORY must name an owner and repository');
  }
  if (event === 'pull_request' || (event === 'workflow_dispatch' && environment.PUBLISH !== 'true')) {
    return { tag: '', publish: false, build: true };
  }
  if (!['release', 'workflow_run', 'workflow_dispatch'].includes(event)) {
    throw new Error('Unsupported package release event');
  }
  if (event === 'workflow_dispatch' && environment.REFERENCE !== 'refs/heads/main') {
    throw new Error('Package publication dispatch requires main');
  }
  if (event === 'workflow_run' && (environment.RUN_BRANCH !== 'main'
    || environment.RUN_REPOSITORY !== repository
    || ['cancelled', 'skipped'].includes(environment.RUN_CONCLUSION))) {
    return { tag: '', publish: false, build: false };
  }
  const requested = event === 'release' ? environment.RELEASE_TAG : environment.INPUT_TAG;
  let releases;
  if (event !== 'workflow_run' && requested) {
    if (!RELEASE_TAG.test(requested)) throw new Error('Release tag must be v<major>.<minor>.<patch>');
    releases = [readGithub('repos/' + repository + '/releases/tags/' + requested)];
  } else {
    releases = readGithub('repos/' + repository + '/releases?per_page=30');
  }
  const head = environment.RUN_HEAD ?? '';
  if (event === 'workflow_run' && !COMMIT_IDENTIFIER.test(head)) {
    throw new Error('CI completion must identify a full commit');
  }
  for (const release of releases) {
    if (release.draft || release.prerelease || !RELEASE_TAG.test(release.tag_name)) continue;
    if (event === 'workflow_run') {
      const commit = readGithub('repos/' + repository + '/commits/' + release.tag_name);
      if (commit.sha !== head && commit.parents?.[0]?.sha !== head) continue;
    }
    return { tag: release.tag_name, publish: true, build: true };
  }
  if (event === 'workflow_run') return { tag: '', publish: false, build: false };
  throw new Error('No published stable release exists for package publication');
}

/** Emit the resolver's decision as Actions outputs. */
export function main(environment = process.env) {
  const decision = resolvePackageRelease(environment);
  const output = 'tag=' + decision.tag + '\npublish=' + decision.publish + '\nbuild=' + decision.build + '\n';
  process.stdout.write(output);
  if (environment.GITHUB_OUTPUT) appendFileSync(environment.GITHUB_OUTPUT, output);
  return decision;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
