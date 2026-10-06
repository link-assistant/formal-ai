// The solver event log and its evidence projection: rust/src/event_log.rs
// `EventLog::append` (content-addressed event ids) and `build_evidence_links`
// (one typed link per event, after the prompt link, ending with the response
// link). Every arm of the Rust `match` is here; the arms that address an event
// by its id (`trace:execution_failure`, `formalization`, `search:local`, ...)
// produce exactly the catch-all's `{kind}:{id}`, so they need no row.

import { stableId } from './ids.mjs';

/** Mirrors rust/src/event_log.rs `EventLog` (append, events, first_of). */
export class EventLog {
  constructor(events = []) {
    this.events = [];
    for (const event of events) this.append(event.kind, event.payload ?? '');
  }

  /** `EventLog::append`: the event id is `stable_id(kind, "{kind}:{index}:{payload}")`. */
  append(kind, payload) {
    const text = String(payload);
    const id = stableId(kind, `${kind}:${this.events.length}:${text}`);
    this.events.push({ id, kind: String(kind), payload: text });
    return id;
  }

  firstOf(kind) {
    return this.events.find((event) => event.kind === kind) || null;
  }

  lastOf(kind) {
    for (let index = this.events.length - 1; index >= 0; index -= 1) {
      if (this.events[index].kind === kind) return this.events[index];
    }
    return null;
  }
}

/** Kinds whose link is `{kind}:{payload}`, the payload kept verbatim. */
const PAYLOAD_KINDS = new Set([
  'language', 'language_from', 'language_to',
  'definition_merge:language', 'definition_merge:source_declared',
  'meaning', 'translation_gap', 'wikidata',
  'formalization:subject_q', 'formalization:predicate_p', 'formalization:object_q',
  'formalization:item_q', 'formalization:property_p', 'formalization:fallback',
  'formalization:raw', 'formalization_unresolved', 'statement_weight',
  'intent_formalization:kind', 'intent_formalization:route', 'intent_formalization:relevant',
  'fact_query:relation', 'fact_query:subject', 'fact_query:cache:hit',
  'fact_query:subject_qid', 'fact_query:value_qid',
  'web_search:request', 'web_search:query_kind', 'web_search:provider',
  'web_search:provider_planned', 'web_search:language', 'web_search:combined',
  'web_search:fusion_planned', 'web_search:rank', 'web_search:fused', 'web_search:disabled',
  'document_originality_check:request', 'document_originality_check:attachment',
  'document_originality_check:text_sample',
  'relative_meta_logic:assumed_prior', 'relative_meta_logic:trusted_source_tier',
  'relative_meta_logic:ignored_source_tier',
  'statement_verification:statement_count', 'statement_verification:statement',
  'statement_verification:query', 'statement_verification:assessment',
  'market_price_claim:claim_count', 'market_price_claim:claim', 'market_price_claim:asset',
  'market_price_claim:period', 'market_price_claim:claimed_price', 'market_price_claim:source',
  'market_price_claim:range', 'market_price_claim:assessment',
  'read_local_file:request', 'http_fetch:request',
  'docs_method:project', 'docs_method:method', 'docs_method:source_kind',
  'release_timeline:snapshot', 'release_timeline:sha256', 'release_timeline:released',
  'release_timeline:announced', 'release_timeline:stale',
  'project:promoted', 'project_lookup:promotion', 'project_lookup:repository:github',
  'project_lookup:repository:gitlab', 'project_lookup:repository:bitbucket',
  'url_navigate:request', 'url_preview:iframe', 'tool_call',
  'text_operation', 'text_rule', 'text_rule_chain',
  'procedural_how_to:request', 'procedural_how_to:action', 'procedural_how_to:object',
  'procedural_how_to:stage', 'procedural_how_to:wikihow_candidate', 'procedural_how_to:source_gate',
  'concept_lookup:request', 'concept_lookup:context', 'concept_lookup:response-language',
  'concept_lookup:hit', 'concept_lookup:miss', 'concept_lookup:context-match',
  'concept_lookup:context-mismatch',
  'followup:subject', 'mechanism_query:request', 'mechanism_query:stage', 'mechanism_query:source_gate',
  'source_refresh', 'skill_compile:package', 'skill_compile:procedure',
  'verifiable_task:check', 'verifiable_task:executed', 'verifiable_task:gap',
  'verifiable_task:searched', 'verifiable_task:uncorroborated', 'verifiable_task:membership_source',
  'skill_learning_proposal', 'compiled_skill:replay', 'cache_hit',
  'calculation:engine', 'calculation:lino', 'method:learned:operations_verified',
  'calendar:today', 'calendar:weekday', 'intent',
  'program_parameter:language', 'program_parameter:task', 'legacy_intent',
  'policy:agent_mode_required_for_tools', 'policy:package_permission_required',
  'probability:model', 'filter:user', 'diagnostic_mode',
]);

