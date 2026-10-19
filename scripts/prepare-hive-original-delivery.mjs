#!/usr/bin/env node
// Narrow transport adapter: the pinned Hive task and all original oracles stay byte-exact.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {prepareHiveOperationalEvidence} from './lib/hive-operational-evidence.mjs';

const ENTRYPOINT_BLOB = '6bec72e1bc03104a05ff9588ca2964197e13247a';
export function adaptHiveEntrypoint(source) {
  const bytes = Buffer.from(source);
  const blob = createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex');
  assert.equal(blob, ENTRYPOINT_BLOB, 'unreviewed Hive entrypoint bytes');
  const imported = "import { verifyGeneratedWorkflow } from './e2e-workflow.lib.mjs';";
  const returned = "stdout: results.map(result => result.stdout).join('\\n')";
  assert.equal(source.split(imported).length, 2);
  assert.equal(source.split(returned).length, 2);
  return source.replace(imported, imported + "\nimport { decodeActWorkflowLog } from './formal-ai-hive-delivery-evidence.mjs';")
    .replace(returned, "stdout: decodeActWorkflowLog(results.map(result => result.stdout).join('\\n'))");
}

export function main() {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'hosted Hive preparation only');
  const root = process.cwd(), hive = path.join(root, '.hive-mind');
  const inputs = JSON.parse(fs.readFileSync(path.join(root, 'data/benchmarks/hive-original-delivery-inputs.json')));
  const head = execFileSync('git', ['rev-parse', 'HEAD'], {cwd: hive, encoding: 'utf8', timeout: 30000}).trim();
  assert.equal(head, inputs.hiveCommit, 'pinned Hive source');
  const entrypoint = path.join(hive, 'scripts/e2e-hello-world.mjs');
  const original = fs.readFileSync(entrypoint, 'utf8');
  const adapted = adaptHiveEntrypoint(original);
  prepareHiveOperationalEvidence({hive, root});
  const helper = fs.readFileSync(path.join(root, 'scripts/lib/hive-delivery-evidence.mjs'));
  const digest = bytes => createHash('sha256').update(bytes).digest('hex');
  fs.writeFileSync(path.join(hive, 'scripts/formal-ai-hive-delivery-evidence.mjs'), helper, {flag: 'wx'});
  fs.writeFileSync(entrypoint, adapted);
  fs.writeFileSync(path.join(hive, '.formal-ai-transport-adapter.json'), JSON.stringify({
    schema: 'formal-ai-reviewed-hive-transport/v1', upstreamCommit: head,
    upstreamEntrypointBlob: ENTRYPOINT_BLOB, originalSha256: digest(original),
    adaptedSha256: digest(adapted), helperSha256: digest(helper),
    originalOracleFilesChanged: false, taskChanged: false, authorshipCredit: 0,
  }, null, 2) + '\n', {flag: 'wx'});
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
