// Generic finite own-key storage repair. No task, owner or identifier cases.
import { sourceToLinks, sourceFromLinks, ownedSourceLinks, verifySourceLinks } from '../meta/readable-source.js';
import { observedSourceView, sourceCstViews } from './observed_source_usage.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
const {
  children,
  value,
  form,
  hasRole,
  concrete,
  one
} = sourceCstViews;
const bytes = text => new TextEncoder().encode(text);
const maximumDictionaryCandidates = 32;
const equal = (left, right) => JSON.stringify(left) === JSON.stringify(right);
function policy(knowledge) {
  if (knowledge?.schema !== 'ecmascript-own-key-dictionary-knowledge/v1' || knowledge.language !== 'JavaScript' ||
         knowledge.operation !== 'own-key-storage' || knowledge.prototype !== null || !/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(knowledge.intrinsic ?? '') ||
         !/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(knowledge.method ?? '') || knowledge.initializer !== knowledge.intrinsic + '.' + knowledge.method + '(null)' ||
         !Array.isArray(knowledge.dynamicLookup) || !knowledge.dynamicLookup.every(name => typeof name === 'string') ||
         !Array.isArray(knowledge.sources) || knowledge.sources.length < 2) throw Error('DictionaryKnowledgeUnavailable');
  return knowledge;
}
function selection(source, document, knowledge, repaired) {
  const {
      records,
      byNode
    } = observedSourceView(source, document),
    facets = knowledge.facets;
  const is = (node, key) => Array.isArray(node) && Array.isArray(facets[key]) && facets[key].includes(form(node));
  const record = node => byNode.get(node);
  const parent = node => record(node)?.ancestors.at(-1) ?? null;
  const contains = (ancestor, node) => ancestor === node || record(node)?.ancestors.includes(ancestor);
  const simple = node => node && is(node, 'identifier') && value(node, 'text') !== null ? concrete(node) : null;
  const computed = node => is(node, 'member') && children(node).some(child => concrete(child) === '[');
  const forbidden = records.some(item => ['with_statement'].includes(form(item.node)) || is(item.node, 'identifier') && knowledge.dynamicLookup.includes(concrete(item.node)));
  if (forbidden) throw Error('DictionaryDynamicScopeUnsupported');
  // Resolve the standard intrinsic conservatively: every occurrence is an
  // unshadowed member receiver and no receiver member is assigned or updated.
  for (const item of records.filter(item => simple(item.node) === knowledge.intrinsic)) {
    const member = parent(item.node);
    if (!is(member, 'member') || one(member, ['object']) !== item.node || computed(member)) throw Error('DictionaryIntrinsicShadowedOrEscaped');
    const upper = parent(member);
    if (is(upper, 'assignment') && one(upper, ['left']) === member || ['update_expression', 'unary_expression'].includes(form(upper))) throw Error('DictionaryIntrinsicMutationUnsupported');
  }
  const candidates = [];
  let eligibleBindings = 0;
  for (const item of records.filter(item => is(item.node, 'declarator'))) {
    const declaration = parent(item.node),
      binding = one(item.node, ['name']),
      initializer = one(item.node, ['init', 'value']);
    if (!is(declaration, 'declaration') || !children(declaration).some(child => concrete(child) === 'const') || !initializer) continue;
    const name = simple(binding);
    if (!name || name === knowledge.intrinsic) continue;
    const initialText = concrete(initializer);
    if (repaired ? initialText.replace(/\s/gu, '') !== knowledge.initializer : !is(initializer, 'object') || !/^\{\s*\}$/u.test(initialText)) continue;
    const block = [...item.ancestors].reverse().find(node => is(node, 'block'));
    const ownerFunction = [...item.ancestors].reverse().find(node => is(node, 'function'));
    if (!block || !ownerFunction) continue;
    if (++eligibleBindings > maximumDictionaryCandidates) throw Error('DictionarySelectionBudgetExceeded');
    const occurrences = records.filter(other => simple(other.node) === name);
    let writes = 0,
      reads = 0,
      spreads = 0,
      supported = true;
    const references = [];
    for (const occurrence of occurrences) {
      const node = occurrence.node;
      if (node === binding) continue;
      if (!occurrence.ancestors.includes(block)) {
        supported = false;
        break;
      }
      const upper = parent(node);
      if (is(upper, 'spread') && (one(upper, ['argument']) ?? children(upper).filter(child => !sourceCstViews.trivia(child) && concrete(child) !== '...')[0]) === node && is(parent(upper), 'object')) {
        spreads++;
        references.push({
          span: [occurrence.start, occurrence.end],
          kind: 'own-spread'
        });
        continue;
      }
      if (!computed(upper) || one(upper, ['object']) !== node || !one(upper, ['property', 'index'])) {
        supported = false;
        break;
      }
      const container = parent(upper);
      if (is(container, 'assignment') && one(container, ['left']) === upper) {
        const operators = children(container).filter(child => !hasRole(child, 'left') && !hasRole(child, 'right') && ['=', '+=', '-=', '*=', '/=', '??=', '||=', '&&='].includes(concrete(child)));
        if (operators.length !== 1 || concrete(operators[0]) !== '=') {
          supported = false;
          break;
        }
        writes++;
        references.push({
          span: [occurrence.start, occurrence.end],
          kind: 'computed-write'
        });
        continue;
      }
      if (['update_expression', 'unary_expression'].includes(form(container))) {
        supported = false;
        break;
      }
      if (occurrence.ancestors.some(ancestor => is(ancestor, 'call') && ['callee', 'function', 'constructor', 'tag'].some(role => {
        const called = one(ancestor, [role]);
        return called && contains(called, upper);
      }))) {
        supported = false;
        break;
      }
      reads++;
      references.push({
        span: [occurrence.start, occurrence.end],
        kind: 'computed-read'
      });
    }
    if (!supported || writes === 0 || reads + spreads === 0) continue;
    if (records.some(other => contains(ownerFunction, other.node) && (value(other.node, 'meaning') !== null || value(other.node, 'semantic-gap') !== null))) throw Error('DictionaryUnresolvedSourceMeaning');
    const selected = record(initializer);
    candidates.push({
      binding: name,
      initializer,
      span: [selected.start, selected.end],
      references,
      writes,
      reads,
      spreads
    });
  }
  if (candidates.length !== 1) throw Error(candidates.length ? 'DictionaryBindingAmbiguous' : 'DictionaryBindingUnsupported');
  return candidates[0];
}
export function deriveOwnKeyDictionaryRepair({
  owner,
  source,
  frontend,
  knowledge
}) {
  const rules = policy(knowledge);
  if (typeof owner !== 'string' || !/^[A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:mjs|js)$/u.test(owner) || owner.split('/').some(part => !part || part === '.' || part === '..') || typeof source !== 'string' || bytes(source).length > 262144 || typeof frontend?.parse !== 'function') throw Error('DictionaryObservedSourceRequired');
  const document = ownedSourceLinks(owner, sourceToLinks(source, 'JavaScript', frontend));
  verifySourceLinks(document, frontend);
  if (sourceFromLinks(document) !== source) throw Error('DictionarySourceReconstructionMismatch');
  const selected = selection(source, document, rules, false),
    original = bytes(source),
    [start, end] = selected.span;
  const decode = value => new TextDecoder('utf-8', {
    fatal: true
  }).decode(value);
  const prefix = decode(original.subarray(0, start)),
    before = decode(original.subarray(start, end)),
    suffix = decode(original.subarray(end));
  const replacement = rules.intrinsic + '.' + rules.method + '(' + before.slice(1, -1) + 'null)';
  if (!equal(before.match(/\s/gu) ?? [], replacement.match(/\s/gu) ?? [])) throw Error('DictionaryInitializerTriviaMismatch');
  const candidate = prefix + replacement + suffix;
  if (bytes(candidate).length > 262144) throw Error('DictionaryCandidateBudgetExceeded');
  const changed = ownedSourceLinks(owner, sourceToLinks(candidate, 'JavaScript', frontend));
  verifySourceLinks(changed, frontend);
  if (sourceFromLinks(changed) !== candidate) throw Error('DictionaryCandidateReconstructionMismatch');
  const after = selection(candidate, changed, rules, true);
  if (after.binding !== selected.binding || after.writes !== selected.writes || after.reads !== selected.reads || after.spreads !== selected.spreads) throw Error('DictionaryBindingProofMismatch');
  const restored = candidate.slice(0, prefix.length) + before + candidate.slice(prefix.length + replacement.length);
  if (restored !== source || !equal(ownedSourceLinks(owner, sourceToLinks(restored, 'JavaScript', frontend)), document)) throw Error('DictionaryExactInverseMismatch');
  return {
    schema: 'observed-own-key-dictionary-repair/v1',
    owner,
    candidate: {
      path: owner,
      source: candidate,
      beforeSha256: sha256Hex(source),
      afterSha256: sha256Hex(candidate),
      changed: true
    },
    binding: {
      name: selected.binding,
      initializerSpan: selected.span,
      references: selected.references
    },
    knowledge: {
      schema: rules.schema,
      sources: rules.sources,
      assumptions: rules.assumptions
    },
    verification: {
      schema: 'own-key-dictionary-source-proof/v1',
      completeConcreteResidual: true,
      initializerTriviaPreserved: true,
      independentGrammarReparse: true,
      exactInverse: true,
      writeCount: selected.writes,
      readCount: selected.reads,
      spreadCount: selected.spreads,
      scope: rules.claim,
      arbitrarySemanticEquivalence: false,
      sourceReflectionEquivalence: false,
      compilerValidity: 'not-checked',
      authorshipCredit: 0
    },
    authored: false,
    verified: false,
    independentAcceptance: false
  };
}
