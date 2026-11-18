// Issue #1486: the Hive task image travels as a delta over its digest-pinned
// runtime base. These checks use synthetic `docker save` archives and a fake
// `docker` on PATH, so they need neither Docker nor a Rust build.
import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { Readable, Writable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { gunzipSync } from 'node:zlib';
import { deltaPlan, paxRecords, TarDeltaFilter, verifyDelta } from '../../../scripts/lib/oci-delta-archive.mjs';
import { pinnedBaseRef, requirePinnedBase, saveImageArchive } from '../../../scripts/build-hive-original-delivery-image.mjs';
import { IMAGE_BUNDLE_SCHEMA, requireImageTransport } from '../../../scripts/lib/hive-delivery-bundle.mjs';

const digest = text => 'sha256:' + createHash('sha256').update(text).digest('hex');
const hex = value => value.slice('sha256:'.length);

function header(name, size, type = '0') {
  const block = Buffer.alloc(512);
  block.write(name.slice(0, 100), 0);
  block.write('0000644\0', 100);
  block.write('0000000\0', 108);
  block.write('0000000\0', 116);
  block.write(size.toString(8).padStart(11, '0') + '\0', 124);
  block.write('00000000000\0', 136);
  block.write('        ', 148);
  block.write(type, 156);
  block.write('ustar\0' + '00', 257);
  let sum = 0;
  for (const byte of block) sum += byte;
  block.write(sum.toString(8).padStart(6, '0') + '\0 ', 148);
  return block;
}
const pad = body => Buffer.concat([body, Buffer.alloc((512 - (body.length % 512)) % 512)]);
function member(name, body, { pax = false } = {}) {
  const bytes = Buffer.from(body);
  const parts = [];
  if (pax) {
    let record = ' path=' + name + '\n';
    let length = record.length;
    while (String(length).length + record.length !== length) length = String(length).length + record.length;
    const paxBody = Buffer.from(length + record);
    parts.push(header('PaxHeaders/x', paxBody.length, 'x'), pad(paxBody));
  }
  parts.push(header(pax ? 'placeholder-name' : name, bytes.length), pad(bytes));
  return Buffer.concat(parts);
}

/** A synthetic image archive in the shape `docker save` writes with the classic store. */
function imageArchive({ base = ['base-one', 'base-two'], added = ['task-layer', 'e2e-layer'], pax = false, extraImage = false } = {}) {
  const baseIds = base.map(digest);
  const addedIds = added.map(digest);
  const config = JSON.stringify({ rootfs: { type: 'layers', diff_ids: [...baseIds, ...addedIds] } });
  const configId = digest(config);
  const layers = [...baseIds, ...addedIds].map(id => 'blobs/sha256/' + hex(id));
  const images = [{ Config: 'blobs/sha256/' + hex(configId), RepoTags: ['hive-e2e:candidate'], Layers: layers }];
  if (extraImage) images.push(images[0]);
  const members = [
    member('blobs/', ''),
    ...[...base, ...added].map(text => member('blobs/sha256/' + hex(digest(text)), text.repeat(300), { pax: pax && base.includes(text) })),
    member('blobs/sha256/' + hex(configId), config),
    member('index.json', '{"schemaVersion":2}'),
    member('manifest.json', JSON.stringify(images)),
    member('oci-layout', '{"imageLayoutVersion":"1.0.0"}'),
  ];
  return { bytes: Buffer.concat([...members, Buffer.alloc(1024)]), baseIds, addedIds, candidateIds: [...baseIds, ...addedIds], imageId: configId };
}
const baseRef = 'konard/hive-mind-dind@sha256:' + 'a'.repeat(64);
const planOf = archive => deltaPlan({ baseImageId: digest('base config'), baseRef, baseDiffIds: archive.baseIds, candidateDiffIds: archive.candidateIds });

async function filtered(bytes, drop, chunk = 4096) {
  const filter = new TarDeltaFilter(drop);
  const output = [];
  const chunks = [];
  for (let offset = 0; offset < bytes.length; offset += chunk) chunks.push(bytes.subarray(offset, offset + chunk));
  await pipeline(Readable.from(chunks), filter, new Writable({ write(part, _encoding, done) { output.push(part); done(); } }));
  return { bytes: Buffer.concat(output), report: filter.report };
}
function listTar(bytes) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'delta-tar-'));
  try {
    fs.writeFileSync(path.join(directory, 'image.tar'), bytes);
    return execFileSync('tar', ['-tf', path.join(directory, 'image.tar')], { encoding: 'utf8' }).split('\n').filter(Boolean).sort();
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test('the delta plan drops only the inherited, digest-pinned base layers', () => {
  const archive = imageArchive();
  const plan = planOf(archive);
  assert.deepEqual(plan.drop, archive.baseIds.map(id => 'blobs/sha256/' + hex(id)).sort());
  assert.equal(plan.baseLayers, 2);
  assert.equal(plan.candidateLayers, 4);
  assert.throws(() => deltaPlan({ ...plan, baseImageId: plan.baseImageId, baseRef: 'konard/hive-mind-dind:latest', baseDiffIds: archive.baseIds, candidateDiffIds: archive.candidateIds }), /DeltaBaseNotPinned/);
  assert.throws(() => deltaPlan({ baseImageId: plan.baseImageId, baseRef, baseDiffIds: [digest('other base')], candidateDiffIds: archive.candidateIds }), /DeltaBaseNotPrefix/);
  assert.throws(() => deltaPlan({ baseImageId: plan.baseImageId, baseRef, baseDiffIds: archive.candidateIds, candidateDiffIds: archive.candidateIds }), /DeltaBaseNotPrefix/, 'a candidate that adds nothing is not a delta');
  assert.throws(() => deltaPlan({ baseImageId: 'latest', baseRef, baseDiffIds: archive.baseIds, candidateDiffIds: archive.candidateIds }), /DeltaBaseUnknown/);
  // A layer the candidate itself repeats (an empty layer, say) is never dropped.
  const repeated = deltaPlan({ baseImageId: plan.baseImageId, baseRef, baseDiffIds: archive.baseIds, candidateDiffIds: [...archive.candidateIds, archive.baseIds[0]] });
  assert.deepEqual(repeated.drop, ['blobs/sha256/' + hex(archive.baseIds[1])]);
  assert.equal(pinnedBaseRef('konard/hive-mind-dind:latest', ['other/image@sha256:' + 'b'.repeat(64), baseRef]), baseRef);
  assert.equal(pinnedBaseRef('konard/hive-mind-dind:latest', []), null, 'an unpinned base cannot carry a delta');
});

test('the streamed filter emits a valid tar without the base blobs, in any chunking', async () => {
  for (const pax of [false, true]) {
    const archive = imageArchive({ pax });
    const plan = planOf(archive);
    const results = [];
    for (const chunk of [7, 512, 1000, 1 << 20]) results.push(await filtered(archive.bytes, plan.drop, chunk));
    for (const result of results.slice(1)) assert.deepEqual(result.bytes, results[0].bytes, 'chunking does not change the delta');
    const { bytes, report } = results[0];
    report.ended = true;
    assert.deepEqual(verifyDelta(plan, report), []);
    assert.deepEqual(report.dropped.map(row => row.name).sort(), plan.drop);
    assert.equal(report.keptBytes, bytes.length);
    assert.equal(report.keptBytes + report.droppedBytes, archive.bytes.length, 'every input byte is accounted for');
    const listed = listTar(bytes);
    for (const name of plan.drop) assert.ok(!listed.includes(name), name + ' dropped');
    for (const id of archive.addedIds) assert.ok(listed.includes('blobs/sha256/' + hex(id)), 'added layer kept');
    assert.ok(listed.includes('manifest.json') && listed.includes('index.json'));
    assert.ok(!listed.some(name => name.includes('PaxHeaders')), 'pax headers travel with their member');
    assert.ok(report.droppedBytes >= 'base-one'.repeat(300).length + 'base-two'.repeat(300).length, 'the base layer bytes are not transferred');
  }
  assert.deepEqual(paxRecords(Buffer.from('23 path=blobs/sha256/a\n15 uid=1234567\n')), { path: 'blobs/sha256/a', uid: '1234567' });
});

test('an inconsistent delta is rejected instead of being transferred', async () => {
  const archive = imageArchive();
  const plan = planOf(archive);
  const verdict = async (bytes, drop = plan.drop, tamper = report => report) => {
    const { report } = await filtered(bytes, drop);
    report.ended = true;
    return verifyDelta(plan, tamper(report));
  };
  // Dropping an added layer would leave the consumer unable to rebuild the candidate.
  assert.match((await verdict(archive.bytes, [...plan.drop, 'blobs/sha256/' + hex(archive.addedIds[0])])).join('\n'), /is not an inherited base layer/u);
  assert.match((await verdict(archive.bytes, plan.drop, report => ({ ...report, kept: report.kept.filter(row => !row.name.endsWith(hex(archive.addedIds[1]))) }))).join('\n'), /added layer .* missing/u);
  assert.match((await verdict(archive.bytes, plan.drop, report => ({ ...report, kept: report.kept.filter(row => !row.name.endsWith(hex(archive.imageId))) }))).join('\n'), /config .* missing/u);
  assert.match((await verdict(imageArchive({ extraImage: true }).bytes)).join('\n'), /exactly one image/u);
  assert.match((await verdict(archive.bytes, plan.drop, report => ({ ...report, ended: false }))).join('\n'), /did not end/u);
  assert.match((await verdict(imageArchive({ added: ['task-layer'] }).bytes)).join('\n'), /manifest has 3 layers, the candidate 4/u);
  await assert.rejects(filtered(archive.bytes.subarray(0, 2048), plan.drop), /DeltaTarTruncated/u);
  const corrupt = Buffer.from(archive.bytes);
  corrupt[0] ^= 1;
  await assert.rejects(filtered(corrupt, plan.drop), /DeltaTarChecksum/u);
});

function fakeDocker(archiveBytes) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'fake-docker-'));
  fs.writeFileSync(path.join(directory, 'image.tar'), archiveBytes);
  fs.writeFileSync(path.join(directory, 'docker'), '#!/bin/sh\n[ "$1" = save ] && exec cat "' + path.join(directory, 'image.tar') + '"\nexit 64\n', { mode: 0o755 });
  const run = (command, args, options = {}) => execFileSync(command, args, { stdio: 'pipe', ...options, env: { ...options.env, PATH: directory + ':' + process.env.PATH } });
  return { directory, run, archive: path.join(directory, 'candidate.tar.gz'), cleanup: () => fs.rmSync(directory, { recursive: true, force: true }) };
}

