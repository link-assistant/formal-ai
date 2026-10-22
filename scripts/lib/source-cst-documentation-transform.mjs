// Complete readable CST transformations of proven JSDoc parameter bindings.
import { parseLinks, writeLink, textOf } from '../../js/logging/lino-codec.mjs';
import { parameterDocumentationGrammar, formalizeParameterDocumentation, naturalizeParameterDocumentation } from '../../js/meta/comment-meaning.js';
import { sourceOwner, sourceFromLinks, sourceToLinks, ownedSourceLinks, verifySourceLinks } from '../../js/meta/readable-source.js';
import { createSourceCstTransformRule, readSourceCstTransformRule, writeSourceCstTransformRule, applySourceCstTransform } from './source-cst-transform.mjs';
const quoted = value => ({
  quoted: value
});
const equal = (left, right) => JSON.stringify(left) === JSON.stringify(right);
const excluded = new Set(['form', 'role', 'text', 'language', 'path', 'meaning', 'semantic-gap']);
const children = node => node.filter(item => Array.isArray(item) &&
         !excluded.has(item[0]));
const optional = (node, name) => node.find(item => Array.isArray(item) &&
         item[0] === name) ?? null;
const value = (node, name) => {
  const found = optional(node, name);
  return found ? textOf(found[1]) : null;
};
const form = node => value(node, 'form');
const concrete = node => value(node, 'text') ?? children(node).map(concrete).join('');
function records(node, ancestors = [], path = []) {
  return [{
    node,
    ancestors,
    path,
    parent: ancestors.at(-1) ?? null
  }, ...node.flatMap((item, index) => Array.isArray(item) &&
         !excluded.has(item[0]) ? records(item, [...ancestors, node], [...path, index]) : [])];
}
const atPath = (node, path) => path.reduce((current, index) => current[index], node);
export const DOCUMENTED_PARAMETER_CONTRACT = `(documentation-parameter-transform
 (version "attached-jsdoc-simple-param/v1")
 (match "complete-qualified-function-preimage" "unique-immediately-leading-jsdoc")
 (capture "actual-simple-parameter" "line-tag-name-span" "unchanged-description-and-type")
 (preconditions "owner-bound" "unique-param-tag" "no-owner-override" "grammar-derived-tag" "no-optional-or-property-name")
 (target "replace-only-proven-tag-name-span")
 (preserve "full-source-residual" "description" "type-spelling" "comment-delimiters" "layout")
 (refuse "ambiguous-attachment" "duplicate-tag" "unknown-parameter" "unsupported-name-syntax" "unbalanced-type")
 (inverse "swap-binding-and-target" "full-result-preimage")
 (claim "parameter-binding-only") (not-certified "arbitrary-prose" "compiler-validity" "Rust-Markdown-parameter-meaning"))`;
