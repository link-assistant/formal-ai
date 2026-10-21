#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { sourceBinaryIdentity, requireImageBundle, fileDigest } from './lib/hive-delivery-bundle.mjs';
export function taskRuntimeJavaHomeCommand() {
  return 'runtime_java="$(command -v java)" && runtime_home="$(dirname "$(dirname "$(readlink -f "$runtime_java")")")" && test -x "$runtime_home/bin/java" && ln -s "$runtime_home" "$JAVA_HOME" && "$JAVA_HOME/bin/java" -version';
}
export function taskRuntimeDockerfile(runtimeImage, {
  published = false
} = {}) {
  if (published) assert.match(runtimeImage, /^ghcr\.io\/link-assistant\/formal-ai@sha256:[a-f0-9]{64}$/u);
  return ['FROM ' + runtimeImage,
 'USER root',
 'COPY --chmod=0755 formal-ai-build/ /usr/local/lib/formal-ai-build/',
 'RUN if ! command -v java >/dev/null 2>&1; then FORMAL_AI_APT_PRIVILEGE= bash /usr/local/lib/formal-ai-build/apt-install-with-retry.sh --no-install-recommends default-jre-headless; fi && java -version && rm -rf /var/lib/apt/lists/* /usr/local/lib/formal-ai-build',
 'ENV JAVA_HOME=/usr/local/lib/formal-ai-jvm',
 'RUN ' + taskRuntimeJavaHomeCommand(),
 ...(published ? [] : ['COPY formal-ai-pr-binary /usr/local/bin/formal-ai']),
 'RUN chmod 0755 /usr/local/bin/formal-ai && formal-ai --version',
 'ENV FORMAL_AI_INSTALL_GRANT=workspace',
 'ENV FORMAL_AI_DIALOG_LOG_DIR=/home/box/logs/formal-ai-dialog',
 'USER box',
 'ENV HOME=/home/box FORMAL_AI_MEMORY_PATH=/home/box/.formal-ai/memory.lino',
 'RUN mkdir -p /home/box/.formal-ai',
 'CMD ["formal-ai","serve","--agent-mode","--host","0.0.0.0","--port","8080"]'].join('\n') + '\n';
}

/** Hosted image observations must agree with the exact prepared adapter before a live solve. */
export function verifyTaskImageOperationalEvidence({
  hive,
  sourceIdentity,
  hiveCommit,
  read,
  run
}) {
  const adapter = JSON.parse(fs.readFileSync(path.join(hive, '.formal-ai-operational-adapter.json'), 'utf8'));
  assert.equal(adapter.upstreamCommit, hiveCommit, 'operational adapter upstream source');
  assert.deepEqual(adapter.formalAiSource, sourceIdentity, 'operational adapter source identity');
  const safePath = value => {
    assert.equal(typeof value, 'string');
    assert.match(value, /^src\/[A-Za-z0-9_-][A-Za-z0-9_.-]*\.mjs$/u, 'owned image source path required');
    assert.ok(!value.includes('..'), 'image source traversal');
    return value;
  };
  const helperPath = safePath(adapter.helperPath);
  const sha = bytes => createHash('sha256').update(bytes).digest('hex');
  const helper = fs.readFileSync(path.join(hive, helperPath));
  assert.equal(sha(helper), adapter.helperSha256, 'prepared helper digest');
  assert.ok(Array.isArray(adapter.sources) && adapter.sources.length > 0 && adapter.sources.length <= 8);
  const owners = adapter.sources.map(row => {
    const owner = safePath(row.path);
    assert.notEqual(owner, helperPath);
    assert.equal(sha(fs.readFileSync(path.join(hive, owner))), row.adaptedSha256, 'prepared owner digest');
    return owner;
  });
  assert.equal(new Set(owners).size, owners.length, 'unique adapted owners');
  const actualImageHelperSha256 = read('docker',
 ['run',
 '--rm',
 '--entrypoint',
 'sha256sum',
 'hive-e2e:candidate',
 '/opt/hive-e2e/' + helperPath]).split(/\s/u)[0];
  assert.equal(actualImageHelperSha256, adapter.helperSha256, 'actual task image operational helper digest');
  const imports = owners.map(owner => 'file:///opt/hive-e2e/' + owner);
  run('docker',
 ['run',
 '--rm',
 '--entrypoint',
 'node',
 'hive-e2e:candidate',
 '--input-type=module',
 '--eval',
 'for (const file of ' + JSON.stringify(imports) + ') await import(file);']);
  return {
    ...adapter,
    actualImageHelperSha256,
    adaptedOwnerImports: 'passed'
  };
}
export function verifyTaskImage({
  hive,
  expected,
  inputs,
  read,
  run
}) {
  const observed = read('docker', ['run', '--rm', '--entrypoint', 'sha256sum', 'hive-e2e:candidate', '/usr/local/bin/formal-ai']).split(/\s/u)[0];
  assert.equal(observed, expected.binarySha256, 'actual task image must run this source-qualified binary');
  run('docker', ['run', '--rm', '--entrypoint', 'java', 'hive-e2e:candidate', '-version']);
  run('docker',
 ['run',
 '--rm',
 '--entrypoint',
 '/bin/sh',
 'hive-e2e:candidate',
 '-c',
 'test -x "$JAVA_HOME/bin/java" && test "$(readlink -f "$JAVA_HOME/bin/java")" = "$(readlink -f "$(command -v java)")"']);
  const operationalEvidence = verifyTaskImageOperationalEvidence({
    hive,
    sourceIdentity: {
      commit: expected.source,
      tree: expected.sourceTree
    },
    hiveCommit: inputs.hiveCommit,
    read,
    run
  });
  const transportAdapter = JSON.parse(fs.readFileSync(path.join(hive, '.formal-ai-transport-adapter.json')));
  assert.equal(transportAdapter.upstreamCommit, inputs.hiveCommit);
  // The original task Dockerfile copies src only; the transport entrypoint stays on the host.
  for (const [member,
 digest] of [['scripts/e2e-hello-world.mjs',
 transportAdapter.adaptedSha256],
 ['scripts/formal-ai-hive-delivery-evidence.mjs',
 transportAdapter.helperSha256]]) assert.equal(fileDigest(path.join(hive,
 member)),
 digest,
 'prepared transport source digest');
  const imageMembers = operationalEvidence.sources.map(row => [row.path, row.adaptedSha256]);
  for (const [member, digest] of imageMembers) {
    assert.equal(fileDigest(path.join(hive, member)), digest, 'prepared source member digest');
    assert.equal(read('docker',
 ['run',
 '--rm',
 '--entrypoint',
 'sha256sum',
 'hive-e2e:candidate',
 '/opt/hive-e2e/' + member]).split(/\s/u)[0],
 digest,
 'actual image adapted source digest');
  }
  return {
    actualTaskImageBinarySha256: observed,
    transportAdapter,
    operationalEvidence
  };
}

