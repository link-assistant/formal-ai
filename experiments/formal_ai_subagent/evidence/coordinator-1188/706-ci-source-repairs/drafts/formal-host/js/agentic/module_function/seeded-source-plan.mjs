import { sourceContractDiagnostic } from "./source-contract-diagnostics.mjs";
import { observedCallableGoalLedger } from "./discovery.mjs";
import { checkedCompositionSource, sourceNullAbsence } from "./source-composition-effects.mjs";
import assert from "./source-invariants.mjs";
import { qualifySeededComposition, sourceGoalScope } from "./seeded-composition-hook.mjs";
import { parseLinoRoot, findChildValue } from "../write_lino.mjs";
import { sentences as sourceSentences } from "../shell_command_policy.mjs";
import { observeSourceCallables } from "./callable_catalog.mjs";
import { sha256Hex } from "../crate/source_fetch.mjs";
import { isKeyword } from "../crate/es_tokenizer.mjs";
const escaped = value => Array.from(value).map(character => ('.*+?^$()|[]{}' + String.fromCharCode(92)).includes(character) ? String.fromCharCode(92) + character : character).join('');
const safe = value => typeof value === 'string' && /^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(value) && !isKeyword(value) && !['eval', 'arguments', 'yield', 'implements', 'interface', 'package', 'private', 'protected', 'public', 'static'].includes(value);
function fill(text, slots) {
  for (let at = 0; at < text.length; at++) {
    if (text[at] !== '{') continue;
    const end = text.indexOf('}', at + 1);
    if (end < 0) continue;
    const name = text.slice(at + 1, end);
    if (/^[a-z]+(?:-[a-z]+)*$/u.test(name)) assert.ok(Object.hasOwn(slots, name), sourceContractDiagnostic("unknown-template-slot"));
  }
  for (const [name, value] of Object.entries(slots)) text = text.split('{' + name + '}').join(value);
  return text;
}
/** Bind a complete seeded return form to a unique source graph. Emission remains a reviewable plan, not execution authority. */
export function deriveSeededSourcePlan(request, observations, acceptance, operation, seed) {
  assert.ok(safe(request.name) && request.parameters.length === 1 && safe(request.parameters[0]), 'unsupported declaration');
  const candidate = qualifySeededComposition(request, observations, acceptance, operation, seed);
  const root = parseLinoRoot(seed).children.find(node => node.name === 'source-callable-composition');
  const goal = root.children.find(node => node.name === 'goal' && node.id === candidate.goal);
  const forms = goal.children.filter(node => node.name === 'form');
  const lexemes = goal.children.filter(node => node.name === 'lexeme' && node.id === 'en').flatMap(node => node.children.filter(value => value.name === 'surface').flatMap(surface => surface.children.filter(value => value.name === 'text').map(value => value.id)));
  const styles = goal.children.filter(node => node.name === 'style').map(node => findChildValue(node, 'surface'));
  const consumer = candidate.graph.bindings[1];
  const observed = observations.find(source => source.path === consumer.path);
  assert.ok(observed, sourceContractDiagnostic("unknown-consumer-source"));
  const catalog = observeSourceCallables(observed.content, observed.path);
  const exported = catalog.exports.find(value => value.exposed === consumer.exported);
  const declaration = catalog.declarations.find(value => value.name === exported.local);
  assert.equal(declaration.parameters.length, 1, sourceContractDiagnostic("unknown-consumer-arity"));
  const consumerParameter = [...declaration.parameters][0];
  assert.ok(safe(consumerParameter));
  const slots = {
    'style': '(?:' + styles.map(escaped).join('|') + ')?',
    'goal': '(?:' + lexemes.map(escaped).join('|') + ')',
    'consumer-parameter': escaped(consumerParameter),
    'wrapper-parameter': escaped(request.parameters[0])
  };
  const sentences = sourceSentences(sourceGoalScope(request.text)).map(sentence => sentence.text);
  const matches = forms.flatMap(form => {
    const technicalValues = {
      'consumer-parameter': consumerParameter,
      'wrapper-parameter': request.parameters[0]
    };
    const technicalCaptures = [];
    const qualifiedPattern = findChildValue(form, 'pattern').replace(/\{(consumer-parameter|wrapper-parameter)\}/gu, (_, role) => {
      const name = 'sourceParameter' + technicalCaptures.length;
      technicalCaptures.push({
        name,
        value: technicalValues[role]
      });
      return '(?<' + name + '>' + slots[role] + ')';
    });
    const pattern = fill(qualifiedPattern, slots);
    return sentences.filter(sentence => {
      const match = new RegExp(pattern, 'iu').exec(sentence);
      return match !== null && technicalCaptures.every(value => match.groups?.[value.name] === value.value);
    }).map(sentence => ({
      form: form.id,
      sentence
    }));
  });
  assert.equal(matches.length, 1, sourceContractDiagnostic("unproved-return-form"));
  const locals = new Set([request.name, ...request.parameters]);
  const fresh = base => {
    let name = base,
      index = 0;
    while (locals.has(name)) name = base + ++index;
    locals.add(name);
    return name;
  };
  const producerLocal = fresh('_sourceProjection'),
    consumerLocal = fresh('_sourceRendering'),
    valueLocal = fresh('_sourceValue');
  const template = goal.children.filter(node => node.name === 'template');
  assert.equal(template.length, 1, 'ambiguous template');
  const text = findChildValue(template[0], 'text');
  assert.ok(text !== '' && candidate.imports.length === 2);
  const producerBinding = candidate.graph.bindings[0];
  const producerObservation = observations.find(value => value.path === producerBinding.path);
  const producerCatalog = observeSourceCallables(producerObservation.content, producerObservation.path);
  const producerExport = producerCatalog.exports.find(value => value.exposed === producerBinding.exported);
  const producerDeclaration = producerCatalog.declarations.find(value => value.name === producerExport.local);
  assert.ok(sourceNullAbsence(producerDeclaration.contract.conditionalIR), sourceContractDiagnostic("unsupported-absence-value"));
  const source = fill(text, {
    'producer-export': candidate.imports[0].exported,
    'producer-local': producerLocal,
    'producer-url': JSON.stringify(candidate.imports[0].moduleURL),
    'consumer-export': candidate.imports[1].exported,
    'consumer-local': consumerLocal,
    'consumer-url': JSON.stringify(candidate.imports[1].moduleURL),
    name: request.name,
    parameter: request.parameters[0],
    'value-local': valueLocal
  });
  assert.ok(candidate.imports.every(value => safe(value.exported)), sourceContractDiagnostic("unsupported-import-binding"));
  const effectPlan = checkedCompositionSource(source, request, candidate.imports);
  return Object.freeze({
    ...candidate,
    effectPlan,
    goalLedger: observedCallableGoalLedger(request, [{
      role: 'user',
      content: request.text
    }]),
    effectAuthorization: 'unproved-complete-Needs',
    kind: 'seeded-source-composition-plan',
    source,
    sourceIdentity: sha256Hex(source),
    seedIdentity: sha256Hex(seed),
    goalClauseCoverage: 'complete-seeded-return-form',
    returnForm: matches[0],
    writeAuthority: false,
    verification: 'not-executed',
    profile: 'conditional JavaScript source-owned plain data; caller effects inherited, not pure'
  });
}
