#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
assert.equal(process.env.GITHUB_ACTIONS, 'true', 'hosted image construction only');
const root = process.cwd();
const hive = path.resolve('.hive-mind');
const inputs = JSON.parse(fs.readFileSync('data/benchmarks/hive-original-delivery-inputs.json'));
const run = (command, argumentsList, options = {}) => execFileSync(command, argumentsList,
  {cwd: hive, stdio: 'inherit', timeout: 1200000, ...options});
const read = (command, argumentsList) => run(command, argumentsList, {encoding: 'utf8', stdio: 'pipe'}).trim();
assert.equal(read('git', ['rev-parse', 'HEAD']), inputs.hiveCommit, 'pinned Hive source');
const source = read('git', ['-C', root, 'rev-parse', 'HEAD']);
const binary = path.join(root, 'rust/target/release/formal-ai');
const binarySha256 = createHash('sha256').update(fs.readFileSync(binary)).digest('hex');
const target = path.join(hive, 'formal-ai-pr-binary');
fs.copyFileSync(binary, target);
fs.chmodSync(target, 0o755);
const dockerfile = path.join(hive, 'Dockerfile.formal-ai-pr');
fs.writeFileSync(dockerfile, [
  'FROM ' + inputs.taskRuntimeImage,
  'USER root',
  'COPY formal-ai-pr-binary /usr/local/bin/formal-ai',
  'RUN chmod 0755 /usr/local/bin/formal-ai && formal-ai --version',
  'ENV FORMAL_AI_INSTALL_GRANT=workspace',
  'USER box',
  'ENV HOME=/home/box FORMAL_AI_MEMORY_PATH=/home/box/.formal-ai/memory.lino',
  'RUN mkdir -p /home/box/.formal-ai',
  'CMD ["formal-ai","serve","--agent-mode","--host","0.0.0.0","--port","8080"]',
].join('\n') + '\n');
run('docker', ['pull', inputs.taskRuntimeImage]);
const imageIdentity = JSON.parse(read('docker', ['image', 'inspect', inputs.taskRuntimeImage]))[0];
run('docker', ['build', '-f', dockerfile, '-t', 'hive-e2e:task', '.']);
run('docker', ['build', '-f', 'Dockerfile.e2e', '-t', 'hive-e2e:candidate', '.']);
const observed = read('docker', ['run', '--rm', '--entrypoint', 'sha256sum',
  'hive-e2e:candidate', '/usr/local/bin/formal-ai']).split(/\s/u)[0];
assert.equal(observed, binarySha256, 'actual task image must run this source-qualified binary');
const destination = process.env.E2E_LOG_DIR;
assert(destination);
fs.mkdirSync(destination, {recursive: true});
fs.writeFileSync(path.join(destination, 'source-binary-identity.json'), JSON.stringify({
  source, binarySha256, hiveCommit: inputs.hiveCommit,
  runtimeImage: inputs.taskRuntimeImage, runtimeImageId: imageIdentity.Id,
  runtimeImageDigests: imageIdentity.RepoDigests, actualTaskImageBinarySha256: observed,
  sameRunBinary: true, authorshipCredit: 0,
}) + '\n');
console.log('Actual task image binary verified against this Formal AI source build.');
