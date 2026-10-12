// Symbolic next-request anticipation for idle dreaming (issue #705): the twin
// of rust/src/anticipation.rs (`plan_anticipation`, `apply_anticipation`,
// `prediction_hit_event`, `run_idle_anticipation`, `anticipation_ledger_path`,
// `AnticipationPlan::links_notation`, `AnticipationPlan::why_prediction`),
// rust/src/anticipation/ledger.rs (`AnticipationLedger::links_notation`),
// rust/src/anticipation/prelearning.rs (`prelearn_predictions`,
// `PrelearningRun::links_notation`) and the Markov transition record of
// rust/src/probability.rs (`ProbabilityEvidence::symbolic`,
// `to_links_notation`, `stable_record_id`).
//
// Request classes are formalized intents, never raw prompts: each user turn
// is one observation, adjacent observations in one conversation are a
// transition, and the classes that followed the current one are predicted.
// Each prediction is expanded from evidence (js/server/anticipation-expansion.mjs)
// and probed offline; every probe the solver cannot route reaches the
// proposal-only adoption frontier (js/server/learning-cycle.mjs). Fetching a
// source for an unrouted prediction needs consent (FORMAL_AI_LIVE_API).
//
// `IntentClass` keeps only `id` and `intent`: the Rust class also carries the
// formalization kind, route and operations, which no record, ledger or
// comparison reads.

import { normalizePrompt } from '../agentic/crate/engine.mjs';
import { detect } from '../agentic/crate/language.mjs';
import { formatLinoRecord } from '../agentic/crate/links_format.mjs';
import { expandClass } from './anticipation-expansion.mjs';
import { debugOption } from './dreaming-support.mjs';
import { solveIntent } from './dreaming-replay.mjs';
import { stableId } from './ids.mjs';
import { ensureDataHost, learningCycleLinksNotation, runLearningCycle } from './learning-cycle.mjs';
import { memoryEvent, parseBoolEnv, writeAtomic } from './memory-store.mjs';

export const ANTICIPATION_FRONTIER = 'anticipation';
export const ANTICIPATION_PREDICTION_KIND = 'anticipation_prediction';
export const ANTICIPATION_SOURCE_KIND = 'anticipation_source';
export const PREDICTION_HIT_KIND = 'prediction_hit';
const DEFAULT_CACHE_TTL_SECONDS = 60 * 60 * 24 * 60;

/** Mirrors `AnticipationConfig::default`. */
export function anticipationConfig(overrides = {}) {
  return { max_predictions: 3, max_variations_per_prediction: 16, source_page_limit: 1, ttl_seconds: 3600, ...overrides };
}

const ALPHANUMERIC_OR_UNDERSCORE = /[\p{Alphabetic}\p{N}_]/u;

/** Mirrors `identifier` in rust/src/anticipation.rs. */
export function identifier(value) {
  let out = '';
  let separator = false;
  for (const character of String(value)) {
    if (ALPHANUMERIC_OR_UNDERSCORE.test(character)) {
      if (separator && out) out += '_';
      out += character.toLowerCase();
      separator = false;
    } else {
      separator = true;
    }
  }
  return out || stableId('intent_class', value);
}

/** An `IntentClass` from a solver intent. */
const classOf = (intent) => ({ id: `intent:${identifier(intent)}`, intent });

/** Mirrors `classify_prompt`: `{class, intent}` from the offline solver. */
function classifyWith(classify, prompt) {
  const intent = classify(prompt);
  return { class: classOf(intent), intent };
}

/** Mirrors `intent_class_for_prompt`. */
export function intentClassForPrompt(prompt, classify = solveIntent) {
  return classifyWith(classify, prompt).class;
}

/** Rust `str` ordering (UTF-8 bytes, which code points preserve). */
function compareStr(left, right) {
  const a = Array.from(left);
  const b = Array.from(right);
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    const diff = a[index].codePointAt(0) - b[index].codePointAt(0);
    if (diff !== 0) return diff;
  }
  return a.length - b.length;
}

