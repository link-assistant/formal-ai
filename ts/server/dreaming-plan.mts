// The dreaming planner and the apply helper that materializes its proposal:
// rust/src/dreaming.rs (`plan_memory_dreaming`, `push_action_once`,
// `deleted_conversation_ids`, `deleted_event_indices`, `duplicate_key`,
// `usage_counts`, `algorithm_candidate_event`, `candidate_failure_event`,
// `synthesized_trial_event`, `pattern_event`, `amendment_event`) and
// rust/src/dreaming/apply.rs (`apply_dreaming_plan`).
//
// Events are the JavaScript `MemoryEvent` shape of js/server/memory-store.mjs;
// a "store" is a plain array of them. Every reason and record sentence is
// data (data/meta/server-messages.lino).

import { stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import { memoryEvent } from './memory-store.mjs';
import { candidateLinksNotation, discoverValidatedAlgorithms, tracesFromMemoryEvents } from './algorithm-discovery.mjs';
import { draftFailureLessons } from './dreaming-draft-failures.mjs';
import { eventTopic, learnFromMemory } from './dreaming-learning.mjs';
import {
  DURABILITY, classifyEvent, isReclaimable, pressurePriority, reclaimableBytes, reconstructionRecord, usageCounts,
} from './dreaming-retention.mjs';
import {
  cmpNum, cmpStr, estimateEventBytes, normalized, percentCeil, requiredReclaimBytes, selectedBytes,
} from './dreaming-support.mjs';

export const ACTION_KIND = Object.freeze({
  PurgeDeletedConversation: 'PurgeDeletedConversation',
  RemoveDuplicateRecomputable: 'RemoveDuplicateRecomputable',
  EvictLowUseRecomputable: 'EvictLowUseRecomputable',
  ForgetCoveredSpecific: 'ForgetCoveredSpecific',
});

/** Mirrors rust/src/dreaming.rs `DreamingConfig::default`. */
export function dreamingConfig(overrides = {}) {
  return {
    daydreaming_enabled: true,
    target_free_ratio_percent: 20,
    storage_capacity_bytes: null,
    free_bytes: null,
    incoming_bytes: 0,
    ...overrides,
  };
}

/** Mirrors rust/src/dreaming.rs `deleted_conversation_ids`. */
function deletedConversationIds(events) {
  return new Set(events
    .filter((event) => event.kind === 'conversation_deleted')
    .map((event) => event.conversation_id)
    .filter((id) => id !== null && id !== undefined));
}

/** Mirrors rust/src/dreaming.rs `deleted_event_indices`. */
function deletedEventIndices(events, deleted) {
  const indices = [];
  events.forEach((event, index) => {
    if (event.conversation_id !== null && event.conversation_id !== undefined && deleted.has(event.conversation_id)) {
      indices.push(index);
    }
  });
  return indices;
}

/** Mirrors rust/src/dreaming.rs `duplicate_key`. */
function duplicateKey(event, durability) {
  if (durability !== DURABILITY.RecomputableCache && durability !== DURABILITY.RecomputableIntermediate) return null;
  return [
    `kind=${normalized(event.kind)}`,
    `role=${normalized(event.role)}`,
    `intent=${normalized(event.intent)}`,
    `tool=${normalized(event.tool)}`,
    `inputs=${normalized(event.inputs)}`,
    `outputs=${normalized(event.outputs)}`,
    `content=${normalized(event.content)}`,
  ].join('|');
}

/** Mirrors rust/src/dreaming.rs `push_action_once`. */
function pushActionOnce(actions, selected, action) {
  if (!action.event_id || selected.has(action.event_id)) return;
  selected.add(action.event_id);
  actions.push(action);
}

function action(kind, event, estimatedBytes, usageCount, reason) {
  return {
    kind,
    event_id: event.id,
    conversation_id: event.conversation_id ?? null,
    estimated_bytes: estimatedBytes,
    usage_count: usageCount,
    reason,
  };
}

/**
 * Mirrors rust/src/dreaming.rs `plan_memory_dreaming`.
 * @param {Array<object>} events
 * @param {object} config see {@link dreamingConfig}
 * @param {(input: string, records: Array<object>) => string} replay
 *   rust/src/dreaming_application.rs `replay_answer_with_amendments`
 */
export function planMemoryDreaming(events, config, replay) {
  const targetFreeRatioPercent = Math.min(config.target_free_ratio_percent, 100);
  const targetFreeBytes = config.storage_capacity_bytes === null || config.storage_capacity_bytes === undefined
    ? null
    : percentCeil(config.storage_capacity_bytes, targetFreeRatioPercent);
  const required = requiredReclaimBytes(targetFreeBytes, config.free_bytes, config.incoming_bytes);
  const base = {
    target_free_ratio_percent: targetFreeRatioPercent,
    storage_capacity_bytes: config.storage_capacity_bytes ?? null,
    free_bytes: config.free_bytes ?? null,
    incoming_bytes: config.incoming_bytes,
    target_free_bytes: targetFreeBytes,
    required_reclaim_bytes: required,
  };
  if (!config.daydreaming_enabled) {
    return {
      daydreaming_enabled: false,
      ...base,
      selected_reclaim_bytes: 0,
      total_reclaimable_bytes: 0,
      requires_bigger_storage: required > 0,
      actions: [],
      observations: [],
      topics: [],
      learned_requirements: [],
      amendments: [],
      candidate_tasks: [],
      patterns: [],
      algorithm_candidates: [],
      synthesized_tasks: [],
      draft_failures: [],
    };
  }

  const deleted = deletedConversationIds(events);
  const usage = usageCounts(events);
  const observations = [];
  const reclaimableCandidates = [];
  const duplicateGroups = new Map();
  events.forEach((event, index) => {
    const durability = classifyEvent(event, deleted);
    const key = duplicateKey(event, durability);
    if (key !== null) {
      if (!duplicateGroups.has(key)) duplicateGroups.set(key, []);
      duplicateGroups.get(key).push(index);
    }
    if (isReclaimable(durability) && event.id) reclaimableCandidates.push(index);
    observations.push({
      event_id: event.id,
      durability,
      usage_count: usage[index],
      estimated_bytes: estimateEventBytes(event),
      duplicate_key: key,
      topic: eventTopic(event),
      covered_by_amendment: false,
    });
  });

  const learning = learnFromMemory(events, observations, deleted, replay);
  const algorithmCandidates = discoverValidatedAlgorithms(tracesFromMemoryEvents(events));
  const draftFailures = draftFailureLessons(events);
  const covered = new Set(learning.amendments.flatMap((amendment) => amendment.covered_event_ids));
  observations.forEach((observation, index) => {
    observation.covered_by_amendment = covered.has(observation.event_id);
    if (observation.covered_by_amendment && events[index].role === 'derived'
      && observation.durability === DURABILITY.IrreplaceableRaw) {
      observation.durability = DURABILITY.RecomputableIntermediate;
      reclaimableCandidates.push(index);
    }
  });
  const reclaimable = (index) => reclaimableBytes(events[index], observations[index].durability);
  const totalReclaimable = reclaimableCandidates.reduce((sum, index) => sum + reclaimable(index), 0);
  const actions = [];
  const selected = new Set();

  for (const index of deletedEventIndices(events, deleted)) {
    pushActionOnce(actions, selected, action(ACTION_KIND.PurgeDeletedConversation, events[index],
      observations[index].estimated_bytes, observations[index].usage_count,
      serverMessage('dreaming_reason_deleted_conversation')));
  }

  for (const [, group] of [...duplicateGroups.entries()].sort(([left], [right]) => cmpStr(left, right))) {
    if (group.length < 2) continue;
    // `Iterator::max_by` keeps the last maximum; the final `right.cmp(left)`
    // makes the earliest index the greatest on a full tie.
    let keep = group[0];
    for (const candidate of group.slice(1)) {
      const order = cmpNum(usage[candidate], usage[keep])
        || cmpNum(observations[candidate].estimated_bytes, observations[keep].estimated_bytes)
        || cmpNum(keep, candidate);
      if (order >= 0) keep = candidate;
    }
    for (const index of group) {
      if (index === keep || reclaimable(index) <= 0) continue;
      pushActionOnce(actions, selected, action(ACTION_KIND.RemoveDuplicateRecomputable, events[index],
        reclaimable(index), observations[index].usage_count,
        serverMessage('dreaming_reason_duplicate', { id: events[keep].id, usage: usage[keep] })));
    }
  }

  let selectedReclaim = selectedBytes(actions);
  if (required > selectedReclaim) {
    const pressure = reclaimableCandidates
      .filter((index) => !selected.has(events[index].id))
      .filter((index) => reclaimable(index) > 0)
      .sort((left, right) => cmpNum(Number(observations[right].covered_by_amendment), Number(observations[left].covered_by_amendment))
        || cmpNum(observations[left].usage_count, observations[right].usage_count)
        || cmpNum(pressurePriority(observations[left].durability), pressurePriority(observations[right].durability))
        || cmpNum(observations[right].estimated_bytes, observations[left].estimated_bytes)
        || cmpNum(left, right));
    for (const index of pressure) {
      if (selectedReclaim >= required) break;
      const coveredSpecific = observations[index].covered_by_amendment;
      pushActionOnce(actions, selected, action(
        coveredSpecific ? ACTION_KIND.ForgetCoveredSpecific : ACTION_KIND.EvictLowUseRecomputable,
        events[index], reclaimable(index), observations[index].usage_count,
        serverMessage(coveredSpecific ? 'dreaming_reason_covered_specific' : 'dreaming_reason_low_use'),
      ));
      selectedReclaim = selectedBytes(actions);
    }
  }

  selectedReclaim = selectedBytes(actions);
  return {
    daydreaming_enabled: true,
    ...base,
    selected_reclaim_bytes: selectedReclaim,
    total_reclaimable_bytes: totalReclaimable,
    requires_bigger_storage: required > selectedReclaim,
    actions,
    observations,
    topics: learning.topics,
    learned_requirements: learning.requirements,
    amendments: learning.amendments,
    candidate_tasks: learning.candidate_tasks,
    patterns: learning.patterns,
    algorithm_candidates: algorithmCandidates,
    synthesized_tasks: learning.synthesized_tasks,
    draft_failures: draftFailures,
  };
}

/** Mirrors rust/src/dreaming.rs `algorithm_candidate_event`. */
function algorithmCandidateEvent(candidate) {
  return memoryEvent({
    id: candidate.evidence_id,
    kind: 'algorithm_learning_candidate',
    role: 'system',
    intent: 'generalize',
    inputs: `steps=${candidate.steps.length}`,
    outputs: 'status=proposal_only',
    content: candidateLinksNotation(candidate),
    evidence: [...candidate.support_trace_ids, ...candidate.held_out.map((test) => test.trace_id)],
  });
}

/** Mirrors rust/src/dreaming.rs `candidate_failure_event`. */
function candidateFailureEvent(candidate) {
  return memoryEvent({
    id: stableId('dreaming_failure', `${candidate.source_event_id}\u0000${candidate.input}`),
    kind: 'dreaming_candidate_failure',
    role: 'system',
    intent: 'generalize',
    inputs: candidate.input,
    outputs: candidate.expected_output,
    content: serverMessage('dreaming_candidate_failure', {
      source: candidate.source_event_id,
      topic: candidate.topic,
      simulated: candidate.simulated_output,
    }),
    demo_label: candidate.topic,
    evidence: [candidate.source_event_id],
  });
}

/** Mirrors rust/src/dreaming.rs `synthesized_trial_event`. */
function synthesizedTrialEvent(trial) {
  return memoryEvent({
    id: stableId('dreaming_trial', `${trial.topic}\u0000${trial.input}`),
    kind: 'dreaming_trial',
    role: 'system',
    intent: 'solve',
    inputs: trial.input,
    outputs: trial.answer,
    content: serverMessage('dreaming_trial', { structure: trial.structure, topic: trial.topic }),
    demo_label: trial.topic,
  });
}

/** Mirrors rust/src/dreaming.rs `pattern_event`. */
function patternEvent(pattern) {
  return memoryEvent({
    id: stableId('dreaming_pattern', `${pattern.topic}\u0000${pattern.structure}`),
    kind: 'dreaming_pattern',
    role: 'system',
    intent: 'generalize',
    inputs: `topic=${pattern.topic}`,
    outputs: `structure=${pattern.structure}`,
    content: serverMessage('dreaming_pattern', { structure: pattern.structure, occurrences: pattern.occurrences }),
    demo_label: pattern.topic,
    evidence: pattern.source_event_ids.slice(),
  });
}

/** Mirrors rust/src/dreaming.rs `amendment_event`. */
function amendmentEvent(amendment) {
  return memoryEvent({
    id: amendment.id,
    kind: 'meta_algorithm_amendment',
    role: 'system',
    intent: 'generalize',
    content: amendment.rule,
    inputs: `topic=${amendment.topic}`,
    outputs: `rule=${amendment.rule}`,
    demo_label: amendment.topic,
    conversation_title: amendment.topic,
    evidence: [...amendment.source_requirement_ids, 'recipe:data/meta/dreaming-recipe.lino'],
  });
}

/** `MemoryStore::from_events`: an absent write count becomes the first write. */
function fromEvents(events) {
  for (const event of events) if (event.write_count === 0) event.write_count = 1;
  return events;
}

/**
 * Mirrors rust/src/dreaming/apply.rs `apply_dreaming_plan`.
 * @returns {{events: Array<object>, outcome: object}} the store after applying,
 *   and the `DreamingOutcome`
 */
export function applyDreamingPlan(events, plan, replay) {
  const current = planMemoryDreaming(events, {
    daydreaming_enabled: plan.daydreaming_enabled,
    target_free_ratio_percent: plan.target_free_ratio_percent,
    storage_capacity_bytes: plan.storage_capacity_bytes,
    free_bytes: plan.free_bytes,
    incoming_bytes: plan.incoming_bytes,
  }, replay);
  const selectedIds = new Set(plan.actions
    .filter((planned) => current.actions.some((now) => now.event_id === planned.event_id)
      && current.observations.filter((observation) => observation.event_id === planned.event_id)
        .every((observation) => isReclaimable(observation.durability)))
    .map((planned) => planned.event_id));
  const existing = new Set(events.map((event) => event.id));
  const reconstructions = events
    .filter((event) => selectedIds.has(event.id))
    .filter((event) => current.observations.some((observation) => observation.event_id === event.id
      && observation.durability === DURABILITY.RecomputableCache))
    .map(reconstructionRecord)
    .filter((record) => record !== null && !existing.has(record.id));
  const fresh = (list) => list.filter((event) => !existing.has(event.id));
  const newAmendments = plan.amendments.filter((amendment) => !existing.has(amendment.id)).map(amendmentEvent);
  const newPatterns = fresh(plan.patterns.map(patternEvent));
  const newAlgorithms = fresh(plan.algorithm_candidates.map(algorithmCandidateEvent));
  const newFailures = fresh(plan.candidate_tasks.filter((candidate) => !candidate.passed).map(candidateFailureEvent));
  const newTrials = fresh(plan.synthesized_tasks.map(synthesizedTrialEvent));
  const outcome = {
    removed_events: 0,
    estimated_reclaimed_bytes: 0,
    learned_amendments: 0,
    learned_patterns: 0,
    recorded_failures: 0,
    recorded_trials: 0,
    learned_algorithm_candidates: 0,
  };
  if (selectedIds.size === 0 && newAmendments.length === 0 && newPatterns.length === 0
    && newAlgorithms.length === 0 && newFailures.length === 0 && newTrials.length === 0) {
    return { events, outcome };
  }
  const initialBytes = events.reduce((sum, event) => sum + estimateEventBytes(event), 0);
  const retained = events.filter((event) => !selectedIds.has(event.id));
  const removed = events.length - retained.length;
  const store = fromEvents([
    ...retained, ...reconstructions, ...newAmendments, ...newPatterns, ...newAlgorithms, ...newFailures, ...newTrials,
  ]);
  const retainedBytes = store.reduce((sum, event) => sum + estimateEventBytes(event), 0);
  return {
    events: store,
    outcome: {
      removed_events: removed,
      estimated_reclaimed_bytes: Math.max(initialBytes - retainedBytes, 0),
      learned_amendments: newAmendments.length,
      learned_patterns: newPatterns.length,
      recorded_failures: newFailures.length,
      recorded_trials: newTrials.length,
      learned_algorithm_candidates: newAlgorithms.length,
    },
  };
}
