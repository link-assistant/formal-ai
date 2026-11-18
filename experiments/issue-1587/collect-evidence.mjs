// Retain supplied procedures and complete observations without crediting them
// as independent Formal AI reasoning. Run after validation with the log directory.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';

const source = process.argv[2];
if (!source) throw new Error('Usage: node experiments/issue-1587/collect-evidence.mjs LOG_DIRECTORY');
const directory = path.dirname(new URL(import.meta.url).pathname);
const hashes = {};
const sourceHashes = {};
for (const file of ['experiments/js_dogfood/drive.mjs', 'js/server/worker-host.mjs',
  'js/server/request-context.mjs', 'rust/src/protocol/request_context.rs']) {
  sourceHashes[file] = createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}
const procedures = [];
for (const name of fs.readdirSync(source).filter(name => name.endsWith('.log')).sort()) {
  const bytes = fs.readFileSync(path.join(source, name));
  hashes[name] = createHash('sha256').update(bytes).digest('hex');
  fs.writeFileSync(path.join(directory, `${name}.gz`), gzipSync(bytes, { level: 9 }));
  procedures.push(...bytes.toString().split('\n').filter(line => line.startsWith('>> bash ')));
}
fs.writeFileSync(path.join(directory, 'supplied-procedures.txt.gz'), gzipSync(procedures.join('\n') + '\n'));
fs.writeFileSync(path.join(directory, 'provenance.json'), JSON.stringify({
  runtime: 'JavaScript Formal AI via experiments/js_dogfood/drive.mjs',
  runtime_and_source_sha256: sourceHashes,
  externally_authored: true,
  frozen_reasoning_units: 8,
  attempted_reasoning_units: 1,
  attempted_reasoning_share: 0.125,
  accepted_reasoning_units: 0,
  accepted_reasoning_share: 0,
  accepted_authored_leaves: 0,
  failed_attempt: 'authorship-attempt.log.gz',
  implementation_request: 'Update js/server/openai.mjs so system and developer message constraints affect the current completion without persisting them to another request.',
  note: 'Supplied commands, code, tests, procedures and conclusions earn zero independent reasoning or authorship credit. Native tests are registered but execution remains unverified under the main-only native-test policy; logs here describe local observations only.',
  uncompressed_sha256: hashes,
}, null, 2) + '\n');
