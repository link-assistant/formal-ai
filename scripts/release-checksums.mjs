import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
const sharedNames = new Set(['latest.yml', 'latest-mac.yml', 'latest-linux.yml']);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export function consolidateChecksums(fragments, resolveSharedMetadata) {
  assert.ok(Array.isArray(fragments) && fragments.length > 0);
  const records = new Map();
  let total = 0;
  for (const bytes of fragments) {
    assert.ok(Buffer.isBuffer(bytes) && bytes.length > 0);
    total += bytes.length;
    assert.ok(total <= 1024 * 1024, 'checksum fragments exceed budget');
    const text = new TextDecoder('utf-8', {fatal: true}).decode(bytes).trim();
    for (const line of text.split(/\r?\n/u)) {
      const match = /^([a-f0-9]{64}) [ *]([^/\\\r\n]+)$/u.exec(line);
      assert.ok(match, 'invalid checksum fragment row');
      const hashes = records.get(match[2]) || new Set();
      hashes.add(match[1]); records.set(match[2], hashes);
    }
  }
  return [...records.keys()].sort().map(name => {
    const hashes = records.get(name);
    let selected = [...hashes][0];
    if (hashes.size > 1) {
      assert.ok(sharedNames.has(name), 'conflicting artifact checksums: ' + name);
      selected = resolveSharedMetadata(name);
      assert.match(selected, /^[a-f0-9]{64}$/u);
      assert.ok(hashes.has(selected), 'published updater is absent from producer fragments');
    }
    return selected + '  ' + name + '\n';
  }).join('');
}
export function publishedMetadataResolver(environment, read) {
  const repository = environment.REPO, tag = environment.TAG;
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(tag, /^v[0-9]+\.[0-9]+\.[0-9]+$/u);
  let release;
  return name => {
    assert.ok(sharedNames.has(name), 'only shared updater metadata may be reconciled');
    if (!release) {
      release = JSON.parse(read(['api', 'repos/' + repository + '/releases/tags/' + tag], 512 * 1024));
      assert.equal(release.tag_name, tag);
      assert.equal(release.draft, false); assert.equal(release.prerelease, false);
      assert.ok(Array.isArray(release.assets) && release.assets.length < 100);
    }
    const matches = release.assets.filter(asset => asset.name === name);
    assert.equal(matches.length, 1, 'one published updater asset required');
    const asset = matches[0];
    assert.match(String(asset.id), /^[1-9][0-9]*$/u);
    assert.equal(asset.state, 'uploaded');
    assert.ok(asset.size > 0 && asset.size <= 256 * 1024);
    assert.match(asset.digest, /^sha256:[a-f0-9]{64}$/u);
    const bytes = read(['api', '-H', 'Accept: application/octet-stream',
      'repos/' + repository + '/releases/assets/' + asset.id], 256 * 1024);
    assert.ok(Buffer.isBuffer(bytes)); assert.equal(bytes.length, asset.size);
    assert.equal('sha256:' + digest(bytes), asset.digest, 'published updater digest mismatch');
    return digest(bytes);
  };
}
export function main(arguments_, environment = process.env) {
  const fragments = arguments_.map(file => {
    const stat = fs.lstatSync(file);
    assert.ok(stat.isFile() && !stat.isSymbolicLink());
    assert.ok(stat.size > 0 && stat.size <= 1024 * 1024);
    return fs.readFileSync(file);
  });
  const read = (args, maxBuffer) => execFileSync('gh', args, {
    timeout: 30000, maxBuffer, stdio: ['ignore', 'pipe', 'pipe']
  });
  return consolidateChecksums(fragments, publishedMetadataResolver(environment, read));
}
if (import.meta.url === pathToFileURL(process.argv[1] || '').href) {
  process.stdout.write(main(process.argv.slice(2)));
}
