import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const registries = ['marketplace', 'open-vsx'];
const statuses = ['skipped-unconfigured', 'publication-command-completed',
  'publication-failed', 'existing-publication-unverified', 'not-attempted'];
const identityFields = ['repository', 'tag', 'packageVersion', 'sourceCommit',
  'sourceTree', 'protocolCommit', 'producerRun', 'producerAttempt',
  'consumerRun', 'consumerAttempt', 'packageSha256'];

export function publicationIdentity(environment, packageBytes, manifest) {
  assert.ok(Buffer.isBuffer(packageBytes) && packageBytes.length > 0);
  assert.ok(packageBytes.length <= 400 * 1024 * 1024);
  const identity = {
    repository: environment.GITHUB_REPOSITORY,
    tag: environment.PACKAGE_RELEASE_TAG,
    packageVersion: manifest.version,
    sourceCommit: environment.EXTENSION_SOURCE_COMMIT,
    sourceTree: environment.EXTENSION_SOURCE_TREE,
    protocolCommit: environment.GITHUB_WORKFLOW_SHA,
    producerRun: environment.EXTENSION_PRODUCER_RUN,
    producerAttempt: environment.EXTENSION_PRODUCER_ATTEMPT,
    consumerRun: environment.GITHUB_RUN_ID,
    consumerAttempt: environment.GITHUB_RUN_ATTEMPT,
    packageSha256: digest(packageBytes)
  };
  assert.match(identity.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(identity.tag, /^v[0-9]+\.[0-9]+\.[0-9]+$/u);
  assert.match(identity.packageVersion, /^[0-9]+\.[0-9]+\.[0-9]+$/u);
  if (environment.EXTENSION_BUILT_VERSION !== undefined) {
    assert.equal(identity.packageVersion, environment.EXTENSION_BUILT_VERSION,
      'packaged metadata differs from independently selected prepared version');
  }
  for (const key of ['sourceCommit', 'sourceTree', 'protocolCommit']) {
    assert.match(identity[key], /^[a-f0-9]{40}$/u);
  }
  for (const key of ['producerRun', 'producerAttempt', 'consumerRun', 'consumerAttempt']) {
    assert.match(String(identity[key]), /^[1-9][0-9]*$/u);
  }
  assert.equal(identity.packageSha256, environment.EXTENSION_PACKAGE_SHA256);
  assert.equal(manifest.publisher, 'link-assistant');
  assert.equal(manifest.name, 'formal-ai-vscode');
  return Object.freeze(identity);
}

export function publicationReceipt(identity, registry, status, processExitCode = null) {
  assert.ok(registries.includes(registry));
  assert.ok(statuses.includes(status));
  assert.ok(processExitCode === null || Number.isInteger(processExitCode));
  if (status === 'publication-command-completed') assert.equal(processExitCode, 0);
  if (status === 'publication-failed') assert.ok(processExitCode === null || processExitCode !== 0);
  const result = {
    schema: 'formal-ai-extension-publication-outcome/v1',
    ...identity, registry, status, processExitCode,
    authenticated: false, registryDigestObserved: false,
    delivery: 'Unknown', publicationComplete: false
  };
  for (const key of identityFields) assert.ok(result[key] !== undefined);
  return result;
}

export function qualifyPublicationOutcomes(expected, receipts) {
  assert.equal(receipts.length, registries.length);
  for (const registry of registries) {
    const matches = receipts.filter(receipt => receipt.registry === registry);
    assert.equal(matches.length, 1);
    const receipt = matches[0];
    assert.equal(receipt.schema, 'formal-ai-extension-publication-outcome/v1');
    for (const key of identityFields) assert.equal(receipt[key], expected[key]);
    const canonical = publicationReceipt(expected, registry, receipt.status, receipt.processExitCode);
    assert.deepEqual(receipt, canonical);
  }
  return {
    sourceBoundData: true, authenticated: false, delivery: 'Unknown',
    publicationComplete: false, registryEvidence: 'Missing',
    runtimeEvidence: 'Missing', outcomes: receipts
  };
}

export function preparedPublicationIdentity(environment, packageBytes,
  sourceManifest, cargoText, packagedManifest) {
  const sections = cargoText.split(/^\[package\]\s*$/mu);
  assert.equal(sections.length, 2, 'one source-owned Cargo package section required');
  const packageSection = sections[1].split(/^\[/mu)[0];
  const versions = [...packageSection.matchAll(/^\s*version\s*=\s*"([^"\r\n]+)"\s*$/gmu)];
  assert.equal(versions.length, 1, 'one literal source-owned Cargo version required');
  const version = versions[0][1];
  assert.match(version, /^[0-9]+\.[0-9]+\.[0-9]+$/u);
  assert.equal(environment.EXTENSION_BUILT_VERSION, version);
  assert.equal(environment.PACKAGE_RELEASE_TAG, 'v' + version);
  assert.equal(packagedManifest.version, version);
  assert.equal(packagedManifest.publisher, sourceManifest.publisher);
  assert.equal(packagedManifest.name, sourceManifest.name);
  return Object.freeze({
    ...publicationIdentity(environment, packageBytes, packagedManifest),
    sourceManifestVersion: sourceManifest.version
  });
}

function identityFromFiles(environment) {
  const file = path.resolve(environment.EXTENSION_PACKAGE_PATH);
  const stat = fs.lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink());
  assert.ok(stat.size > 0 && stat.size <= 400 * 1024 * 1024);
  const manifest = JSON.parse(fs.readFileSync(path.resolve('package.json'), 'utf8'));
  const bytes = fs.readFileSync(file);
  const result = spawnSync('unzip', ['-p', file, 'extension/package.json'], {
    timeout: 10000, maxBuffer: 512 * 1024, encoding: 'utf8'
  });
  assert.equal(result.status, 0, 'exact packaged extension metadata extraction required');
  assert.equal(result.signal, null);
  assert.equal(result.error, undefined);
  const packaged = JSON.parse(result.stdout);
  assert.equal(digest(fs.readFileSync(file)), digest(bytes), 'package changed during metadata read');
  const cargo = fs.readFileSync(path.resolve('../rust/Cargo.toml'), 'utf8');
  return preparedPublicationIdentity(environment, bytes, manifest, cargo, packaged);
}

export function main(arguments_, environment = process.env) {
  const [operation, registry] = arguments_;
  assert.ok(['publish', 'collect'].includes(operation));
  assert.equal(arguments_.length, operation === 'publish' ? 2 : 1);
  const identity = identityFromFiles(environment);
  const directory = path.resolve(environment.EXTENSION_OUTCOME_DIRECTORY);
  fs.mkdirSync(directory, { recursive: true });
  if (operation === 'collect') {
    assert.ok(['true', 'false'].includes(environment.EXTENSION_CHANGED));
    const receipts = registries.map(name => {
      const file = path.join(directory, name + '.json');
      if (fs.existsSync(file)) return JSON.parse(fs.readFileSync(file, 'utf8'));
      return publicationReceipt(identity, name, environment.EXTENSION_CHANGED === 'false'
        ? 'existing-publication-unverified' : 'not-attempted');
    });
    const qualified = qualifyPublicationOutcomes(identity, receipts);
    qualified.marketplaceInstallOutcome = environment.EXTENSION_INSTALL_OUTCOME || 'not-attempted';
    assert.ok(['not-attempted', 'skipped-unconfigured', 'observed-installed', 'failed']
      .includes(qualified.marketplaceInstallOutcome));
    fs.writeFileSync(path.join(directory, 'publication-outcomes.json'),
      JSON.stringify(qualified, null, 2) + '\n', { flag: 'wx' });
    return qualified;
  }
  assert.ok(registries.includes(registry));
  const tokenName = registry === 'marketplace' ? 'VSCE_PAT' : 'OVSX_PAT';
  let status = 'skipped-unconfigured', processExitCode = null;
  if (environment[tokenName]) {
    const command = registry === 'marketplace'
      ? ['--no-install', 'vsce', 'publish', '--packagePath', environment.EXTENSION_PACKAGE_PATH]
      : ['--yes', 'ovsx@1.2.0', 'publish', environment.EXTENSION_PACKAGE_PATH];
    const result = spawnSync('npx', command, {
      env: environment, stdio: 'inherit', timeout: 15 * 60 * 1000
    });
    processExitCode = result.status;
    status = result.status === 0 && !result.error && !result.signal
      ? 'publication-command-completed' : 'publication-failed';
  }
  const receipt = publicationReceipt(identity, registry, status, processExitCode);
  fs.writeFileSync(path.join(directory, registry + '.json'),
    JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
  if (status === 'publication-failed') process.exitCode = processExitCode || 1;
  return receipt;
}

if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  console.log(JSON.stringify(main(process.argv.slice(2))));
}
