#!/usr/bin/env node
// Issue #1486: transfer only the layers a same-source Hive task image adds on
// top of its immutable, digest-pinned runtime base.
//
// `docker save` writes a tar whose layer blobs are named `blobs/sha256/<hex>`
// (the uncompressed diffID with the classic store; the registry blob digest
// with the containerd store) plus `manifest.json`. A delta archive omits the
// base blobs. The consumer first pulls the base by digest and verifies its
// image ID; `docker load` then reuses the present layers, and the loaded image
// ID must still equal the producer's. Missing base layers fail the load closed;
// nothing ever substitutes an arbitrary cached image for the exact candidate.
//
// The filter streams: it never buffers a layer, so a multi-gigabyte image
// passes through a bounded Node process.
//
//   docker save IMAGE | node scripts/lib/oci-delta-archive.mjs filter --plan PLAN.json --report REPORT.json | gzip -1 > delta.tar.gz
import { readFileSync, writeFileSync } from 'node:fs';
import { Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

export const DELTA_PLAN_SCHEMA = 'formal-ai-oci-delta-plan/v1';
const BLOCK = 512;
const DIGEST = /^sha256:[0-9a-f]{64}$/u;
const SMALL_JSON = new Set(['manifest.json', 'index.json']);

const blobName = digest => 'blobs/sha256/' + digest.slice('sha256:'.length);

/**
 * Which blobs a delta may omit: the base layers (by diffID and, when known, by
 * registry blob digest) that the candidate inherits as an exact prefix and
 * does not repeat in its own layers.
 */
export function deltaPlan({ baseImageId, baseRef, baseDiffIds, candidateDiffIds, baseBlobDigests = [] }) {
  if (!DIGEST.test(baseImageId ?? '')) throw Error('DeltaBaseUnknown: ' + baseImageId);
  if (!/@sha256:[0-9a-f]{64}$/u.test(baseRef ?? '')) throw Error('DeltaBaseNotPinned: ' + baseRef);
  for (const digest of [...baseDiffIds, ...candidateDiffIds, ...baseBlobDigests]) if (!DIGEST.test(digest)) throw Error('DeltaDigest: ' + digest);
  if (!baseDiffIds.length || baseDiffIds.length >= candidateDiffIds.length || baseDiffIds.some((digest, index) => candidateDiffIds[index] !== digest)) {
    throw Error('DeltaBaseNotPrefix: the candidate does not inherit the pinned base layers');
  }
  const added = new Set(candidateDiffIds.slice(baseDiffIds.length));
  const drop = [...new Set([...baseDiffIds, ...baseBlobDigests])].filter(digest => !added.has(digest)).map(blobName).sort();
  return { schema: DELTA_PLAN_SCHEMA, baseImageId, baseRef, baseLayers: baseDiffIds.length, candidateLayers: candidateDiffIds.length, drop };
}

const field = (header, offset, length) => {
  const end = header.indexOf(0, offset);
  return header.toString('utf8', offset, end >= offset && end < offset + length ? end : offset + length);
};
const octal = (header, offset, length) => {
  if (header[offset] & 0x80) {
    let value = 0;
    for (let index = offset + 1; index < offset + length; index += 1) value = value * 256 + header[index];
    return value;
  }
  const text = field(header, offset, length).trim();
  return text ? parseInt(text, 8) : 0;
};
const padded = size => Math.ceil(size / BLOCK) * BLOCK;

/** Parse a pax extended header body into its records. */
export function paxRecords(body) {
  const records = {};
  let offset = 0;
  while (offset < body.length) {
    const space = body.indexOf(0x20, offset);
    if (space < 0) break;
    const length = Number(body.toString('utf8', offset, space));
    if (!(length > 0)) break;
    const record = body.toString('utf8', space + 1, offset + length - 1);
    const equals = record.indexOf('=');
    records[record.slice(0, equals)] = record.slice(equals + 1);
    offset += length;
  }
  return records;
}

/**
 * A streaming tar filter that drops the planned blobs together with any pax
 * or GNU long-name header that describes them, and records every member.
 * Per-file metadata members (`x`, `L`, `K`) are held until the member they
 * describe decides whether they are kept; global pax headers are always kept.
 */
export class TarDeltaFilter extends Transform {
  constructor(drop) {
    super();
    this.drop = new Set(drop);
    this.pending = Buffer.alloc(0);
    this.state = 'header';
    this.remaining = 0;
    this.held = [];
    this.longName = null;
    this.member = null;
    this.report = { kept: [], dropped: [], json: {}, keptBytes: 0, droppedBytes: 0, ended: false };
  }

  _transform(chunk, _encoding, done) {
    this.pending = this.pending.length ? Buffer.concat([this.pending, chunk]) : chunk;
    try {
      this.consume();
      done();
    } catch (error) {
      done(error);
    }
  }

  _flush(done) {
    if (this.state !== 'end' || this.pending.some(byte => byte !== 0)) return done(Error('DeltaTarTruncated: the image archive ended inside a member'));
    this.write_(this.pending, true);
    this.report.ended = true;
    done();
  }

  write_(buffer, keep) {
    if (!buffer.length) return;
    if (keep) {
      this.push(buffer);
      this.report.keptBytes += buffer.length;
    } else this.report.droppedBytes += buffer.length;
  }

  consume() {
    while (this.pending.length && this.state !== 'end') {
      if (this.state === 'body') {
        const take = Math.min(this.remaining, this.pending.length);
        const part = this.pending.subarray(0, take);
        this.pending = this.pending.subarray(take);
        this.remaining -= take;
        if (this.member.capture) this.member.capture.push(Buffer.from(part));
        if (this.member.meta) this.held.push(Buffer.from(part));
        else this.write_(part, this.member.keep);
        if (this.remaining === 0) this.finishMember();
        continue;
      }
      if (this.pending.length < BLOCK) return;
      const header = Buffer.from(this.pending.subarray(0, BLOCK));
      this.pending = this.pending.subarray(BLOCK);
      if (header.every(byte => byte === 0)) {
        if (this.held.length) throw Error('DeltaTarTruncated: metadata header without a member');
        this.write_(header, true);
        this.state = 'end';
        continue;
      }
      let checksum = 0;
      for (let index = 0; index < BLOCK; index += 1) checksum += index >= 148 && index < 156 ? 0x20 : header[index];
      if (checksum !== octal(header, 148, 8)) throw Error('DeltaTarChecksum: not a tar header');
      const type = String.fromCharCode(header[156] || 0x30);
      const size = octal(header, 124, 12);
      if (type === 'x' || type === 'L' || type === 'K') {
        this.held.push(header);
        this.member = { type, size, meta: true, capture: [] };
      } else if (type === 'g') {
        this.write_(header, true);
        this.member = { name: null, size, keep: true, capture: null };
      } else {
        const ustarPrefix = field(header, 345, 155);
        const name = (this.longName ?? (ustarPrefix ? ustarPrefix + '/' : '') + field(header, 0, 100)).replace(/^\.\//u, '');
        this.longName = null;
        const keep = !this.drop.has(name);
        for (const block of this.held) this.write_(block, keep);
        this.held = [];
        this.write_(header, keep);
        this.member = { name, size, keep, capture: SMALL_JSON.has(name) && size < 4 * 1024 * 1024 ? [] : null };
        (keep ? this.report.kept : this.report.dropped).push({ name, bytes: size });
      }
      this.remaining = padded(size);
      this.state = 'body';
      if (!this.remaining) this.finishMember();
    }
  }

  finishMember() {
    const member = this.member;
    this.state = 'header';
    const body = member.capture ? Buffer.concat(member.capture).subarray(0, member.size) : null;
    if (member.meta) {
      if (member.type === 'L') this.longName = body.toString('utf8').replace(/\0+$/u, '');
      if (member.type === 'x' && paxRecords(body).path) this.longName = paxRecords(body).path;
    } else if (body) this.report.json[member.name] = body.toString('utf8');
  }
}

/**
 * The delta is complete only if every dropped blob is one of the inherited
 * base layers, every added layer and the config are present, and the archive
 * names exactly one image.
 */
export function verifyDelta(plan, report) {
  const problems = [];
  if (!report.ended) problems.push('archive did not end');
  let manifest;
  try {
    manifest = JSON.parse(report.json['manifest.json'] ?? 'null');
  } catch {
    manifest = null;
  }
  if (!Array.isArray(manifest) || manifest.length !== 1) return [...problems, 'manifest.json must name exactly one image'];
  const [image] = manifest;
  const layers = image.Layers ?? [];
  const kept = new Set(report.kept.map(member => member.name));
  const dropped = new Set(report.dropped.map(member => member.name));
  if (layers.length !== plan.candidateLayers) problems.push('manifest has ' + layers.length + ' layers, the candidate ' + plan.candidateLayers);
  if (!kept.has(image.Config)) problems.push('config ' + image.Config + ' missing');
  const inherited = new Set(layers.slice(0, plan.baseLayers));
  for (const name of dropped) if (!inherited.has(name)) problems.push('dropped ' + name + ' is not an inherited base layer');
  for (const name of layers.slice(plan.baseLayers)) if (!kept.has(name)) problems.push('added layer ' + name + ' missing');
  for (const name of layers.slice(0, plan.baseLayers)) if (!kept.has(name) && !dropped.has(name)) problems.push('base layer ' + name + ' absent from the archive');
  return problems;
}

function option(argv, name) {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : undefined;
}

export async function filterMain(argv = process.argv.slice(2), input = process.stdin, output = process.stdout) {
  const plan = JSON.parse(readFileSync(option(argv, '--plan'), 'utf8'));
  if (plan.schema !== DELTA_PLAN_SCHEMA) throw Error('DeltaPlanSchema');
  const filter = new TarDeltaFilter(plan.drop);
  await pipeline(input, filter, output);
  const problems = verifyDelta(plan, filter.report);
  const report = {
    plan, problems,
    keptBytes: filter.report.keptBytes, droppedBytes: filter.report.droppedBytes,
    keptMembers: filter.report.kept.length, droppedLayers: filter.report.dropped.map(member => member.name),
  };
  writeFileSync(option(argv, '--report'), JSON.stringify(report, null, 2) + '\n');
  if (problems.length) throw Error('DeltaIncomplete: ' + problems.join('; '));
}

if (process.argv[1] === fileURLToPath(import.meta.url) && process.argv[2] === 'filter') {
  filterMain(process.argv.slice(3)).catch(error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
