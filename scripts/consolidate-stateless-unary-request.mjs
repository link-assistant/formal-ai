#!/usr/bin/env node
// Installed Node capability. Supplied operation authorship remains zero.
import fs from 'node:fs';
import path from 'node:path';
import { createHash, randomBytes } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { planUnaryConsolidation } from './consolidate-stateless-unary.mjs';
import { sourceUnaryConsolidationRequest } from '../js/agentic/source_unary_consolidation.mjs';
const hash = value => createHash('sha256').update(value).digest('hex');
const contracts = Object.freeze({
  lower: {
    module: 'js/agentic/crate/rust_str.mjs',
    exportName: 'lower'
  },
  trim: {
    module: 'js/agentic/callback_values.mjs',
    exportName: 'callbackTrim'
  },
  text: {
    module: 'js/agentic/callback_values.mjs',
    exportName: 'callbackText'
  },
  value: {
    module: 'js/agentic/callback_values.mjs',
    exportName: 'callbackValue'
  },
  nonempty: {
    module: 'js/agentic/callback_values.mjs',
    exportName: 'callbackNonempty'
  },
  positiveLength: {
    module: 'js/agentic/callback_values.mjs',
    exportName: 'callbackHasLength'
  }
});
/** The existing algorithm validates helper source bodies and all preimages before effects. */
export function executeUnaryConsolidationRequest({
  root = process.cwd(),
  request,
  expected,
  writeChunk = fs.writeSync
}) {
  root = fs.realpathSync(root);
  const definition = sourceUnaryConsolidationRequest(request);
  if (!definition ||
         !/^([a-f0-9]{64})$/u.test(expected ?? '')) throw Error('QualifiedObservedUnaryRequestRequired');
  const file = path.join(root, definition.owner);
  let physical = root;
  for (const part of definition.owner.split('/')) {
    physical = path.join(physical, part);
    if (fs.lstatSync(physical).isSymbolicLink()) throw Error('UnarySourceSymlinkRefused');
  }
  const stat = fs.lstatSync(file);
  if (!stat.isFile() ||
         stat.nlink !== 1 ||
         stat.size > 1048576) throw Error('UnarySourceBoundsRefused');
  const before = fs.readFileSync(file);
  if (hash(before) !== expected) throw Error('UnarySourcePreimageChanged');
  if (/(?:@generated|DO NOT EDIT|Auto-generated)/u.test(before.toString('utf8').slice(0, 4096))) throw Error('UnaryGeneratedOwnerRequiresCanonicalGenerator');
  const plan = planUnaryConsolidation(root, [definition.owner], contracts);
  if (plan.deferred.length) throw Error('UnaryCallbacksRequireUnresolvedOwnership');
  const change = plan.changes[0],
    expectedAfter = change?.afterSha256 ?? expected;
  if (change) {
    const descriptor = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW),
      staged = file + '.formal-ai-unary-' + randomBytes(12).toString('hex');
    let output = null,
      renamed = false;
    const unchanged = () => {
      const current = fs.fstatSync(descriptor),
        physical = fs.lstatSync(file);
      if (current.dev !== stat.dev ||
         current.ino !== stat.ino ||
         physical.dev !== current.dev ||
         physical.ino !== current.ino ||
         !physical.isFile() ||
         physical.nlink !== 1 ||
         hash(fs.readFileSync(file)) !== expected) throw Error('UnaryPhysicalSourceDrift');
      for (const helper of Object.values(plan.helpers)) if (hash(fs.readFileSync(path.join(root, helper.module))) !== helper.sha256) throw Error('UnaryPhysicalHelperDrift');
    };
    try {
      unchanged();
      output = fs.openSync(staged, fs.constants.O_WRONLY | fs.constants.O_CREAT | fs.constants.O_EXCL | fs.constants.O_NOFOLLOW, stat.mode & 0o777);
      const bytes = Buffer.from(change.after);
      let offset = 0;
      while (offset < bytes.length) {
        const remaining = bytes.length - offset,
          written = writeChunk(output, bytes, offset, remaining, offset);
        if (!Number.isSafeInteger(written) ||
         written <= 0 ||
         written > remaining) throw Error('UnaryStagedWriteProgress');
        offset += written;
      }
      fs.fsyncSync(output);
      fs.closeSync(output);
      output = null;
      if (!fs.readFileSync(staged).equals(bytes)) throw Error('UnaryStagedWriteMismatch');
      unchanged();
      fs.renameSync(staged, file);
      renamed = true;
    } finally {
      if (output !== null) fs.closeSync(output);
      fs.closeSync(descriptor);
      if (!renamed &&
         fs.existsSync(staged)) fs.unlinkSync(staged);
    }
  }
  const after = fs.readFileSync(file);
  if (hash(after) !== expectedAfter) throw Error('UnaryPhysicalWriteMismatch');
  const applied = {
    files: plan.changes.length,
    callbacks: plan.changes.reduce((sum, change) => sum + change.edits.length, 0),
    requiresIndependentConsumerChecks: true
  };
  return {
    schema: 'stateless-unary-request/v1',
    request,
    owner: definition.owner,
    beforeSha256: expected,
    afterSha256: hash(after),
    ...applied,
    completeSemanticEquivalence: false,
    independentBehaviorValidation: 'pending',
    autonomousAuthorshipCredit: 0
  };
}
if (process.argv[1] &&
         path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 6 ||
         process.argv[2] !== '--request' ||
         process.argv[4] !== '--expected') throw Error('usage: node scripts/consolidate-stateless-unary-request.mjs --request NATURAL_REQUEST --expected SOURCE_SHA256');
    console.log(JSON.stringify(executeUnaryConsolidationRequest({
      request: process.argv[3],
      expected: process.argv[5]
    })));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
