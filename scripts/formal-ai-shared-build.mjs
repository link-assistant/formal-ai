#!/usr/bin/env node
// Issue #1486: one authenticated Formal AI build per effective build identity.
//
//   identity --source DIR --out FILE    write the identity; output key=<sha256>
//   resolve  --identity FILE            elect the producer or find its artifact;
//                                       output mode=reuse|build, artifact_id, producer_run_id,
//                                       waited_seconds
//   verify   --identity FILE --dir DIR --producer-run ID
//                                       reject stale, poisoned or incomplete products
//   manifest --identity FILE --binary FILE --out FILE
//                                       the manifest a successful producer publishes
//
// See scripts/lib/formal-ai-build-identity.mjs for the custody rules.
import { execFileSync } from 'node:child_process';
import { appendFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildIdentity, buildManifest, resolveSharedBuild, verifyBuildProduct } from './lib/formal-ai-build-identity.mjs';

const option = (argv, name) => {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : undefined;
};
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));
const output = (env, text) => env.GITHUB_OUTPUT && appendFileSync(env.GITHUB_OUTPUT, text);

function githubApi(env) {
  const base = env.GITHUB_API_URL ?? 'https://api.github.com';
  const token = env.GH_TOKEN ?? env.GITHUB_TOKEN;
  return async path => {
    const response = await fetch(base + path, {
      headers: { accept: 'application/vnd.github+json', authorization: 'Bearer ' + token, 'x-github-api-version': '2022-11-28' },
      signal: AbortSignal.timeout(30000),
    });
    if (!response.ok) throw Error('GitHub API ' + response.status + ' for ' + path);
    return response.json();
  };
}

export async function main(argv = process.argv.slice(2), env = process.env) {
  const command = argv[0];
  if (command === 'identity') {
    const source = option(argv, '--source') ?? '.formal-ai-source';
    const git = (...args) => execFileSync('git', ['-C', source, ...args], { encoding: 'utf8', timeout: 15000 }).trim();
    const lock = join(source, 'rust/Cargo.lock');
    const identity = buildIdentity({
      sourceCommit: git('rev-parse', 'HEAD'),
      sourceTree: git('rev-parse', 'HEAD^{tree}'),
      rustcVerbose: execFileSync('rustc', ['-vV'], { encoding: 'utf8', timeout: 30000 }),
      cargoLock: existsSync(lock) ? readFileSync(lock) : null,
    });
    writeFileSync(option(argv, '--out'), JSON.stringify(identity, null, 2) + '\n');
    output(env, 'key=' + identity.key + '\n');
    console.log('Formal AI build identity ' + identity.key + ': ' + JSON.stringify(identity.inputs));
    return 0;
  }
  if (command === 'resolve') {
    const identity = readJson(option(argv, '--identity'));
    const decision = await resolveSharedBuild({
      key: identity.key, self: env.GITHUB_RUN_ID, repository: env.GITHUB_REPOSITORY, api: githubApi(env),
      sleep: ms => new Promise(resolve => setTimeout(resolve, ms)), now: () => Date.now(),
      maxWaitSeconds: Number(env.FORMAL_AI_SHARED_BUILD_WAIT_SECONDS ?? 600), log: line => console.log(line),
    });
    output(env, 'mode=' + decision.mode + '\nreason=' + (decision.reason ?? 'reuse') + '\nartifact_id=' + (decision.artifact?.id ?? '')
      + '\nproducer_run_id=' + (decision.producerRunId ?? '') + '\nwaited_seconds=' + decision.waitedSeconds + '\n');
    console.log('Formal AI build ' + identity.key + ': ' + decision.mode + (decision.mode === 'reuse'
      ? ' artifact ' + decision.artifact.id + ' from run ' + decision.producerRunId
      : ' (' + decision.reason + ', leader run ' + decision.leader + ')') + ' after ' + decision.waitedSeconds + 's');
    return 0;
  }
  if (command === 'verify') {
    const identity = readJson(option(argv, '--identity'));
    const dir = option(argv, '--dir');
    const manifestPath = join(dir, 'formal-ai-build.json');
    const binaryPath = join(dir, identity.inputs.bin);
    const verified = verifyBuildProduct({
      identity,
      manifest: existsSync(manifestPath) ? readJson(manifestPath) : null,
      binary: existsSync(binaryPath) ? readFileSync(binaryPath) : null,
      producerRunId: option(argv, '--producer-run'),
      repository: env.GITHUB_REPOSITORY,
    });
    console.log('Reusing the authenticated Formal AI build ' + verified.key + ' from run ' + verified.producerRunId + ': sha256 ' + verified.sha256 + ', ' + verified.bytes + ' bytes.');
    return 0;
  }
  if (command === 'manifest') {
    const identity = readJson(option(argv, '--identity'));
    const binaryPath = option(argv, '--binary');
    const manifest = buildManifest({
      identity, binary: readFileSync(binaryPath),
      version: execFileSync(binaryPath, ['--version'], { encoding: 'utf8', timeout: 60000 }),
      producer: { repository: env.GITHUB_REPOSITORY, runId: env.GITHUB_RUN_ID, runAttempt: env.GITHUB_RUN_ATTEMPT },
    });
    writeFileSync(option(argv, '--out'), JSON.stringify(manifest, null, 2) + '\n');
    console.log('Published Formal AI build ' + manifest.key + ': sha256 ' + manifest.binary.sha256 + '.');
    return 0;
  }
  console.error('usage: formal-ai-shared-build.mjs identity|resolve|verify|manifest ...');
  return 2;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main().then(code => { process.exitCode = code; }, error => {
    console.error('::error title=Shared Formal AI build::' + error.message);
    process.exitCode = 1;
  });
}
