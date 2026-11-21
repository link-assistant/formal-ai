// Verify a complete paired witness delta consists only of reviewed source hashes.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {deserialize} from 'node:v8';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const prefix = process.argv[2];
const sourceBase = process.argv[3];
assert.match(sourceBase || "", /^[0-9a-f]{40}$/u, "Immutable source base required");
const before = deserialize(fs.readFileSync(prefix + '.before.v8'));
const after = deserialize(fs.readFileSync(prefix + '.after.v8'));
// One source file was split to retain the repository line limit. Verify its test body.
const source = 'rust/tests/unit/specification/openai_compatibility.rs';
const destination = 'rust/tests/unit/specification/openai_compatibility/output.rs';
const name = 'chat_completion_allows_declared_tools_when_tool_choice_is_none';
const oldSource = execFileSync('git', ['show', sourceBase + ':' + source], {encoding: 'utf8'});
const newSource = fs.readFileSync(destination, 'utf8');
const signature = 'fn ' + name + '() {';
assert.equal(newSource.slice(newSource.indexOf(signature)).trim(), oldSource.slice(oldSource.indexOf(signature)).trim());
const moved = after.inventory.unsupported.find(item => item.id === destination + '::' + name);
assert(moved);
assert.deepEqual({...moved, id: source + '::' + name}, before.inventory.unsupported.find(item => item.id === source + '::' + name));
moved.id = source + '::' + name;
const sourceMap = new Map(after.sourceManifest.map(record => [record.path, record]));
assert.equal(after.sourceManifest.length, before.sourceManifest.length + 1);
assert.deepEqual([...sourceMap.keys()].filter(path => !before.sourceManifest.some(record => record.path === path)), [destination]);
after.sourceManifest = before.sourceManifest.map(record => { assert(sourceMap.has(record.path)); return sourceMap.get(record.path); });
const pairs = new WeakMap(), changes = [], sources = new Map();
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
let visited = 0;
function visit(left, right, path, parent) {
  if (Object.is(left, right)) return;
  if (!left || !right || typeof left !== 'object' || typeof right !== 'object') {
    assert.equal(path.at(-1), 'sha256', 'Non-source-identity delta: ' + path.join('/'));
    const file = (parent.path || parent.file || '').replace(/^repo:/u, '');
    assert.match(file, /^(rust|desktop|js|vscode)\/(?!.*\.\.)/u);
    assert.equal(right, hash(fs.readFileSync(file)), 'Current source hash: ' + file);
    assert.equal(left, hash(execFileSync('git', ['show', sourceBase + ':' + file])), 'Previous source hash: ' + file);
    sources.set(file, {before: left, after: right, count: (sources.get(file)?.count || 0) + 1});
    changes.push({path: path.join('/'), file, before: left, after: right});
    return;
  }
  let matches = pairs.get(left);
  if (!matches) { matches = new WeakSet(); pairs.set(left, matches); }
  if (matches.has(right)) return;
  matches.add(right);
  if (++visited > 200000) throw Error('WitnessComparisonBoundExceeded');
  assert.equal(Array.isArray(left), Array.isArray(right));
  assert.deepEqual(Object.keys(left), Object.keys(right), 'Record shape/order: ' + path.join('/'));
  for (const key of Object.keys(left)) visit(left[key], right[key], [...path, key], left);
}
visit(before, after, [], null);
fs.writeFileSync(prefix + '.complete-delta.jsonl', changes.map(change => JSON.stringify(change)).join('\n') + '\n');
console.log(JSON.stringify({reviewedMove: {source, destination, name, bodyUnchanged: true}, changes: changes.length, visited, sources: [...sources].map(([path, detail]) => ({path, ...detail}))}, null, 2));
