import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { resolvePackageRelease } from './resolve-package-release.mjs';
import { selectCompletedSource } from './release-extension-selection.mjs';
import { createReadonlyGithub,
 transferFromAuthenticatedGithub } from './release-extension-github.mjs';
export function resolveExtensionRelease(environment,
 event,
 github) {
  if (environment.EVENT === 'workflow_run' &&
 event.workflow_run?.name === 'Desktop Release') {
    assert.equal(event.workflow_run.head_branch,
 'main');
    assert.equal(event.workflow_run.head_repository.full_name,
 environment.REPOSITORY);
    const selected = selectCompletedSource({
      repository: environment.REPOSITORY,

      runId: event.workflow_run.id,

      attempt: event.workflow_run.run_attempt
    },
 github);
    return {
      tag: selected.tag,

      publish: true,

      build: true
    };
  }
  const decision = resolvePackageRelease(environment,
 github.json);
  return {
    ...decision,

    publish: environment.EVENT === 'workflow_run' ? false : decision.publish
  };
}
export function receiveExtension(environment,
 event,
 github,
 directory) {
  const repository = environment.GITHUB_REPOSITORY,

    tag = environment.PACKAGE_RELEASE_TAG;
  assert.match(repository,
 /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(tag,
 /^v[0-9]+\.[0-9]+\.[0-9]+$/u);
  for (const key of ['GITHUB_RUN_ID',
 'GITHUB_RUN_ATTEMPT']) assert.match(environment[key],
 /^[1-9][0-9]*$/u);
  assert.ok(['workflow_run',
 'release',
 'workflow_dispatch'].includes(environment.GITHUB_EVENT_NAME));
  if (environment.GITHUB_EVENT_NAME === 'workflow_dispatch') assert.equal(environment.GITHUB_REF,
 'refs/heads/main');
  if (environment.GITHUB_EVENT_NAME === 'release') assert.equal(event.release.tag_name,
 tag);
  const base = 'repos/' + repository;
  const consumer = github.json(base + '/actions/runs/' + environment.GITHUB_RUN_ID + '/attempts/' + environment.GITHUB_RUN_ATTEMPT);
  assert.equal(consumer.id,
 Number(environment.GITHUB_RUN_ID));
  assert.equal(consumer.run_attempt,
 Number(environment.GITHUB_RUN_ATTEMPT));
  assert.equal(consumer.repository.full_name,
 repository);
  assert.equal(consumer.head_repository.full_name,
 repository);
  assert.equal(consumer.path,
 '.github/workflows/publish-vscode.yml');
  assert.equal(consumer.event,
 environment.GITHUB_EVENT_NAME);
  if (environment.GITHUB_EVENT_NAME !== 'release') assert.equal(consumer.head_branch,
 'main');
  const release = github.json(base + '/releases/tags/' + tag);
  assert.equal(release.tag_name,
 tag);
  assert.equal(release.draft,
 false);
  assert.equal(release.prerelease,
 false);
  const asset = release.assets.filter(item => item.name === 'formal-ai-native-source-' + tag.slice(1) + '.json');
  assert.equal(asset.length,
 1,
 'missing or duplicate producer source receipt');
  const receipt = JSON.parse(new TextDecoder('utf-8',
 {
    fatal: true
  }).decode(github.bytes(base + '/releases/assets/' + asset[0].id)));
  assert.equal(receipt.release_tag,
 tag);
  assert.match(receipt.producer_run,
 /^[1-9][0-9]*$/u);
  const run = github.json(base + '/actions/runs/' + receipt.producer_run);
  if (environment.GITHUB_EVENT_NAME === 'workflow_run') {
    assert.equal(event.workflow_run.name,
 'Desktop Release',
 'CI completion is checks only');
    assert.equal(String(event.workflow_run.id),
 receipt.producer_run);
    assert.equal(event.workflow_run.run_attempt,
 run.run_attempt);
  }
  const selected = selectCompletedSource({
    repository,

    runId: receipt.producer_run,

    attempt: run.run_attempt
  },
 github);
  assert.equal(selected.tag,
 tag);
  assert.equal(selected.sourceCommit,
 receipt.source_commit);
  assert.equal(selected.sourceTree,
 receipt.source_tree);
  const checks = github.json(base + '/actions/runs/' + environment.GITHUB_RUN_ID + '/attempts/' + environment.GITHUB_RUN_ATTEMPT + '/jobs?per_page=100');
  assert.equal(checks.total_count,
 checks.jobs.length);
  const matches = checks.jobs.filter(job => job.name === 'publish');
  assert.equal(matches.length,
 1,
 'exact completed consumer checks job required');
  const expected = {
    ...selected,

    consumerCheckJobId: matches[0].id,

    consumerRunId: Number(environment.GITHUB_RUN_ID),

    consumerAttempt: Number(environment.GITHUB_RUN_ATTEMPT)
  };
  const verified = transferFromAuthenticatedGithub(expected,
 matches[0].id,
 directory,
 github);
  const destination = path.resolve(environment.EXTENSION_DESTINATION);
  assert.ok(fs.lstatSync(destination).isDirectory() &&
 !fs.lstatSync(destination).isSymbolicLink());
  const file = path.join(destination,
 verified.name);
  assert.equal(fs.existsSync(file),
 false,
 'refuse existing package overwrite');
  fs.writeFileSync(file,
 verified.bytes,
 {
    flag: 'wx',

    mode: 0o600
  });
  return {
    ...verified,

    bytes: undefined,

    path: file
  };
}
export function main(arguments_,
 environment = process.env) {
  assert.equal(arguments_.length,
 1);
  assert.ok(['resolve',
 'receive'].includes(arguments_[0]));
  const event = JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH,
 'utf8'));
  const directory = path.join(environment.RUNNER_TEMP,
 'extension-transfer-verification');
  const github = createReadonlyGithub(directory,
 {
    deadline: Date.now() + (arguments_[0] === 'resolve' ? 170000 : 15 * 60 * 1000)
  });
  if (arguments_[0] === 'resolve') {
    const result = resolveExtensionRelease(environment,
 event,
 github);
    const outputs = 'tag=' + result.tag + '\npublish=' + result.publish + '\nbuild=' + result.build + '\n';
    fs.appendFileSync(environment.GITHUB_OUTPUT,
 outputs);
    return result;
  }
  const result = receiveExtension(environment,
 event,
 github,
 directory);
  fs.appendFileSync(environment.GITHUB_OUTPUT,
 'source-commit=' + result.sourceCommit + '\npackage-path=' + result.path + '\nsha256=' + result.sha256 + '\n');
  return result;
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) console.log(JSON.stringify(main(process.argv.slice(2))));
