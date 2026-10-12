import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { compileContract, observeCommandStagedContext, observeCommandCurrentGroupStagedContext } from './staged-caller-authority.mjs';
import { runStagedReleaseGenerator, buildStagedReleaseProjection } from './generate-staged-release.mjs';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const YAML = createRequire(path.join(root, 'package.json'))('yaml');
const issued = new WeakMap();
export function compileMaintainedStagedAuthority() {
  const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  const paths = {
    canonical: '.github/workflows/release.yml',
    callerDraft: evidence + 'candidate-release-caller.yml',
    staged: '.github/workflows/release-staged.yml',
    packet: evidence + 'stage-source-coverage.json',
    bindings: evidence + 'candidate-output-bindings.json',
    generator: 'scripts/generate-staged-release.mjs',
    generatorReader: 'scripts/lib/ci-speed-workflows.mjs',
    freshness: 'scripts/release-source-freshness.mjs',
    validator: 'scripts/staged-caller-authority.mjs',
    provider: 'scripts/governed-github-command-provider.mjs',
    maintainedValidator: 'scripts/maintained-staged-authority.mjs'
  };
  const bytes = Object.fromEntries(Object.entries(paths).map(([key, file]) => [key, fs.readFileSync(path.join(root, file), 'utf8')]));
  runStagedReleaseGenerator(['--check']);
  for (const [key, file] of Object.entries(paths)) assert.equal(fs.readFileSync(path.join(root, file), 'utf8'), bytes[key], 'source changed during checked generation');
  const packet = JSON.parse(bytes.packet),
    bindings = JSON.parse(bytes.bindings);
  assert.equal(bindings.originalOperations, 52);
  assert.equal(bindings.bindings.length, 104);
  const contract = compileContract({
    ...bytes,
    packet,
    YAML
  });
  const source = Object.freeze(Object.fromEntries(Object.entries(paths).map(([key, file]) => [file, createHash('sha256').update(bytes[key]).digest('hex')])));
  const receipt = Object.freeze({});
  issued.set(receipt, {
    contract,
    source
  });
  return receipt;
}
export function maintainedStagedInventory(receipt) {
  const authority = issued.get(receipt);
  assert.ok(authority, 'maintained checked compiler receipt required');
  return Object.freeze({
    operations: authority.contract.routes.length,
    bindings: authority.contract.bindings,
    sources: authority.source,
    writerGroup: authority.contract.writerGroup,
    productionAuthority: false
  });
}
export async function observeMaintainedStagedContext(receipt, input) {
  const authority = issued.get(receipt);
  assert.ok(authority, 'maintained checked compiler receipt required');
  for (const [file, sha] of Object.entries(authority.source)) assert.equal(createHash('sha256').update(fs.readFileSync(path.join(root, file))).digest('hex'), sha, 'maintained source changed since compilation');
  return observeCommandStagedContext({
    contract: authority.contract,
    expected: input.expected,
    environment: input.environment,
    event: input.event
  });
}

// Current-operation entry derives host fields; observation inputs cannot override them.
export async function observeCurrentMaintainedStagedContext(receipt, selection) {
  const authority = issued.get(receipt);
  assert.ok(authority, 'maintained checked compiler receipt required');
  assert.deepEqual(Object.keys(selection).sort(), ['caller', 'ordinal']);
  const route = authority.contract.routes.find(route => route.caller === selection.caller && route.ordinal === selection.ordinal);
  assert.ok(route, 'unregistered canonical operation');
  const environment = Object.freeze({
    ...process.env
  });
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  assert.equal(environment.GITHUB_JOB, route.stageJob);
  assert.equal(environment.GITHUB_REF, 'refs/heads/main');
  assert.equal(environment.GITHUB_EVENT_NAME, route.event);
  assert.equal(environment.GITHUB_WORKFLOW_REF, environment.GITHUB_REPOSITORY + '/.github/workflows/release.yml@refs/heads/main');
  assert.ok(environment.GITHUB_EVENT_PATH, 'actual event file required');
  const event = JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH, 'utf8'));
  const expected = Object.freeze({
    caller: route.caller,
    ordinal: route.ordinal,
    stageJob: route.stageJob,
    operationSha256: route.operationSha256,
    repository: environment.GITHUB_REPOSITORY,
    head: environment.GITHUB_SHA,
    run: environment.GITHUB_RUN_ID,
    attempt: environment.GITHUB_RUN_ATTEMPT,
    runnerName: environment.RUNNER_NAME
  });
  return observeMaintainedStagedContext(receipt, {
    expected,
    environment,
    event
  });
}