/** Rust `format!("{:.6}", f32)`. */
const f32Fixed = (value) => Math.fround(Number.isFinite(value) ? value : 0).toFixed(6);

/** A `ProbabilityEvidence` built by `symbolic(..).with_model(MarkovTransition).with_transition_from(..)`. */
function markovEvidence(target, observation, weight, provenance, recordedAt, transitionFrom) {
  const evidence = {
    target, observation, weight: Number.isFinite(weight) ? weight : 0, model: 'markov_transition', provenance,
    recorded_at: recordedAt, transition_from: transitionFrom,
  };
  evidence.id = stableId('probability', [target, observation, f32Fixed(evidence.weight), evidence.model, provenance,
    recordedAt, debugOption(transitionFrom), ''].join(':'));
  return evidence;
}

/** Mirrors `ProbabilityEvidence::to_links_notation`. */
function evidenceLinksNotation(evidence) {
  return formatLinoRecord('probability_evidence', [
    ['id', evidence.id], ['target', evidence.target], ['observation', evidence.observation],
    ['weight', f32Fixed(evidence.weight)], ['model', evidence.model], ['provenance', evidence.provenance],
    ['recorded_at', evidence.recorded_at], ['transition_from', evidence.transition_from],
  ]);
}

/** Mirrors `observations`: one per non-empty user turn, its class overridden by a recorded intent. */
function observations(events, classify) {
  const out = [];
  events.forEach((event, index) => {
    if (event.role !== 'user' || event.content === null || event.content === undefined) return;
    const prompt = String(event.content).trim();
    if (!prompt) return;
    // A recorded intent replaces the solver's class, so only an unlabelled turn is solved.
    const intentClass = event.intent ? classOf(event.intent) : intentClassForPrompt(prompt, classify);
    out.push({
      event_id: event.id,
      prompt,
      class: intentClass,
      recorded_at: event.sent_at ?? `append:${index}`,
      conversation_id: event.conversation_id ?? null,
    });
  });
  return out;
}

/** Mirrors `transition_counts`. */
function transitionCounts(observed) {
  const groups = new Map();
  for (let index = 0; index + 1 < observed.length; index += 1) {
    const from = observed[index];
    const to = observed[index + 1];
    if (from.conversation_id !== null && to.conversation_id !== null && from.conversation_id !== to.conversation_id) continue;
    const key = JSON.stringify([from.class.id, to.class.id]);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push([from, to]);
  }
  const outgoing = new Map();
  for (const [key, pairs] of groups) {
    const [from] = JSON.parse(key);
    outgoing.set(from, (outgoing.get(from) ?? 0) + pairs.length);
  }
  const keys = [...groups.keys()].sort((left, right) => {
    const [lf, lt] = JSON.parse(left);
    const [rf, rt] = JSON.parse(right);
    return compareStr(lf, rf) || compareStr(lt, rt);
  });
  return keys.map((key) => {
    const pairs = groups.get(key);
    const from = pairs[0][0].class;
    const to = pairs[0][1].class;
    const count = pairs.length;
    const total = Math.max(outgoing.get(from.id) ?? count, 1);
    const probability = Math.fround(count / total);
    const evidenceLinks = pairs.flatMap(([left, right]) => [`memory:${left.event_id}`, `memory:${right.event_id}`]);
    const recordedAt = pairs.at(-1)?.[1].recorded_at ?? '';
    const evidence = markovEvidence(to.id, `count=${count};outgoing=${total}`, probability, evidenceLinks.join(','), recordedAt, from.id);
    return { from, to, count, probability, evidence, evidence_links: evidenceLinks };
  });
}

/**
 * Mirrors `plan_anticipation` in rust/src/anticipation.rs.
 * @param {Array<object>} events
 * @param {object} config see {@link anticipationConfig}
 * @param {(prompt: string) => string} [classify] the offline solver's intent
 */
