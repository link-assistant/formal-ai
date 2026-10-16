// Integrity of retained driver evidence, not PR attribution or computational replay.
import fs from 'node:fs';
import path from 'node:path';
import { provenancePrimitives } from './session.mjs';
import { blake3Hex } from '../crypto/blake3.mjs';
import { encodeTyped, decodeTyped } from '../server/structured-log.mjs';
const p = provenancePrimitives({
  encodeTyped,
  decodeTyped
});
function regularBytes(directory, relative) {
  if (typeof relative !== 'string' || !relative || relative.includes('\\') || relative.split('/').some(x => !x || x === '.' || x === '..') || path.isAbsolute(relative)) throw Error('unsafe_evidence_path');
  let target = fs.realpathSync(directory);
  for (const part of relative.split('/')) {
    target = path.join(target, part);
    if (fs.lstatSync(target).isSymbolicLink()) throw Error('evidence_symlink');
  }
  if (!fs.statSync(target).isFile()) throw Error('evidence_not_file');
  return fs.readFileSync(target);
}
export function verifyDriveEvidence(directory, workspace, expectedAnchor) {
  const anchor = expectedAnchor ?? decodeTyped(regularBytes(directory, 'anchor.lino').toString('utf8').trim());
  const chain = regularBytes(directory, 'chain.lino').toString('utf8');
  if (!chain.endsWith('\n')) throw Error('incomplete_chain_line');
  const lines = chain.slice(0, -1).split('\n');
  p.verifyChain(lines, anchor);
  const records = lines.map(decodeTyped),
    outputs = new Set(),
    thinking = new Set(),
    latestEdits = new Map();
  let missingThinkingFields = 0;
  for (const record of records) {
    const evidence = record.payload?.evidence;
    if (evidence) {
      if (!/^\d{6}\.lino$/u.test(evidence.file) || !Number.isSafeInteger(evidence.size) || evidence.size < 0) throw Error('invalid_evidence_binding');
      const text = regularBytes(directory, evidence.file).toString('utf8');
      if (!text.endsWith('\n')) throw Error('incomplete_evidence_line');
      const encoded = text.slice(0, -1),
        value = decodeTyped(encoded);
      if (encodeTyped(value) !== encoded || value.type !== 'bytes-v1' || value.encoding !== 'base64' || typeof value.value !== 'string') throw Error('invalid_evidence_bytes');
      const bytes = Buffer.from(value.value, 'base64');
      if (bytes.toString('base64') !== value.value || bytes.length !== evidence.size || blake3Hex(bytes) !== evidence.blake3) throw Error('evidence_hash');
    }
    if (record.kind === 'message_out') outputs.add(record.hash);
    if (record.kind === 'thinking') {
      if (!outputs.has(record.payload.messageOut) || thinking.has(record.payload.messageOut)) throw Error('thinking_binding');
      thinking.add(record.payload.messageOut);
      if (record.payload.status === 'unavailable') missingThinkingFields++;
    }
    if (record.kind === 'edit') {
      if (!outputs.has(record.payload.messageOut)) throw Error('edit_message_binding');
      latestEdits.set(record.payload.path, record.payload);
    }
  }
  if (thinking.size !== outputs.size) throw Error('thinking_missing');
  const files = [...latestEdits].sort((a, b) => Buffer.compare(Buffer.from(a[0]), Buffer.from(b[0]))).map(([file, edit]) => {
    try {
      const bytes = regularBytes(workspace, file);
      return {
        path: file,
        status: blake3Hex(bytes) === edit.postimage.blake3 ? 'matches-observation' : 'foreign'
      };
    } catch {
      return {
        path: file,
        status: 'foreign'
      };
    }
  });
  return {
    schema: 'formal-ai-driver-evidence-report-v1',
    integrity: true,
    anchorTrust: expectedAnchor ? 'caller-pinned' : 'self-supplied',
    files,
    missingThinkingFields,
    scope: 'driver-dialogue-and-observed-write-edit-bytes',
    authorship: 'not-established',
    replay: 'not-performed',
    prDiffCoverage: 'not-checked'
  };
}