test('the image job writes a pinned delta, and falls back to the full archive when it cannot', () => {
  const archive = imageArchive();
  const runtime = { Id: digest('base config'), RepoDigests: [baseRef], RootFS: { Layers: archive.baseIds } };
  const image = { Id: archive.imageId, RootFS: { Layers: archive.candidateIds } };
  const delta = fakeDocker(archive.bytes);
  try {
    const transport = saveImageArchive({ archive: delta.archive, runtimeImage: 'konard/hive-mind-dind:latest', runtime, image, run: delta.run, scratch: delta.directory });
    assert.equal(transport.mode, 'delta');
    assert.deepEqual(transport.base, { ref: baseRef, imageId: runtime.Id, diffIds: archive.baseIds });
    assert.equal(transport.droppedLayers, 2);
    const listed = listTar(gunzipSync(fs.readFileSync(delta.archive)));
    assert.ok(!listed.includes('blobs/sha256/' + hex(archive.baseIds[0])));
    assert.ok(listed.includes('blobs/sha256/' + hex(archive.addedIds[1])));
    assert.deepEqual(requireImageTransport(transport, { runtimeImageId: runtime.Id, runtimeImageDigests: [baseRef] }), transport);
  } finally {
    delta.cleanup();
  }
  // A base the candidate does not inherit (a moved tag, a different runtime) never yields a delta.
  const moved = fakeDocker(archive.bytes);
  try {
    const transport = saveImageArchive({ archive: moved.archive, runtimeImage: 'konard/hive-mind-dind:latest', runtime: { ...runtime, RootFS: { Layers: [digest('moved base')] } }, image, run: moved.run, scratch: moved.directory });
    assert.equal(transport.mode, 'full');
    assert.match(transport.reason, /DeltaBaseNotPrefix/u);
    assert.deepEqual(gunzipSync(fs.readFileSync(moved.archive)), archive.bytes, 'the full archive is the exact saved image');
  } finally {
    moved.cleanup();
  }
  // A streamed delta that fails verification is replaced by the full archive.
  const broken = fakeDocker(imageArchive({ extraImage: true }).bytes);
  try {
    const transport = saveImageArchive({ archive: broken.archive, runtimeImage: 'konard/hive-mind-dind:latest', runtime, image, run: broken.run, scratch: broken.directory });
    assert.equal(transport.mode, 'full');
    assert.match(transport.reason, /delta rejected: .*exactly one image/u);
    assert.deepEqual(gunzipSync(fs.readFileSync(broken.archive)), imageArchive({ extraImage: true }).bytes);
  } finally {
    broken.cleanup();
  }
});