export function planAnticipation(events, config = anticipationConfig(), classify = solveIntent) {
  ensureDataHost();
  const observed = observations(events, classify);
  const transitions = transitionCounts(observed);
  const current = observed.at(-1)?.class ?? null;
  let predictions = current === null ? [] : transitions
    .filter((transition) => transition.from.id === current.id)
    .map((transition) => ({
      id: stableId('anticipation_prediction', `${current.id}:${transition.to.id}:${transition.evidence.id}`),
      class: transition.to,
      rank: 0,
      count: transition.count,
      probability: transition.probability,
      transition_evidence_id: transition.evidence.id,
      evidence_links: transition.evidence_links.slice(),
      variants: expandClass(transition.to, observed, config),
    }));
  predictions.sort((left, right) => right.count - left.count || compareStr(left.class.id, right.class.id));
  predictions = predictions.slice(0, config.max_predictions);
  predictions.forEach((prediction, index) => {
    prediction.rank = index + 1;
  });
  const probes = [];
  const frontier = [];
  for (const prediction of predictions) {
    for (const variant of prediction.variants) {
      const { class: actual, intent: engineIntent } = classifyWith(classify, variant.prompt);
      let status = 'failed';
      if (engineIntent === 'unknown') status = 'unknown';
      else if (actual.id === prediction.class.id) status = 'passed';
      const language = detect(variant.prompt);
      probes.push({
        prediction_id: prediction.id,
        prompt: variant.prompt,
        base_event_id: variant.base_event_id,
        variation_source: variant.source,
        expected_class: prediction.class.id,
        actual_class: actual.id,
        engine_intent: engineIntent,
        language,
        status,
      });
      if (status !== 'passed') {
        frontier.push({
          rank: frontier.length + 1,
          query: variant.prompt,
          language,
          variation: `anticipation_${identifier(variant.source)}`,
          prompt: variant.prompt,
          engine_intent: engineIntent,
        });
      }
    }
  }
  return {
    current_class: current, transitions, predictions, probes, frontier,
    learning_cycle: runLearningCycle(ANTICIPATION_FRONTIER, frontier),
  };
}

/** Mirrors `AnticipationPlan::why_prediction`. */
export function whyPrediction(plan, predictionId) {
  const prediction = plan.predictions.find((candidate) => candidate.id === predictionId);
  if (!prediction) return null;
  return `prediction=${prediction.id} class=${prediction.class.id} transition_evidence=${prediction.transition_evidence_id}`
    + ` count=${prediction.count} probability=${f32Fixed(prediction.probability)}`;
}

/** Mirrors `AnticipationPlan::links_notation`. */
export function anticipationPlanLinksNotation(plan) {
  const records = [formatLinoRecord('anticipation_plan', [
    ['record_type', 'anticipation_plan'], ['issue', '705'], ['model', 'markov_transition'], ['deterministic', 'true'],
    ['current_class', plan.current_class?.id ?? ''], ['predictions', String(plan.predictions.length)],
    ['probes', String(plan.probes.length)], ['frontier_items', String(plan.frontier.length)],
  ])];
  records.push(...plan.transitions.map((transition) => evidenceLinksNotation(transition.evidence)));
  for (const prediction of plan.predictions) {
    records.push(formatLinoRecord('anticipation_prediction', [
      ['record_type', 'anticipation_prediction'], ['id', prediction.id], ['class', prediction.class.id],
      ['rank', String(prediction.rank)], ['count', String(prediction.count)],
      ['probability', f32Fixed(prediction.probability)], ['transition_evidence', prediction.transition_evidence_id],
      ...prediction.evidence_links.map((link) => ['evidence', link]),
    ]));
    for (const variant of prediction.variants) {
      records.push(formatLinoRecord('anticipation_variant', [
        ['prediction', prediction.id], ['prompt', variant.prompt], ['source', variant.source], ['base_event', variant.base_event_id],
      ]));
    }
  }
  for (const probe of plan.probes) {
    records.push(formatLinoRecord('anticipation_probe', [
      ['prediction', probe.prediction_id], ['prompt', probe.prompt], ['expected_class', probe.expected_class],
      ['actual_class', probe.actual_class], ['engine_intent', probe.engine_intent], ['status', probe.status],
      ['variation_source', probe.variation_source],
    ]));
  }
  records.push(learningCycleLinksNotation(plan.learning_cycle));
  return records.join('\n');
}

