// SPDX-License-Identifier: Unlicense
// Adapted from link-foundation/js-ai-driven-development-pipeline-template
// f8e30b7a00c3113eeec555edcfbaa9ef235ed273, scripts/check-secrets.mjs.
// Bound aggregate inflation and metadata; never extract archive paths to disk.
import {gunzipSync} from 'node:zlib';
export const MAX_DECOMPRESSED_BYTES = 128 * 1024 * 1024;
const isCompressed = (name) => /\.(gz|tgz)$/i.test(name);
const isTarball = (name) => /\.(tar\.gz|tgz)$/i.test(name);
const field = (block, start, length) =>
  block
    .subarray(start, start + length)
    .toString('utf8')
    .replace(/\0.*$/s, '');
function octal(value) {
  const text = value.replace(/\0.*$/s, '').trim();
  if (!/^[0-7]+$/.test(text)) {
    throw new Error('Invalid tar numeric field');
  }
  return Number.parseInt(text, 8);
}

function paxPath(content, type) {
  let name;
  for (let position = 0; position < content.length; ) {
    const space = content.indexOf(32, position);
    const text = content.subarray(position, space).toString();
    const length = Number(text);
    if (
      space < position ||
      !/^[0-9]+$/.test(text) ||
      !Number.isSafeInteger(length) ||
      length <= space - position + 1 ||
      position + length > content.length ||
      content[position + length - 1] !== 10
    ) {
      throw new Error('Malformed PAX record');
    }
    const record = content
      .subarray(space + 1, position + length - 1)
      .toString('utf8');
    // Refuse size/sparse overrides that change which bytes the reader scans.
    if (
      !/^(?:path|linkpath|mtime|atime|ctime|uid|gid|uname|gname|comment|charset)=/.test(
        record
      )
    ) {
      throw new Error('Unsupported PAX record');
    }
    if (record.startsWith('path=')) {
      if (type === 'g') {
        throw new Error('Unsupported global PAX path');
      }
      name = record.slice(5);
    }
    position += length;
  }
  return name;
}

function tarHeader(tar, offset, nextName) {
  if (offset + 1024 > tar.length) {
    throw new Error('Truncated tar archive');
  }
  const header = tar.subarray(offset, offset + 512);
  if (!header.every((byte) => byte === 0)) {
    return header;
  }
  if (
    nextName !== undefined ||
    !tar.subarray(offset).every((byte) => byte === 0)
  ) {
    throw new Error('Invalid tar terminator or orphan metadata');
  }
  return null;
}

// ustar and GNU/PAX metadata are scanned as well as regular file contents.
// Unsupported formats fail closed.
export function readTarEntries(tar) {
  const entries = [];
  let nextName;
  for (let offset = 0; ; ) {
    if (entries.length >= 10000) {
      throw new Error('Tar entry count limit exceeded');
    }
    const header = tarHeader(tar, offset, nextName);
    if (!header) {
      return entries;
    }
    const checksum = header.reduce(
      (sum, byte, index) => sum + (index >= 148 && index < 156 ? 32 : byte),
      0
    );
    if (checksum !== octal(field(header, 148, 8))) {
      throw new Error('Invalid tar checksum');
    }
    const size = octal(field(header, 124, 12));
    const end = offset + 512 + Math.ceil(size / 512) * 512;
    if (!Number.isSafeInteger(size) || end > tar.length) {
      throw new Error('Truncated tar entry');
    }
    const type = String.fromCharCode(header[156] || 48);
    if (!['0', '1', '2', '5', '7', 'x', 'g', 'L', 'K'].includes(type)) {
      throw new Error(`Unsupported tar entry type ${JSON.stringify(type)}`);
    }
    const prefix = field(header, 345, 155);
    const name =
      nextName ?? (prefix ? `${prefix}/` : '') + field(header, 0, 100);
    nextName = undefined;
    const content = tar.subarray(offset + 512, offset + 512 + size);
    if (type === 'L') {
      nextName = content.toString('utf8').replace(/\0.*$/s, '');
    } else if (type === 'x' || type === 'g') {
      nextName = paxPath(content, type);
    }
    // Metadata names cannot suppress scanning through secretlint extension filters.
    entries.push({
      name: ['0', '7'].includes(type) ? name : `${name}.metadata.txt`,
      content,
      metadata: header,
    });
    offset = end;
  }
}

export function scanCompressed(
  name,
  bytes,
  scan,
  budget = { remaining: MAX_DECOMPRESSED_BYTES },
  depth = 0
) {
  if (
    depth > 3 ||
    bytes.length > MAX_DECOMPRESSED_BYTES ||
    budget.remaining <= 0
  ) {
    throw new Error('Archive size or nesting limit exceeded');
  }
  const inflated = gunzipSync(bytes, { maxOutputLength: budget.remaining });
  budget.remaining -= inflated.length;
  if (isTarball(name)) {
    for (const entry of readTarEntries(inflated)) {
      const memberName = `${name}/${entry.name}`;
      scan(`${memberName}.metadata.txt`, entry.metadata);
      if (isCompressed(entry.name)) {
        scanCompressed(memberName, entry.content, scan, budget, depth + 1);
      } else {
        scan(memberName, entry.content);
      }
    }
  } else if (isCompressed(name.slice(0, -3))) {
    scanCompressed(name.slice(0, -3), inflated, scan, budget, depth + 1);
  } else {
    scan(name.slice(0, -3), inflated);
  }
}
