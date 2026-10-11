// Physical first-party append adapter. The lock serializes writers; unknown modes refuse.
import fs from 'node:fs';
import path from 'node:path';
import { TextDecoder } from 'node:util';

const CONTRACT = 'atomic-record-append/v1';
const MAXIMUM_BYTES = 65536;
export function atomicRecordAppend(root, args) {
  if (args.append_mode !== 'atomic_record_append' || typeof args.path !== 'string'
    || typeof args.append_request_id !== 'string' || typeof args.content !== 'string' || typeof args.record_id !== 'string'
    || args.content.split('\n')[1] !== args.record_id
    || args.content.split('\n')[0] !== 'general_change_plan'
    || [args.path, args.content, args.record_id, args.append_request_id].some(text => [...text].some(character => {
      const point = character.codePointAt(0);
      return point >= 0xd800 && point <= 0xdfff;
    }))
    || args.content.split('\n').filter(line => line === 'general_change_plan').length !== 1
    || !args.content.endsWith('\n') || args.content.split('\n').filter(line => line === args.record_id).length !== 1) {
    throw new Error('invalid explicit append contract');
  }
  if (args.path.includes('\\') || args.path.includes(':')
    || args.path.split('/').some(part => part === '' || part === '.' || part === '..')) {
    throw new Error('append path escape or nonportable component');
  }
  const physicalRoot = fs.realpathSync(root);
  const target = path.resolve(physicalRoot, args.path);
  if (target === physicalRoot || !target.startsWith(physicalRoot + path.sep)) throw new Error('append path escape');
  let ancestor = path.dirname(target);
  while (!fs.existsSync(ancestor)) ancestor = path.dirname(ancestor);
  const physicalAncestor = fs.realpathSync(ancestor);
  if (physicalAncestor !== physicalRoot && !physicalAncestor.startsWith(physicalRoot + path.sep)) throw new Error('append symlink escape');
  fs.mkdirSync(path.dirname(target), { recursive: true });
  const parent = fs.realpathSync(path.dirname(target));
  if (parent !== path.dirname(target)) throw new Error('append nonphysical parent');
  const lock = target + '.append-lock';
  fs.mkdirSync(lock);
  try {
    let before = '';
    try {
      const stat = fs.lstatSync(target);
      if (!stat.isFile() || stat.isSymbolicLink()) throw new Error('append target is not a regular file');
      const descriptor = fs.openSync(target, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
      try { before = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(fs.readFileSync(descriptor)); }
      finally { fs.closeSync(descriptor); }
    } catch (error) { if (error.code !== 'ENOENT') throw error; }
    if (Buffer.byteLength(before) > MAXIMUM_BYTES) throw new Error('append prior bytes exceed contract');
    const duplicate = before.split('\n').includes(args.record_id);
    let operation = 'appended';
    let after = before + (before !== '' && !before.endsWith('\n') ? '\n' : '') + args.content;
    if (duplicate) {
      const records = before.split(/(?=^general_change_plan\n)/mu);
      if (records.filter(record => record.split('\n').includes(args.record_id)).length !== 1
        || !records.some(record => record === args.content)) throw new Error('append identity collision');
      operation = 'already_present'; after = before;
    }
    if (Buffer.byteLength(after) > MAXIMUM_BYTES) throw new Error('append resulting bytes exceed contract');
    if (operation === 'appended') {
      const descriptor = fs.openSync(target, fs.constants.O_WRONLY | fs.constants.O_APPEND
        | fs.constants.O_CREAT | fs.constants.O_NOFOLLOW, 0o600);
      try {
        const expected = Buffer.from((before !== '' && !before.endsWith('\n') ? '\n' : '') + args.content);
        const written = fs.writeSync(descriptor, expected);
        if (written !== expected.length) throw new Error('append incomplete physical write');
        fs.fsyncSync(descriptor);
      } finally { fs.closeSync(descriptor); }
    }
    const afterBytes = fs.readFileSync(target);
    if (!afterBytes.equals(Buffer.from(after))) throw new Error('append readback mismatch');
    return { content: '', append_receipt: { schema: CONTRACT, path: args.path,
      record_id: args.record_id, append_request_id: args.append_request_id, content: args.content, before, after,
      before_bytes: Buffer.byteLength(before), after_bytes: afterBytes.length,
      operation, complete: true, success: true } };
  } finally {
    fs.rmdirSync(lock);
  }
}