/** Mirrors `source_trace`. */
function sourceTrace(source) {
  return `${source.source_url} fetched_at=${source.fetched_at} sha256=${source.sha256} cached=${source.cached}`;
}

/** Mirrors `source_record`. */
function sourceRecord(source, alias = null) {
  const fields = [
    ['record_type', 'prelearned_source'], ['id', source.id], ['prediction', source.prediction_id], ['class', source.class_id],
    ['base_event', source.base_event_id], ['query', source.query], ['result_url', source.result_url],
    ['source_url', source.source_url], ['fetched_at', source.fetched_at], ['sha256', source.sha256],
    ['cached', String(source.cached)], ['expires_at', String(source.expires_at)], ['source_trace', sourceTrace(source)],
  ];
  if (alias !== null) fields.push(['alias', alias]);
  return formatLinoRecord('anticipation_source', fields);
}

/** Mirrors `PrelearningRun::links_notation`. */
export function prelearningLinksNotation(run) {
  return [
    ...run.attempts.map((attempt) => formatLinoRecord('anticipation_prelearning_attempt', [
      ['prediction', attempt.prediction_id], ['query', attempt.query], ['status', attempt.status], ['diagnostic', attempt.diagnostic],
    ])),
    ...run.sources.map((source) => sourceRecord(source)),
  ].join('\n');
}

/**
 * Mirrors `prelearn_predictions` in rust/src/anticipation/prelearning.rs.
 * `research(query, pageLimit)` is `execute_source_research`: it returns
 * `{search: {fused: [{url, title, excerpt}], captures}, pages: [{ranking: {url}, capture}]}`
 * with captures `{source_url, fetched_at, sha256, cached}`, or throws.
 * @param {object} plan
 * @param {'granted'|'denied'} consent
 * @param {object} config
 * @param {(query: string, pageLimit: number) => object} research
 */
export function prelearnPredictions(plan, consent, config, research) {
  const run = { attempts: [], sources: [] };
  const attempt = (prediction, probe, status, diagnostic) => run.attempts.push({
    prediction_id: prediction.id, query: probe.prompt, status, diagnostic,
  });
  for (const prediction of plan.predictions) {
    const probe = plan.probes.find((candidate) => candidate.prediction_id === prediction.id && candidate.status !== 'passed');
    if (!probe) continue;
    if (consent !== 'granted') {
      attempt(prediction, probe, 'consent_required', 'fetch_consent_required');
      continue;
    }
    let found;
    try {
      found = research(probe.prompt, config.source_page_limit);
    } catch (error) {
      attempt(prediction, probe, 'fetch_failed', String(error?.message ?? error));
      continue;
    }
    const result = found.search.fused[0];
    if (!result) {
      attempt(prediction, probe, 'no_source', 'search_result_absent');
      continue;
    }
    const answer = result.excerpt.trim() === '' ? result.title.trim() : result.excerpt.trim();
    if (!answer) {
      attempt(prediction, probe, 'no_source', 'search_excerpt_absent');
      continue;
    }
    const capture = found.pages.find((page) => page.ranking.url === result.url)?.capture ?? found.search.captures[0];
    if (!capture) continue;
    const fetchedAt = /^\+?\d+$/u.test(capture.fetched_at) ? Number(capture.fetched_at) : 0;
    run.sources.push({
      id: stableId('anticipation_source', `${prediction.id}:${result.url}:${capture.sha256}`),
      prediction_id: prediction.id,
      class_id: prediction.class.id,
      base_event_id: probe.base_event_id,
      query: probe.prompt,
      aliases: prediction.variants.filter((variant) => variant.base_event_id === probe.base_event_id).map((variant) => variant.prompt),
      answer,
      result_url: result.url,
      source_url: capture.source_url,
      fetched_at: capture.fetched_at,
      sha256: capture.sha256,
      cached: capture.cached,
      expires_at: fetchedAt + config.ttl_seconds,
    });
    attempt(prediction, probe, 'captured', result.url);
  }
  return run;
}