test('the consumer accepts only the producer-recorded base, by digest and image ID', () => {
  const archive = imageArchive();
  const identity = { runtimeImageId: digest('base config'), runtimeImageDigests: [baseRef] };
  const transport = { mode: 'delta', base: { ref: baseRef, imageId: identity.runtimeImageId, diffIds: archive.baseIds }, candidateLayers: 4 };
  assert.equal(IMAGE_BUNDLE_SCHEMA, 'formal-ai-hive-image-bundle/v2');
  assert.deepEqual(requireImageTransport(transport, identity), transport);
  assert.deepEqual(requireImageTransport({ mode: 'full', reason: 'x' }, identity).mode, 'full');
  for (const [label, mutated] of [
    ['an unrecorded digest', { ...transport, base: { ...transport.base, ref: 'konard/hive-mind-dind@sha256:' + 'c'.repeat(64) } }],
    ['a mutable tag', { ...transport, base: { ...transport.base, ref: 'konard/hive-mind-dind:latest' } }],
    ['another image ID', { ...transport, base: { ...transport.base, imageId: digest('cached image') } }],
    ['no base layers', { ...transport, base: { ...transport.base, diffIds: [] } }],
    ['a candidate adding nothing', { ...transport, candidateLayers: 2 }],
    ['an unknown mode', { ...transport, mode: 'cache' }],
    ['a full archive naming a base', { mode: 'full', base: transport.base }],
  ]) assert.throws(() => requireImageTransport(mutated, identity), undefined, label);
  const calls = [];
  const fake = pulled => ({ run: (command, args) => calls.push([command, ...args]), read: () => JSON.stringify([pulled]) });
  assert.equal(requirePinnedBase(transport.base, fake({ Id: transport.base.imageId, RootFS: { Layers: archive.baseIds } })).Id, transport.base.imageId);
  assert.deepEqual(calls[0], ['docker', 'pull', baseRef], 'the base is pulled by digest');
  assert.throws(() => requirePinnedBase(transport.base, fake({ Id: digest('substitute'), RootFS: { Layers: archive.baseIds } })), /pinned delta base image/u);
  assert.throws(() => requirePinnedBase(transport.base, fake({ Id: transport.base.imageId, RootFS: { Layers: archive.baseIds.slice(1) } })), /pinned delta base layers/u);
});
