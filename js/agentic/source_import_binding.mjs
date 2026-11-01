// Source-derived named-import addition; full readable CST and concrete residual.
import grammar from '../../data/meta/source-import-request.json' with { type: 'json' };
import { sourceToLinks, sourceFromLinks, ownedSourceLinks, verifySourceLinks } from '../meta/readable-source.js';
import { observedSourceView, sourceCstViews } from './observed_source_usage.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
const {
  one,
  form,
  value,
  concrete
} = sourceCstViews;
const pattern = new RegExp(grammar.expression.replaceAll('(?P<', '(?<'), 'iu');
export function sourceImportContract(task) {
  if (typeof task !== 'string' || new TextEncoder().encode(task).length > 8192) return null;
  const groups = pattern.exec(task)?.groups;
  if (!groups) return null;
  if (groups.owner.split('/').some(part => !part || part === '.' || part === '..') || groups.owner.length > 4096) return null;
  return {
    operation: 'add-named-import',
    authority: 'owned',
    ...groups
  };
}
function inspect(source, document, module, binding) {
  const view = observedSourceView(source, document),
    records = view.records;
  const imports = records.filter(record => ['import_declaration', 'import_statement'].includes(form(record.node))).filter(record => {
    const node = one(record.node, ['source']);
    if (!node) return false;
    const raw = concrete(node);
    return ["'", '"'].includes(raw[0]) && raw.at(-1) === raw[0] && raw.slice(1, -1) === module;
  });
  if (imports.length !== 1) throw Error('NamedImportDeclarationAmbiguous');
  const selected = imports[0],
    inside = records.filter(record => record.ancestors.includes(selected.node));
  if (inside.some(record => ['import_namespace_specifier', 'namespace_import'].includes(form(record.node)))) throw Error('NamedImportNamespaceUnsupported');
  const specifiers = inside.filter(record => form(record.node) === 'import_specifier');
  if (!specifiers.length) throw Error('NamedImportListRequired');
  const existing = specifiers.some(record => {
    const imported = one(record.node, ['imported', 'name']),
      local = one(record.node, ['local', 'alias']) ?? imported;
    return imported && local && concrete(imported) === binding && concrete(local) === binding;
  });
  const sourceNode = one(selected.node, ['source']),
    sourceRecord = view.byNode.get(sourceNode),
    last = Math.max(...specifiers.map(record => record.end));
  const closing = inside.filter(record => value(record.node, 'text') !== null && concrete(record.node) === '}' && record.start >= last && record.end <= sourceRecord.start);
  if (closing.length !== 1) throw Error('NamedImportDelimiterAmbiguous');
  const trailingComma = inside.some(record => value(record.node, 'text') !== null && concrete(record.node) === ',' && record.start >= last && record.end <= closing[0].start);
  return {
    existing,
    position: closing[0].start,
    trailingComma,
    specifiers: specifiers.length
  };
}
export function deriveNamedImportEdit({
  owner,
  source,
  module,
  binding,
  frontend
}) {
  if (typeof source !== 'string' || new TextEncoder().encode(source).length > 262144 || typeof owner !== 'string' || !['.js', '.mjs'].some(suffix => owner.endsWith(suffix)) || typeof module !== 'string' || !module.length || typeof binding !== 'string' || !/^[A-Za-z_$][A-Za-z0-9_$]{0,127}$/u.test(binding)) throw Error('NamedImportInputRequired');
  const document = ownedSourceLinks(owner, sourceToLinks(source, 'JavaScript', frontend));
  verifySourceLinks(document, frontend);
  if (sourceFromLinks(document) !== source) throw Error('NamedImportPreimageReconstructionMismatch');
  const selected = inspect(source, document, module, binding),
    bytes = new TextEncoder().encode(source),
    decoder = new TextDecoder('utf-8', {
      fatal: true,
      ignoreBOM: true
    });
  const insertion = selected.existing ? '' : (selected.trailingComma ? ' ' : ', ') + binding + (selected.trailingComma ? ',' : '');
  const prefix = decoder.decode(bytes.subarray(0, selected.position)),
    suffix = decoder.decode(bytes.subarray(selected.position));
  const candidate = prefix + insertion + suffix;
  if (new TextEncoder().encode(candidate).length > 262144) throw Error('NamedImportCandidateBudget');
  const changed = ownedSourceLinks(owner, sourceToLinks(candidate, 'JavaScript', frontend));
  verifySourceLinks(changed, frontend);
  if (sourceFromLinks(changed) !== candidate || !inspect(candidate, changed, module, binding).existing) throw Error('NamedImportCandidateBindingMismatch');
  const restored = candidate.slice(0, prefix.length) + candidate.slice(prefix.length + insertion.length);
  if (restored !== source || JSON.stringify(ownedSourceLinks(owner, sourceToLinks(restored, 'JavaScript', frontend))) !== JSON.stringify(document)) throw Error('NamedImportInverseMismatch');
  return {
    schema: 'observed-source-edit/v1',
    owner,
    candidate: {
      path: owner,
      source: candidate,
      beforeSha256: sha256Hex(source),
      afterSha256: sha256Hex(candidate),
      changed: !selected.existing
    },
    binding: {
      module,
      name: binding,
      insertionSpan: [selected.position, selected.position + new TextEncoder().encode(insertion).length]
    },
    verification: {
      completeConcreteResidual: true,
      independentGrammarReparse: true,
      exactInverse: true,
      commentsAndWhitespacePreserved: true,
      arbitrarySemanticEquivalence: false,
      exportResolution: 'not-checked'
    },
    knowledge: {
      schema: grammar.schema,
      scope: grammar.scope
    },
    independentAcceptance: false
  };
}
