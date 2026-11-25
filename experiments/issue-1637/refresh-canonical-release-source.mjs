#!/usr/bin/env node
// Issue #1637: release.yml is a projection of the canonical release source,
// so a hand edit of release.yml fails `generate-staged-release.mjs --check-deployed`
// ("deployed caller differs from checked original projection").
// Usage: node experiments/issue-1637/refresh-canonical-release-source.mjs <new-canonical.yml>
// Writes the partitioned canonical source, refreshes the packet digest, writes
// every staged output to its repository path, then runs both generator checks.
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {join, resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {partitionCanonicalSource, CANONICAL_PREFIX} from '../../scripts/canonical-release-source.mjs';
import {buildStagedReleaseProjection} from '../../scripts/generate-staged-release.mjs';
const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const text = readFileSync(process.argv[2], 'utf8');
const files = partitionCanonicalSource(text);
for (const file of files) writeFileSync(join(root, file.path), file.text);
const packetPath = join(root, CANONICAL_PREFIX, 'stage-source-coverage.json');
const raw = readFileSync(packetPath, 'utf8');
writeFileSync(packetPath, raw.replace(JSON.parse(raw).workflowSha256, createHash('sha256').update(text).digest('hex')));
const packet = JSON.parse(readFileSync(packetPath, 'utf8'));
const {outputs} = buildStagedReleaseProjection(text, packet, {partitioned: files.length === 3});
for (const [path, bytes] of outputs) writeFileSync(join(root, path), bytes);
writeFileSync(join(root, '.github/workflows/release.yml'), outputs.get(CANONICAL_PREFIX + 'candidate-release-caller.yml'));
execFileSync(process.execPath, ['scripts/generate-staged-release.mjs', '--check-deployed'], {cwd: root, stdio: 'inherit'});
