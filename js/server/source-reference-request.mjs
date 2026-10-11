import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { parseLino } from './lino.mjs';
import { compileCompoundTask } from '../agentic/compound_task/clause_contract.mjs';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function physicalSource(file) {
  if (fs.realpathSync(file) !== file) throw Error('ReferenceRequestSourceAlias');
  const before = fs.lstatSync(file, {bigint: true});
  if (!before.isFile()) throw Error('ReferenceRequestSourceNotRegular');
  const bytes = fs.readFileSync(file);
  const after = fs.lstatSync(file, {bigint: true});
  const identity = value => [value.dev, value.ino, value.ctimeNs, value.size].join(':');
  if (identity(before) !== identity(after) || before.size !== BigInt(bytes.length)) {
    throw Error('ReferenceRequestSourceChangedDuringRead');
  }
  return Object.freeze({file, identity: identity(after), sha256: hash(bytes),
    text: new TextDecoder('utf-8', {fatal: true, ignoreBOM: true}).decode(bytes)});
}

/** Data projection only; the real factory owns context and physical Read issuance. */
export function deriveReferenceReadRequest(request, seed, factoryURL) {
  const contract = compileCompoundTask(request, parseLino(seed));
  if (contract === null) return null;
  const seedFile = fileURLToPath(new URL('../../data/seed/source-authoring-grammar.lino', factoryURL));
  const sources = [physicalSource(seedFile), physicalSource(fileURLToPath(import.meta.url)),
    physicalSource(fileURLToPath(new URL('../agentic/compound_task/clause_contract.mjs', import.meta.url)))];
  if (sources[0].text !== seed) throw Error('DifferentPhysicalReferenceRequestSeed');
  const rename = contract.clauses.find(clause => clause.kind === 'private-constant-rename');
  if (!rename || contract.wholeRequestConsumed !== true || contract.originalRequest !== request) {
    throw Error('IncompleteReferenceReadRequest');
  }
  const validate = () => {
    for (const source of sources) {
      const current = physicalSource(source.file);
      if (current.identity !== source.identity || current.sha256 !== source.sha256) {
        throw Error('ReferenceRequestSourceDrift');
      }
    }
  };
  return Object.freeze({contract, operands: Object.freeze([rename.captures.sourcePath.text]), validate});
}