const contract = parseLinks(DOCUMENTED_PARAMETER_CONTRACT, {
  quotedText: 'json'
})[0];
function attachedDocumentation(document, definition, inspectOnly = false) {
  const all = records(document),
    functions = all.filter(record => form(record.node) === 'function_declaration' &&
         children(record.node).some(child => value(child, 'role') === 'name' &&
         concrete(child) === definition.declaration));
  if (functions.length !== 1) throw Error('UniqueDocumentationFunctionRequired');
  let selected = functions[0];
  while (selected.parent &&
         ['export_named_declaration', 'export_default_declaration'].includes(form(selected.parent))) {
    selected = all.find(record => record.node === selected.parent);
  }
  if (!selected.parent) throw Error('DocumentationAttachmentUnavailable');
  const siblings = children(selected.parent),
    position = siblings.indexOf(selected.node);
  let previous = position - 1,
    spacing = '';
  while (previous >= 0 &&
         form(siblings[previous]) === 'whitespace') {
    spacing = concrete(siblings[previous]) + spacing;
    previous -= 1;
  }
  const candidate = siblings[previous];
  if (!candidate ||
         candidate[0] !== 'comment' ||
         !concrete(candidate).startsWith('/**')) throw Error('ProvenJSDocAttachmentRequired');
  if (!inspectOnly &&
         (spacing.match(/\r\n|\n|\r/gu) ?? []).length > 1) throw Error('AmbiguousDocumentationAttachment');
  let earlier = previous - 1;
  while (earlier >= 0 &&
         form(siblings[earlier]) === 'whitespace') earlier -= 1;
  if (!inspectOnly &&
         earlier >= 0 &&
         siblings[earlier][0] === 'comment' &&
         concrete(siblings[earlier]).startsWith('/**')) throw Error('AmbiguousDocumentationAttachment');
  const record = all.find(item => item.node === candidate);
  let cursor = document,
    offset = 0;
  for (const index of record.path) {
    offset += cursor.slice(0, index).filter(item => Array.isArray(item) &&
         !excluded.has(item[0])).map(concrete).join('').length;
    cursor = cursor[index];
  }
  const previousLine = sourceFromLinks(document).slice(0, offset).split(/\r\n|\n|\r/u).at(-1);
  if (!inspectOnly &&
         /[^ \t\uFEFF]/u.test(previousLine)) throw Error('AmbiguousTrailingDocumentationAttachment');
  return record;
}
export function createDocumentedParameterRule(document, definition) {
  const grammar = parameterDocumentationGrammar();
  return ['documented-source-transform', structuredClone(contract), ['parameter-rule', createSourceCstTransformRule(document, definition)], ['documentation-grammar', structuredClone(grammar.network)]];
}
function once(document, rule, frontend) {
  const grammar = parameterDocumentationGrammar();
  if (rule[0] !== 'documented-source-transform' ||
         rule.length !== 4 ||
         !equal(rule[1], contract) ||
         !equal(rule[3], ['documentation-grammar', grammar.network])) throw Error('UnsupportedDocumentationRule');
  if (rule[2]?.[0] !== 'parameter-rule' ||
         rule[2].length !== 2) throw Error('UnsupportedDocumentationRule');
  const base = rule[2][1],
    parsed = readSourceCstTransformRule(base);
  if (!equal(parsed.preimage, document)) throw Error('DocumentationSourcePreimageMismatch');
  const language = value(document[0] === 'owned-source' ? document[2] : document, 'language');
  if (!['JavaScript', 'TypeScript'].includes(language)) throw Error('RustDocumentationBindingNotStandardized');
  const record = attachedDocumentation(document, parsed.definition),
    text = concrete(record.node);
  const input = {
    text,
    source: {
      path: sourceOwner(document),
      language
    },
    owner: {
      kind: 'function',
      name: parsed.definition.declaration
    },
    role: 'documentation'
  };
  const model = formalizeParameterDocumentation(input),
    names = model.bindings.map(item => item.binding);
  const functionRecord = records(document).find(item => form(item.node) === 'function_declaration' &&
         children(item.node).some(child => value(child, 'role') === 'name' &&
         concrete(child) === parsed.definition.declaration));
  const parameters = children(functionRecord.node).filter(child => value(child, 'role') === 'params').map(child => concrete(children(child).find(nested => form(nested) === 'identifier') ?? child));
  if (names.some(name => !parameters.includes(name))) throw Error('UnboundDocumentationParameter');
  const naturalized = naturalizeParameterDocumentation(model, parsed.definition.binding, parsed.definition.target);
  const renamed = applySourceCstTransform(document, base, frontend),
    changed = structuredClone(renamed.document);
  const comment = atPath(changed, record.path),
    leaf = optional(comment, 'text');
  if (!leaf) throw Error('UnsupportedNestedDocumentationComment');
  leaf[1] = quoted(naturalized.text);
  verifySourceLinks(changed, frontend);
  const reparsed = ownedSourceLinks(sourceOwner(document), sourceToLinks(sourceFromLinks(changed), language, frontend));
  if (!equal(reparsed, changed)) throw Error('IndependentDocumentationReparseMismatch');
  return {
    document: changed,
    model,
    naturalized,
    definition: parsed.definition,
    commentPath: record.path,
    otherCommentNeeds: renamed.verification.commentNeeds.filter(need => !equal(need.cstPath, record.path))
  };
}
export function applyDocumentedParameterTransform(document, rule, frontend) {
  const result = once(document, rule, frontend);
  const needs = [...result.naturalized.needs.map(need => ({
    ...need,
    owner: sourceOwner(document),
    cstPath: result.commentPath
  })), ...result.otherCommentNeeds];
  const inverse = createDocumentedParameterRule(result.document, {
    ...result.definition,
    binding: result.definition.target,
    target: result.definition.binding
  });
  const restored = once(result.document, inverse, frontend);
  if (!equal(restored.document, document)) throw Error('DocumentationInverseMismatch');
  return {
    document: result.document,
    inverse,
    documentaryModel: result.model,
    needs,
    verification: {
      schema: 'source-cst-documented-alpha-transform/v1',
      binding: 'declaration-qualified-parameter',
      typesAndEffects: 'unchanged-alpha-residual',
      concreteResidual: true,
      sourceReflection: 'not-certified',
      commentMeaning: 'parameter-binding-only',
      commentNeeds: needs,
      documentationSynchronization: needs.length ? 'proven-parameter-tags-with-unresolved-references' : 'proven-parameter-tags-updated',
      arbitrarySemanticEquivalence: false,
      parameterDocumentation: 'proven-syntax-bound-reference',
      arbitraryProse: 'not-certified',
      compilerValidity: 'not-checked',
      exactInverse: true,
      independentReparse: true,
      authorshipCredit: 0
    }
  };
}