const emptyOutcome = () => ({ probability_records: 0, prediction_records: 0, prelearned_aliases: 0 });

/** `AnticipationOutcome::recorded_events`. */
export const recordedEvents = (outcome) => outcome.probability_records + outcome.prediction_records + outcome.prelearned_aliases;

/**
 * Mirrors `apply_anticipation`: the store with the new transition,
 * prediction and alias records appended, and the counts.
 * @returns {{events: Array<object>, outcome: object}}
 */
export function applyAnticipation(events, plan, prelearning) {
  const known = new Set(events.map((event) => event.id));
  const out = events.slice();
  const outcome = emptyOutcome();
  const append = (event) => {
    known.add(event.id);
    out.push(memoryEvent({ ...event, write_count: 1 }));
  };
  for (const transition of plan.transitions) {
    if (known.has(transition.evidence.id)) continue;
    append({
      id: transition.evidence.id, kind: 'probability_evidence', role: 'system', intent: 'markov_transition',
      inputs: transition.from.id, outputs: transition.to.id, content: evidenceLinksNotation(transition.evidence),
      evidence: transition.evidence_links.slice(),
    });
    outcome.probability_records += 1;
  }
  for (const prediction of plan.predictions) {
    if (known.has(prediction.id)) continue;
    append({
      id: prediction.id, kind: ANTICIPATION_PREDICTION_KIND, role: 'system', intent: prediction.class.id,
      inputs: plan.current_class?.id ?? null, outputs: prediction.class.id,
      content: formatLinoRecord('anticipation_prediction', [
        ['rank', String(prediction.rank)], ['count', String(prediction.count)],
        ['probability', f32Fixed(prediction.probability)], ['transition_evidence', prediction.transition_evidence_id],
      ]),
      evidence: [...prediction.evidence_links, `probability_evidence:${prediction.transition_evidence_id}`],
    });
    outcome.prediction_records += 1;
  }
  for (const source of prelearning.sources) {
    for (const alias of source.aliases) {
      const id = stableId('anticipation_source_alias', `${source.id}:${normalizePrompt(alias)}`);
      if (known.has(id)) continue;
      append({
        id, kind: ANTICIPATION_SOURCE_KIND, role: 'system', intent: source.class_id, inputs: alias, outputs: source.answer,
        content: sourceRecord(source, alias), sent_at: source.fetched_at, demo_label: source.query,
        evidence: [`anticipation_prediction:${source.prediction_id}`, `source:http:${sourceTrace(source)}`],
      });
      outcome.prelearned_aliases += 1;
    }
  }
  return { events: out, outcome };
}

/**
 * Mirrors `prediction_hit_event`: the hit linking a later actual request to
 * the newest prediction of its class, or null. Only a store that holds a
 * prediction is classified.
 */
export function predictionHitEvent(events, prompt, actualRequestId, classify = solveIntent) {
  if (!events.some((event) => event.kind === ANTICIPATION_PREDICTION_KIND)) return null;
  const intentClass = intentClassForPrompt(prompt, classify);
  const prediction = events.findLast((event) => event.kind === ANTICIPATION_PREDICTION_KIND && event.intent === intentClass.id);
  if (!prediction) return null;
  return memoryEvent({
    id: stableId('prediction_hit', `${prediction.id}:${actualRequestId}`),
    kind: PREDICTION_HIT_KIND,
    role: 'system',
    intent: intentClass.id,
    inputs: prediction.id,
    outputs: actualRequestId,
    content: formatLinoRecord('prediction_hit', [
      ['prediction', prediction.id], ['actual_request', actualRequestId], ['actual_class', intentClass.id],
    ]),
    evidence: [`anticipation_prediction:${prediction.id}`, actualRequestId],
    write_count: 1,
  });
}