/** Kinds whose link is `{kind}:{payload with every space as ':'}`. */
const SPACE_AS_COLON_KINDS = new Set([
  'intent_formalization_cache', 'tool_parameter', 'tool_result', 'tool_permission',
  'source:http', 'skill_compile:procedure_step',
]);

/** Kinds whose link is `{kind}:{payload with every space as '_'}` (issue #674). */
const SPACE_AS_UNDERSCORE_KINDS = new Set(['skill_gap', 'program_gap']);

/** Kinds whose link is the bare kind, the payload dropped. */
const BARE_KINDS = new Set([
  'fact_query:cache:miss', 'fact_query:cache:bypass', 'fact_query:force_fresh',
  'policy:chat_bounded_autonomy', 'policy:add_only_history',
  'policy:destructive_action_requires_confirmation', 'policy:cache_flush_requires_confirmation',
  'policy:offline', 'policy:inappropriate_content',
  'policy:guessed_under_ambiguity', 'policy:clarify_under_ambiguity',
]);

/** Kinds whose link is `source:{payload}`. */
const SOURCE_KINDS = new Set(['docs_method:source', 'source', 'release_timeline:source']);

const OFFLINE_SKIP = 'skipped:offline';

/** The typed evidence link for one event: one arm of `build_evidence_links`. */
export function evidenceLink(event) {
  const { kind, id } = event;
  const payload = String(event.payload);
  if (kind === 'response') return payload;
  if (PAYLOAD_KINDS.has(kind)) return `${kind}:${payload}`;
  if (SPACE_AS_COLON_KINDS.has(kind)) return `${kind}:${payload.split(' ').join(':')}`;
  if (SPACE_AS_UNDERSCORE_KINDS.has(kind)) return `${kind}:${payload.split(' ').join('_')}`;
  if (BARE_KINDS.has(kind)) return kind;
  if (SOURCE_KINDS.has(kind)) return `source:${payload}`;
  if (kind === 'release_timeline:hit') return `release_timeline:${payload}`;
  if (kind === 'spelling_correction') return `${kind}:${payload.split(' ').join('')}`;
  if (kind === 'program_parameters') return `${kind}:${payload.replace(/[ ,]/g, ':')}`;
  if (kind === 'search:external' && payload === OFFLINE_SKIP) return 'policy:offline';
  return `${kind}:${id}`;
}

/**
 * Mirrors rust/src/event_log.rs `build_evidence_links`: the prompt link, one
 * link per logged event, then `responseLink` unless a link already equals it.
 * @param {string} prompt
 * @param {EventLog|{events: Array<{id: string, kind: string, payload: string}>}} log
 * @param {string} responseLink
 * @returns {Array<string>}
 */
export function buildEvidenceLinks(prompt, log, responseLink) {
  const links = [`prompt:${stableId('prompt', prompt)}`];
  for (const event of log.events) links.push(evidenceLink(event));
  if (!links.includes(responseLink)) links.push(responseLink);
  return links;
}