/** Extract shipped bytes from an immutable published filesystem; no candidate compiler runs in this phase. */
export function preparePublishedBinary({
  root = process.cwd(),
  environment = process.env,
  run = execFileSync
} = {}) {
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  const selection = JSON.parse(fs.readFileSync(environment.HIVE_PUBLISHED_RUNTIME_FILE));
  assert.equal(selection.admitted, true);
  assert.equal(selection.producer.run, environment.GITHUB_RUN_ID);
  assert.equal(selection.producer.attempt, environment.GITHUB_RUN_ATTEMPT);
  const runtime = selection.publishedRuntime;
  assert.match(runtime.image, /^ghcr\.io\/link-assistant\/formal-ai@sha256:[a-f0-9]{64}$/u);
  const invoke = (command, args, options = {}) => run(command, args, {
    cwd: root,
    timeout: 1200000,
    maxBuffer: 1048576,
    ...options
  });
  assert.equal(invoke('git', ['rev-parse', 'HEAD'], {
    encoding: 'utf8'
  }).trim(), runtime.source);
  invoke('docker', ['pull', runtime.image], {
    stdio: 'inherit'
  });
  const container = invoke('docker', ['create', runtime.image], {
    encoding: 'utf8'
  }).trim();
  assert.match(container, /^[a-f0-9]{64}$/u);
  let primary;
  try {
    const directory = path.join(root, 'rust/target/release');
    fs.mkdirSync(directory, {
      recursive: true
    });
    const destination = path.join(directory, 'formal-ai');
    assert.equal(fs.existsSync(destination), false, 'fresh published binary destination');
    invoke('docker', ['cp', container + ':/usr/local/bin/formal-ai', destination], {
      stdio: 'inherit'
    });
    assert.equal(fileDigest(destination), runtime.binarySha256);
    fs.chmodSync(destination, 0o755);
  } catch (error) {
    primary = error;
    throw error;
  } finally {
    try {
      invoke('docker', ['rm', '--force', container], {
        stdio: 'inherit',
        timeout: 30000
      });
    } catch (cleanup) {
      if (primary) primary.message += '; container cleanup: ' + cleanup.message;else throw cleanup;
    }
  }
  return runtime;
}
export function main() {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'hosted image construction only');
  if (process.argv[2] === 'extract-published-binary') {
    preparePublishedBinary();
    return;
  }
  const mode = process.argv[2] ?? 'build';
  assert.ok(['build', 'load'].includes(mode), 'explicit image phase required');
  const root = process.cwd(),
    hive = path.resolve('.hive-mind');
  const inputs = JSON.parse(fs.readFileSync('data/benchmarks/hive-original-delivery-inputs.json'));
  const run = (command, args, options = {}) => execFileSync(command, args, {
    cwd: hive,
    stdio: 'inherit',
    timeout: 1200000,
    ...options
  });
  const read = (command, args) => run(command, args, {
    encoding: 'utf8',
    stdio: 'pipe'
  }).trim();
  assert.equal(read('git', ['rev-parse', 'HEAD']), inputs.hiveCommit, 'pinned Hive source');
  const expected = sourceBinaryIdentity({
    root
  });
  const directory = process.env.HIVE_IMAGE_BUNDLE_DIR;
  assert.ok(directory && path.isAbsolute(directory), 'explicit hosted image bundle directory required');
  const destination = process.env.E2E_LOG_DIR;
  assert.ok(destination && path.isAbsolute(destination));
  let identity;
  if (mode === 'load') {
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, 'image-bundle.json')));
    const transferred = requireImageBundle(manifest, {
      directory,
      expected,
      inputs
    });
    run('docker', ['load', '--input', path.join(directory, 'candidate.tar.gz')]);
    const loaded = JSON.parse(read('docker', ['image', 'inspect', 'hive-e2e:candidate']))[0];
    assert.equal(loaded.Id, manifest.imageId, 'loaded content-addressed task image');
    const observed = verifyTaskImage({
      hive,
      expected,
      inputs,
      read,
      run
    });
    for (const key of Object.keys(observed)) assert.deepEqual(observed[key], transferred[key], 'loaded image ' + key);
    identity = transferred;
  } else {
    const binary = path.join(root, 'rust/target/release/formal-ai'),
      target = path.join(hive, 'formal-ai-pr-binary');
    if (!expected.publishedRuntime) {
      fs.copyFileSync(binary, target);
      fs.chmodSync(target, 0o755);
    }
    const dockerfile = path.join(hive, 'Dockerfile.formal-ai-pr'),
      helpers = path.join(hive, 'formal-ai-build');
    fs.mkdirSync(helpers, {
      recursive: true
    });
    for (const name of ['apt-install-with-retry.sh', 'run-with-deadline.sh']) fs.copyFileSync(path.join(root, 'scripts', name), path.join(helpers, name));
    const runtimeImage = expected.publishedRuntime?.image ?? inputs.taskRuntimeImage;
    fs.writeFileSync(dockerfile, taskRuntimeDockerfile(runtimeImage, {
      published: !!expected.publishedRuntime
    }));
    run('docker', ['pull', runtimeImage]);
    const runtime = JSON.parse(read('docker', ['image', 'inspect', runtimeImage]))[0];
    run('docker', ['build', '-f', dockerfile, '-t', 'hive-e2e:task', '.']);
    run('docker', ['build', '-f', 'Dockerfile.e2e', '-t', 'hive-e2e:candidate', '.']);
    identity = {
      ...expected,
      hiveCommit: inputs.hiveCommit,
      runtimeImage,
      runtimeImageId: runtime.Id,
      runtimeImageDigests: runtime.RepoDigests,
      ...verifyTaskImage({
        hive,
        expected,
        inputs,
        read,
        run
      }),
      sameRunBinary: true,
      runtimeBootstrap: {
        package: 'default-jre-headless',
        scope: 'image-build',
        javaHome: '/usr/local/lib/formal-ai-jvm',
        compilerPackagesAdded: []
      },
      authorshipCredit: 0
    };
    fs.mkdirSync(directory, {
      recursive: true
    });
    const archive = path.join(directory, 'candidate.tar.gz');
    assert.equal(fs.existsSync(archive), false, 'fresh image bundle required');
    // Stream through gzip: never buffer a Docker image in the bounded Node process or keep a second raw tar.
    run('bash', ['-e', '-o', 'pipefail', '-c', 'docker save hive-e2e:candidate | gzip -1 > "$HIVE_IMAGE_ARCHIVE"'], {
      env: {
        ...process.env,
        HIVE_IMAGE_ARCHIVE: archive
      }
    });
    const image = JSON.parse(read('docker', ['image', 'inspect', 'hive-e2e:candidate']))[0];
    fs.writeFileSync(path.join(directory, 'image-bundle.json'), JSON.stringify({
      schema: 'formal-ai-hive-image-bundle/v1',
      imageId: image.Id,
      archiveSha256: fileDigest(archive),
      identity
    }, null, 2) + '\n', {
      flag: 'wx'
    });
  }
  fs.mkdirSync(destination, {
    recursive: true
  });
  fs.writeFileSync(path.join(destination, 'source-binary-identity.json'), JSON.stringify(identity, null, 2) + '\n');
  console.log('Actual task image, binary and reviewed adapters verified against this source and workflow run; phase=' + mode);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