// Deployment is checked against retained source, separately from ordinary inline checks.
export function compileDeployedStagedAuthority() {
  const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
  const paths={canonical:evidence+'original-release-workflow.yml',deployedCaller:'.github/workflows/release.yml',
    callerDraft:evidence+'candidate-release-caller.yml',staged:'.github/workflows/release-staged.yml',
    packet:evidence+'stage-source-coverage.json',bindings:evidence+'candidate-output-bindings.json',
    generator:'scripts/generate-staged-release.mjs',generatorReader:'scripts/lib/ci-speed-workflows.mjs',
    freshness:'scripts/release-source-freshness.mjs',validator:'scripts/staged-caller-authority.mjs',
    provider:'scripts/governed-github-command-provider.mjs',maintainedValidator:'scripts/maintained-staged-authority.mjs'};
  const bytes=Object.fromEntries(Object.entries(paths).map(([key,file])=>[key,fs.readFileSync(path.join(root,file),'utf8')]));
  const packet=JSON.parse(bytes.packet);
  const projection=buildStagedReleaseProjection(bytes.canonical,packet);
  assert.equal(projection.operations,52);assert.equal(projection.binding.length,104);
  for(const [file,expected] of projection.outputs)assert.equal(fs.readFileSync(path.join(root,file),'utf8'),expected,'retained-source projection differs');
  assert.equal(bytes.deployedCaller,bytes.callerDraft,'deployed caller differs from checked original projection');
  for(const [key,file] of Object.entries(paths))assert.equal(fs.readFileSync(path.join(root,file),'utf8'),bytes[key],'source changed during deployed compilation');
  const contract=compileContract({canonical:bytes.canonical,callerDraft:bytes.callerDraft,staged:bytes.staged,packet,YAML,retainedSourceProof:{originalSource:bytes.canonical,packetSource:bytes.packet}});
  const source=Object.freeze(Object.fromEntries(Object.entries(paths).map(([key,file])=>[file,createHash('sha256').update(bytes[key]).digest('hex')])));
  const receipt=Object.freeze({});issued.set(receipt,{contract,source});return receipt;
}

export async function observeCurrentStepContext(receipt,stepIdentifier) {
  const authority=issued.get(receipt);assert.ok(authority,'maintained checked compiler receipt required');
  const routes=authority.contract.routes.filter(route=>route.stageJob===process.env.GITHUB_JOB && route.stepIdentifier===stepIdentifier);
  assert.equal(routes.length,1,'unique source-declared operation in current stage required');
  return observeCurrentGroupMaintainedStagedContext(receipt,{caller:routes[0].caller,ordinal:routes[0].ordinal});
}

export async function observeCurrentGroupMaintainedStagedContext(receipt, selection) {
  const authority = issued.get(receipt);
  assert.ok(authority, 'maintained checked compiler receipt required');
  assert.deepEqual(Object.keys(selection).sort(), ['caller', 'ordinal']);
  const route = authority.contract.routes.find(route => route.caller === selection.caller && route.ordinal === selection.ordinal);
  assert.ok(route, 'unregistered canonical operation');
  const environment = Object.freeze({
    ...process.env
  });
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  assert.equal(environment.GITHUB_JOB, route.stageJob);
  assert.equal(environment.GITHUB_REF, 'refs/heads/main');
  assert.equal(environment.GITHUB_EVENT_NAME, route.event);
  assert.equal(environment.GITHUB_WORKFLOW_REF, environment.GITHUB_REPOSITORY + '/.github/workflows/release.yml@refs/heads/main');
  assert.ok(environment.GITHUB_EVENT_PATH, 'actual event file required');
  const event = JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH, 'utf8'));
  const expected = Object.freeze({
    caller: route.caller,
    ordinal: route.ordinal,
    stageJob: route.stageJob,
    operationSha256: route.operationSha256,
    repository: environment.GITHUB_REPOSITORY,
    head: environment.GITHUB_SHA,
    run: environment.GITHUB_RUN_ID,
    attempt: environment.GITHUB_RUN_ATTEMPT,
    runnerName: environment.RUNNER_NAME
  });
  for(const [file,sha] of Object.entries(authority.source))assert.equal(createHash('sha256').update(fs.readFileSync(path.join(root,file))).digest('hex'),sha,'maintained source changed since compilation');
  return observeCommandCurrentGroupStagedContext({contract:authority.contract,
    expected,
    environment,
    event
  });
}