/** Read only the exact supported contract and the canonical installed grammar. */
export function readDocumentedParameterRule(notation) {
  const network = typeof notation === 'string' ? parseLinks(notation, {
    quotedText: 'json'
  }) : [notation];
  if (network.length !== 1 ||
         network[0]?.[0] !== 'documented-source-transform' ||
         network[0].length !== 4) throw Error('DocumentedSourceTransformRuleRequired');
  const rule = network[0],
    parsed = readSourceCstTransformRule(rule[2]?.[1]);
  if (!equal(rule, createDocumentedParameterRule(parsed.preimage, parsed.definition))) throw Error('UnsupportedDocumentationRule');
  return {
    ...parsed,
    network: rule
  };
}
export function writeDocumentedParameterRule(rule) {
  const {
    network
  } = readDocumentedParameterRule(rule);
  const pretty = (node, indent = '') => node.every(item => !Array.isArray(item)) ? indent + writeLink(node, {
    quotedText: 'json'
  }) : indent + '(' + node[0] + '\n' + node.slice(1).map(item => pretty(item, indent + '  ')).join('\n') + ')';
  return pretty(network) + '\n';
}
/** Select documentation synchronization only when actual parameter-tag references are present. */
export function sourceCstRuleForObservedParameter(document, definition) {
  const language = value(document[0] === 'owned-source' ? document[2] : document, 'language');
  if (!['JavaScript', 'TypeScript'].includes(language)) return createSourceCstTransformRule(document, definition);
  let record;
  try {
    record = attachedDocumentation(document, definition, true);
  } catch (error) {
    if (['ProvenJSDocAttachmentRequired', 'DocumentationAttachmentUnavailable'].includes(error.message)) return createSourceCstTransformRule(document, definition);
    throw error;
  }
  const model = formalizeParameterDocumentation({
    text: concrete(record.node),
    source: {
      path: sourceOwner(document),
      language
    },
    owner: {
      kind: 'function',
      name: definition.declaration
    },
    role: 'documentation'
  });
  const mentioned = (concrete(record.node).match(/[A-Za-z_$][A-Za-z0-9_$]*/gu) ?? []).includes(definition.binding);
  const relevant = model.bindings.some(item => item.binding === definition.binding) ||
         model.needs.length > 0 &&
         mentioned;
  return relevant ? createDocumentedParameterRule(document, definition) : createSourceCstTransformRule(document, definition);
}
export function readQualifiedSourceCstRule(notation) {
  const parsed = typeof notation === 'string' ? parseLinks(notation, {
    quotedText: 'json'
  }) : [notation];
  if (parsed.length !== 1) throw Error('QualifiedSourceTransformRuleRequired');
  const network = parsed[0];
  return network?.[0] === 'documented-source-transform' ? readDocumentedParameterRule(network) : readSourceCstTransformRule(network);
}
export function writeQualifiedSourceCstRule(rule) {
  return rule[0] === 'documented-source-transform' ? writeDocumentedParameterRule(rule) : writeSourceCstTransformRule(rule);
}
export function applyQualifiedSourceCstRule(document, rule, frontend) {
  return rule[0] === 'documented-source-transform' ? applyDocumentedParameterTransform(document, rule, frontend) : applySourceCstTransform(document, rule, frontend);
}
export function verifyQualifiedSourceCstRule(before, after, rule, frontend) {
  verifySourceLinks(after, frontend);
  if (!equal(applyQualifiedSourceCstRule(before, rule, frontend).document, after)) throw Error('QualifiedTransformResidualMismatch');
  return true;
}
