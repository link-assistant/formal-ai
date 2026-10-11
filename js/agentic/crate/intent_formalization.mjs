// `crate::intent_formalization` (rust/src/intent_formalization.rs): the
// routing-facing intent record a prompt formalizes into, and the events
// `record_intent_formalization` appends to the solver log (R1013).
//
// The write-side planner reads only `impulse_id` and `source_text`
// (`formalizeIntent`); the solver-log twin (`formalizeIntentRecord`) builds
// the whole record. `route_for_prompt` is answered by the booted worker realm
// (`solverRouteForPrompt`: the write_program request, the declared role
// surfaces, the intent routing table), and the promoted handlers by its
// handler-promotion interpreter (`solverPromotedHandlers`, the twin of
// rust/src/handler_promotion.rs `promoted_relevants`). Not mirrored: the
// coding `task_spec` route to `program_synthesis` (the worker has no twin of
// `coding::task_spec::recognise`), and the file-write / quoted-replacement
// guards of `requested_write_program_parameters` (the worker's
// `writeProgramParameters` reads the request as `write_program_parameters`).

import { cached, childValue, childrenNamed, readText, realm } from '../host.mjs';
import { parseLinoRoot } from '../write_lino.mjs';
import { stableId } from './engine_stable_identifier.mjs';
import { normalizePrompt } from './engine.mjs';
import { formatLinoRecord, escapeReference } from './links_format.mjs';
import { explicitLearnedMethodRelevants, methodForRoute, methodRegistry } from './method_registry.mjs';
import { wordsForRoleInLanguages } from './seed_meanings.mjs';

const ROLE_INTERROGATIVE_OPENER = 'interrogative_opener';
const TRANSLATION_PREDICATE = 'wikidata:P5972';
const COURTESY_ROUTES = new Set(['greeting', 'wellbeing', 'farewell', 'courtesy_response']);
const QUESTION_ROUTES = new Set(['assistant_name', 'identity']);
const TASK_ROUTES = new Set([
  'translation', 'algorithm', 'write_program', 'program_synthesis', 'text_manipulation',
  'software_project_plan', 'software_project_implementation',
]);
const REQUIREMENT_TOKENS = ['must', 'should', 'require', 'requires'];
const TASK_TOKENS = ['translate', 'write', 'calculate', 'search', 'find', 'prove', 'define'];

/** Mirrors `fn impulse_id_for`. @param {string} prompt */
export function impulseIdFor(prompt) {
  return stableId('impulse', normalizePrompt(prompt));
}

/**
 * Mirrors `fn formalize_intent` for the fields the planner reads.
 * @param {string} prompt
 * @param {string} language
 * @returns {{impulse_id: string, source_text: string, language: string}}
 */
export function formalizeIntent(prompt, language) {
  return { impulse_id: impulseIdFor(prompt), source_text: prompt, language };
}

/**
 * Mirrors `fn load_cue_sets` in rust/src/cue_lexicon.rs: `{name: {match, cues}}`
 * from data/meta/cue-lexicon.lino, the sets a `cue_set` promotion reads.
 */
export function cueSets() {
  return cached('cue-lexicon-sets', () => {
    const out = {};
    for (const record of parseLinoRoot(readText('data/meta/cue-lexicon.lino')).children || []) {
      if (childValue(record, 'record_type') !== 'cue_set') continue;
      const name = childValue(record, 'name');
      const match = childValue(record, 'match');
      if (!name || !['token', 'substring', 'prefix'].includes(match)) continue;
      out[name] = { match, cues: childrenNamed(record, 'cue').map((cue) => cue.value) };
    }
    return out;
  });
}

function pushUnique(values, value) {
  if (!values.includes(value)) values.push(value);
}

/** Mirrors `fn contains_token`: CJK aliases by substring, the rest by whole token. */
function containsToken(normalized, expected) {
  if (/[぀-ヿ㐀-鿿가-힯]/u.test(expected)) return normalized.includes(expected);
  return normalized.split(/\s+/u).includes(expected);
}

/**
 * Mirrors `fn slot_known_link`.
 * @param {string} role
 * @param {string} kind
 * @param {string} id
 * @returns {string}
 */
function slotKnownLink(role, kind, id) {
  if (kind === 'wikidata_item') {
    if (role === 'subject') return `formalization:subject_q:${id}`;
    if (role === 'object') return `formalization:object_q:${id}`;
    return `formalization:item_q:${id}`;
  }
  if (kind === 'wikidata_property') return role === 'predicate' ? `formalization:predicate_p:${id}` : `formalization:property_p:${id}`;
  if (kind === 'wikipedia_article' || kind === 'wiktionary_entry') return `formalization:fallback:${id}`;
  return `formalization:raw:${id}`;
}

/** Mirrors `fn append_candidate_knowns`. */
function appendCandidateKnowns(candidate, knowns, relevants) {
  for (const slot of candidate.slots) {
    pushUnique(knowns, slotKnownLink(slot.role, slot.anchor.kind, slot.anchor.id));
    if (slot.role === 'predicate' && slot.anchor.id === TRANSLATION_PREDICATE) {
      pushUnique(relevants, 'handler:translation');
      pushUnique(relevants, 'route:translation');
    }
  }
  for (const term of candidate.unresolved_terms) pushUnique(knowns, `formalization_unresolved:${term}`);
}

