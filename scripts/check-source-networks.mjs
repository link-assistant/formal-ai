#!/usr/bin/env node
// R1188-U3: every owned production source survives full upstream network serialization.
// Distribution packets contain full .lino.gz documents; receipts are identities, never substitutes for them.
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { gzipSync } from 'node:zlib';
import { upstreamDirectory, workflowPin } from './translate-js-rust.mjs';
import { partitionSourcePaths, serializeSourceNetwork, sourceDigest, verifySourcePacket } from './lib/source-network-packets.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const languageFor = (path) => path.endsWith('.rs') ? 'rust'
  : /\.(?:ts|mts|tsx)$/u.test(path) ? 'TypeScript' : 'JavaScript';

/** Authored or generated production sources, excluding vendored distributions and test harnesses. */
export function ownedProductionSources(root = ROOT) {
  return execFileSync('git', ['ls-files', '-z', '--', 'rust/src', 'js', 'ts'], { cwd: root, encoding: 'utf8' })
    .split('\0').filter((path) => /\.(?:rs|js|mjs|jsx|ts|mts|tsx)$/u.test(path)
      && !/^js\/(?:vendor|seed)\//u.test(path) && !/\.bundle\.js$/u.test(path));
}

export async function main(argumentsList) {
  const value = (name, fallback) => argumentsList.includes(name)
    ? argumentsList[argumentsList.indexOf(name) + 1] : fallback;
  const shardIndex = Number(value('--shard-index', '0')), shardCount = Number(value('--shard-count', '1'));
  const commit = workflowPin(readFileSync(join(ROOT, '.github/workflows/layered-ci.yml'), 'utf8'));
  const upstream = upstreamDirectory(argumentsList, commit);
  const { LinkNetwork } = await import(pathToFileURL(join(upstream, 'js/src/index.js')).href);
  const baseHead = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: ROOT, encoding: 'utf8' }).trim();
  const workingTreeChanges = execFileSync('git', ['diff', '--name-only', 'HEAD', '--', 'rust/src', 'js', 'ts'],
    { cwd: ROOT, encoding: 'utf8' }).trim();
  const paths = ownedProductionSources();
  const selected = partitionSourcePaths(paths, shardIndex, shardCount);
  if (selected.length === 0) throw new Error('source shard contains no owned modules');
  const output = value('--output', '');
  if (output && !argumentsList.includes('--write')) throw new Error('--output requires --write');
  const decoder = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });
  const receipts = [];
  for (const path of selected) {
    const sourceBytes = readFileSync(join(ROOT, path)), source = decoder.decode(sourceBytes);
    if (!Buffer.from(source, 'utf8').equals(sourceBytes)) throw new Error('source is not losslessly decoded: ' + path);
    const { document, receipt } = serializeSourceNetwork({ path, source, language: languageFor(path) }, LinkNetwork);
    if (!readFileSync(join(ROOT, path)).equals(sourceBytes)) throw new Error('source changed during serialization: ' + path);
    const compressed = gzipSync(Buffer.from(document, 'utf8'));
    const packet = path + '.lino.gz';
    const packetReceipt = { ...receipt, packet, compressedBytes: compressed.length, compressedSha256: sourceDigest(compressed) };
    if (verifySourcePacket(packetReceipt, compressed, LinkNetwork) !== source) throw new Error('packet reconstruction differs');
    receipts.push(packetReceipt);
    if (output) {
      const destination = join(resolve(output), packet);
      mkdirSync(dirname(destination), { recursive: true });
      writeFileSync(destination, compressed);
    }
  }
  for (const receipt of receipts) {
    if (sourceDigest(readFileSync(join(ROOT, receipt.path))) !== receipt.sourceSha256) {
      throw new Error('source changed after serialization: ' + receipt.path);
    }
  }
  const observedHead = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: ROOT, encoding: 'utf8' }).trim();
  if (observedHead !== baseHead) throw new Error('repository HEAD changed during serialization');
  const finalChanges = execFileSync('git', ['diff', '--name-only', 'HEAD', '--', 'rust/src', 'js', 'ts'],
    { cwd: ROOT, encoding: 'utf8' }).trim();
  const sourceHead = workingTreeChanges || finalChanges ? null : baseHead;
  const report = { sourceHead, baseHead, inputState: sourceHead ? 'committed-head' : 'working-tree', upstreamCommit: commit, shardIndex, shardCount, totalOwnedSources: paths.length,
    checkedSources: receipts.length, fidelity: 'lossless-network-serialization', sources: receipts };
  if (output) {
    mkdirSync(resolve(output), { recursive: true });
    writeFileSync(join(resolve(output), 'source-network-receipts.json'), JSON.stringify(report, null, 2) + '\n');
  }
  console.log(JSON.stringify({ sourceHead, upstreamCommit: commit, shardIndex, shardCount,
    totalOwnedSources: paths.length, checkedSources: receipts.length, fidelity: report.fidelity }));
  return report;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).catch((error) => { console.error(error.message); process.exitCode = 1; });
}