/** Mirrors `AnticipationLedger::links_notation` in rust/src/anticipation/ledger.rs. */
export function anticipationLedgerLinksNotation(plan, prelearning, events) {
  const predictionIds = new Set(plan.predictions.map((prediction) => prediction.id));
  const predictedBy = (event) => event.evidence
    .map((link) => (link.startsWith('anticipation_prediction:') ? link.slice('anticipation_prediction:'.length) : null))
    .find((id) => id !== null && predictionIds.has(id));
  const hits = events.filter((event) => event.kind === PREDICTION_HIT_KIND && predictedBy(event) !== undefined);
  const classesHit = new Set(hits.map(predictedBy));
  const rate = plan.predictions.length === 0 ? 0 : Math.floor((classesHit.size * 10000) / plan.predictions.length);
  const records = [formatLinoRecord('anticipation_ledger', [
    ['record_type', 'anticipation_ledger'], ['issue', '705'], ['predictions', String(plan.predictions.length)],
    ['probe_results', String(plan.probes.length)], ['prelearned_sources', String(prelearning.sources.length)],
    ['prediction_hits', String(hits.length)], ['predicted_classes_hit', String(classesHit.size)],
    ['hit_rate_basis_points', String(rate)], ['deterministic', 'true'], ['mode', 'proposal_only'], ['human_gated', 'true'],
  ])];
  records.push(anticipationPlanLinksNotation(plan));
  const prelearned = prelearningLinksNotation(prelearning);
  if (prelearned) records.push(prelearned);
  for (const hit of hits) if (hit.content !== null && hit.content !== undefined) records.push(hit.content);
  return records.join('\n');
}

/** Mirrors `anticipation_ledger_path`: `Path::with_extension("anticipation.lino")`. */
export function anticipationLedgerPath(memoryPath) {
  const dot = memoryPath.lastIndexOf('.');
  const slash = Math.max(memoryPath.lastIndexOf('/'), memoryPath.lastIndexOf('\\'));
  const stem = dot > slash + 1 ? memoryPath.slice(0, dot) : memoryPath;
  return `${stem}.anticipation.lino`;
}

/** No source-research port runs in the JavaScript server; a consented fetch reports that. */
function unportedResearch() {
  throw new Error('source_research_unavailable');
}

/**
 * Mirrors `run_idle_anticipation`: plan, prelearn under FORMAL_AI_LIVE_API
 * consent, append the records, and write the ledger beside the memory log.
 * @returns {{events: Array<object>, outcome: object}}
 */
export function runIdleAnticipation(memoryPath, events, { env = process.env, classify = solveIntent, research = unportedResearch } = {}) {
  const ttl = /^\d+$/u.test(env.FORMAL_AI_CACHE_TTL_SECONDS ?? '') ? Number(env.FORMAL_AI_CACHE_TTL_SECONDS) : DEFAULT_CACHE_TTL_SECONDS;
  const config = anticipationConfig({ ttl_seconds: ttl });
  const plan = planAnticipation(events, config, classify);
  const consent = parseBoolEnv(env.FORMAL_AI_LIVE_API) === true ? 'granted' : 'denied';
  const prelearning = prelearnPredictions(plan, consent, config, research);
  const applied = applyAnticipation(events, plan, prelearning);
  writeAtomic(anticipationLedgerPath(memoryPath), `${anticipationLedgerLinksNotation(plan, prelearning, applied.events)}\n`);
  return applied;
}