/** Mirrors `fn route_from_relevants`. */
function routeFromRelevants(relevants) {
  const registry = methodRegistry();
  for (const relevant of relevants) {
    const slug = relevant.startsWith('route:') ? relevant.slice(6) : relevant.startsWith('handler:') ? relevant.slice(8) : null;
    if (slug !== null && methodForRoute(registry, slug)) return slug;
  }
  return null;
}

/** Mirrors `fn starts_with_question_word`. */
function startsWithQuestionWord(normalized) {
  return wordsForRoleInLanguages(ROLE_INTERROGATIVE_OPENER, ['en', 'ru'])
    .some((word) => normalized.startsWith(word) && normalized.slice(word.length).startsWith(' '));
}

/** Mirrors `fn infer_kind`. */
function inferKind(prompt, normalized, route, candidate) {
  if (COURTESY_ROUTES.has(route)) return 'courtesy';
  if (QUESTION_ROUTES.has(route)) return 'question';
  if (TASK_ROUTES.has(route)) return 'task';
  if (prompt.includes('?') || prompt.includes('？') || startsWithQuestionWord(normalized)) return 'question';
  if (REQUIREMENT_TOKENS.some((token) => containsToken(normalized, token))) return 'requirement';
  if (TASK_TOKENS.some((token) => containsToken(normalized, token))) return 'task';
  if (candidate && candidate.slots.length) return 'statement';
  return 'unknown';
}

/** `requested_write_program_parameters` as the sorted `[name, value]` pairs of its `BTreeMap`. */
function writeProgramParameters(prompt) {
  const requested = typeof realm().writeProgramParameters === 'function' ? realm().writeProgramParameters(prompt) : null;
  if (!requested) return [];
  const pairs = [];
  if (requested.language) pairs.push(['language', String(requested.language)]);
  if (requested.task) pairs.push(['task', String(requested.task)]);
  return pairs;
}

/**
 * Mirrors `fn formalize_intent` in rust/src/intent_formalization.rs: the whole
 * `IntentFormalization` record for `prompt`, with the selected formalization
 * candidate (or null) supplying its knowns.
 * @param {string} prompt
 * @param {string} language
 * @param {object|null} candidate
 */
export function formalizeIntentRecord(prompt, language, candidate) {
  const normalized = normalizePrompt(prompt);
  const matched = typeof realm().solverRouteForPrompt === 'function' ? realm().solverRouteForPrompt(prompt) : null;
  const parameters = writeProgramParameters(prompt);
  const impulseId = impulseIdFor(prompt);
  const knowns = [`impulse:${impulseId}`, `language:${language}`];
  const relevants = [];
  if (candidate) appendCandidateKnowns(candidate, knowns, relevants);
  for (const [name, value] of parameters) pushUnique(knowns, `parameter:${name}:${value}`);
  const promoted = typeof realm().solverPromotedHandlers === 'function' ? Array.from(realm().solverPromotedHandlers(prompt, cueSets())) : [];
  for (const handler of promoted) pushUnique(relevants, `handler:${handler}`);
  for (const relevant of explicitLearnedMethodRelevants(methodRegistry(), prompt)) pushUnique(relevants, relevant);
  const route = matched ? String(matched.slug) : routeFromRelevants(relevants);
  if (route !== null) {
    pushUnique(relevants, `route:${route}`);
    if (methodForRoute(methodRegistry(), route)) pushUnique(relevants, `handler:${route}`);
  }
  return {
    impulse_id: impulseId,
    source_text: prompt,
    normalized_text: normalized,
    language,
    kind: inferKind(prompt, normalized, route, candidate),
    knowns,
    relevants,
    parameters,
    route,
    response_link: matched && matched.responseLink ? String(matched.responseLink) : null,
  };
}

/** Mirrors `IntentFormalization::to_links_notation`. */
export function intentFormalizationLinksNotation(record) {
  const pairs = [
    ['impulse_id', record.impulse_id],
    ['source_text', record.source_text],
    ['normalized_text', record.normalized_text],
    ['language', record.language],
    ['kind', record.kind],
  ];
  if (record.route !== null) pairs.push(['route', record.route]);
  if (record.response_link !== null) pairs.push(['response_link', record.response_link]);
  for (const [name, value] of record.parameters) pairs.push(['parameter', `${name}=${value}`]);
  for (const known of record.knowns) pairs.push(['known', known]);
  for (const relevant of record.relevants) pairs.push(['relevant', relevant]);
  return `${formatLinoRecord(`intent_formalization ${escapeReference(record.impulse_id)}`, pairs)}\n`;
}

/**
 * Mirrors `fn record_intent_formalization` for a cache miss, the only state a
 * protocol solve meets (`solve_with_history` opens a fresh cache per turn).
 */
export function recordIntentFormalization(log, record, cacheHit = false) {
  log.push({ kind: 'intent_formalization_cache', payload: `${cacheHit ? 'hit' : 'miss'} ${record.impulse_id}` });
  if (cacheHit) log.push({ kind: 'cache_hit', payload: `intent_formalization:${record.impulse_id}` });
  else log.push({ kind: 'intent_formalization', payload: intentFormalizationLinksNotation(record) });
  log.push({ kind: 'intent_formalization:kind', payload: record.kind });
  if (record.route !== null) log.push({ kind: 'intent_formalization:route', payload: record.route });
  for (const relevant of record.relevants) log.push({ kind: 'intent_formalization:relevant', payload: relevant });
}
