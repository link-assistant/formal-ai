// The owned status-ledger field contract comes from its maintained producer.
// Parsing uses the same Links Notation implementation as the server and browser.
import {parseLino} from '../../js/server/lino.mjs';
import {SHARED_FIELDS} from '../generate-requirement-status.mjs';

const fields = new Map([
  ['id', 'id'], ['shard', 'shard'], ['issue', 'issue'], ['verdict', 'verdict'],
  ...SHARED_FIELDS,
]);
const empty = () => Object.fromEntries([...fields.values()].map(name => [name, '']));

/**
 * Read every owned field, inheriting only fields in the canonical producer.
 * Unknown legacy metadata can be retained separately, never inherited or used.
 * @param {string} text
 * @param {{unknownFields?: 'refuse'|'preserve'}} options
 */
export function parseRequirementLedger(text, {unknownFields = 'refuse'} = {}) {
  if (!['refuse', 'preserve'].includes(unknownFields)) throw new Error('unknown ledger field policy');
  const tree = parseLino(text);
  if (tree.name !== 'requirement_status_ledger_shard' || tree.id !== '') {
    throw new Error('the status ledger needs exactly one owned shard root');
  }
  const ignoredFields = [];
  const defaults = empty();
  const records = [];
  const provenance = [];
  const defaultNodes = new Map();
  let sawRecord = false;
  const hasOwnedChild = node => node.children.some(child => fields.has(child.name)
    || child.name === 'requirement' || hasOwnedChild(child));
  const scalar = (node, target, seen, scope) => {
    const property = fields.get(node.name);
    if (!property) {
      if (unknownFields === 'refuse' || hasOwnedChild(node)) {
        throw new Error('unowned or foreign ledger field scope');
      }
      ignoredFields.push({scope, node});
      return;
    }
    if (node.children.length) throw new Error('owned ledger fields must be scalar leaves');
    if (seen.has(node.name)) throw new Error('duplicate ledger field');
    seen.add(node.name);
    target[property] = node.id;
  };
  const sharedSeen = new Set();
  for (const node of tree.children) {
    if (node.name !== 'requirement') {
      if (sawRecord && fields.has(node.name)) throw new Error('ledger defaults must precede records');
      if (node.name === 'id') throw new Error('requirement identity cannot be inherited');
      scalar(node, defaults, sharedSeen, 'shard');
      if (fields.has(node.name)) defaultNodes.set(node.name, node);
      continue;
    }
    sawRecord = true;
    if (node.id !== '') throw new Error('requirement fields need their own record scope');
    const record = {...defaults};
    const seen = new Set();
    const origin = new Map(defaultNodes);
    for (const child of node.children) {
      if (child.name === 'requirement') throw new Error('nested requirement scope');
      scalar(child, record, seen, records.length);
      if (fields.has(child.name)) origin.set(child.name, child);
    }
    if (record.id === '') throw new Error('requirement identity is missing');
    records.push(record);
    provenance.push([...origin].map(([field, source]) => ({field, source})));
  }
  return {records, provenance, ignoredFields, tree, sourceRaw: text};
}
