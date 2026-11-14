// Topic/requirement learning, amendment generalization, candidate-task
// replay, failure-driven refinement, language-independent structure mining
// and trial synthesis: rust/src/dreaming/learning.rs.
//
// Every replay goes through `replay(input, records)`, the port of
// rust/src/dreaming_application.rs `replay_answer_with_amendments` that
// js/server/dreaming-replay.mjs supplies (the production amendment path over
// the worker solver), so "covered" keeps meaning "the production path
// re-derives the stored output".

import { stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import {
  asciiLower, cmpNum, cmpStr, isAlphanumeric, lexicon, requirementCues, rustStrLines, splitWhitespace, stringBytes, trim,
  trimMatches,
} from './dreaming-support.mjs';
import {
  boundedStatement, isBoundedRule, isValidTopic, ruleMaxChars, statementTopic,
} from './standing-requirement-bounds.mjs';
import { inScope, scopeKey, scopePairs } from './standing-scope.mjs';

const NO_DIGITS = /^[0-9]+$/;

/** A `BTreeMap` stand-in: a `Map` whose keys are read back in Rust `str` order. */
function sortedEntries(map) {
  return [...map.entries()].sort(([left], [right]) => cmpStr(left, right));
}

/** Mirrors rust/src/dreaming/learning.rs `is_topic_stopword`. */
function isTopicStopword(word) {
  return lexicon().topic_stopwords.some((stopword) => stopword === word);
}

/** Mirrors rust/src/dreaming/learning.rs `first_significant_word`. */
function firstSignificantWord(text) {
  for (const raw of splitWhitespace(text)) {
    const token = trimMatches(raw, (character) => !isAlphanumeric(character)).toLowerCase();
    const count = [...token].length;
    if (count >= 3 && !NO_DIGITS.test(token) && !isTopicStopword(token)) return token;
  }
  return null;
}

/** Mirrors rust/src/dreaming/learning.rs `metadata_topic`: an explicit topic label of the event. */
function metadataTopic(event) {
  for (const raw of [event.conversation_title, event.demo_label, event.intent, event.tool]) {
    if (raw === null || raw === undefined) continue;
    const normalized = trim(String(raw)).toLowerCase();
    if (normalized && !isTopicStopword(normalized)) return normalized;
  }
  return null;
}

/** Mirrors rust/src/dreaming/learning.rs `event_topic`. */
export function eventTopic(event) {
  const labelled = metadataTopic(event);
  if (labelled !== null) return labelled;
  for (const raw of [event.content, event.inputs]) {
    if (raw === null || raw === undefined) continue;
    const topic = firstSignificantWord(String(raw));
    if (topic !== null) return topic;
  }
  return null;
}

/** Mirrors rust/src/dreaming/learning.rs `is_task_event`. */
export function isTaskEvent(event) {
  const kind = String(event.kind ?? '').toLowerCase();
  const intent = String(event.intent ?? '').toLowerCase();
  const cues = lexicon();
  return cues.task_kind_cues.some((cue) => kind.includes(cue))
    || cues.task_intent_cues.some((cue) => intent.includes(cue));
}

/**
 * Roles whose text is never a user's standing requirement (issue #1559 §1):
 * the assistant's own words, and system, developer or session instructions —
 * including one a client delivered as the first user message.
 */
const NON_REQUIREMENT_ROLES = new Set(['assistant', 'system', 'developer', 'instruction']);

/** Mirrors rust/src/dreaming/learning.rs `requirement_statement`. */
function requirementStatement(event) {
  const role = event.role === null || event.role === undefined ? null : asciiLower(event.role);
  if (role !== null && NON_REQUIREMENT_ROLES.has(role)) return null;
  if (event.content === null || event.content === undefined) return null;
  return boundedStatement(String(event.content));
}

/**
 * Mirrors rust/src/dreaming/learning.rs `requirement_topic`: the explicit
 * topic label, else the subject of the statement itself; never a pronoun or
 * function word.
 */
function requirementTopic(event, statement) {
  const topic = metadataTopic(event) ?? statementTopic(statement);
  return topic !== null && isValidTopic(topic) ? topic : null;
}

/** Mirrors rust/src/dreaming/learning.rs `scope_of`: the scope pairs as a lookup object. */
function scopeOf(event) {
  return Object.fromEntries(scopePairs(event.evidence));
}

/** Mirrors rust/src/dreaming/learning.rs `amendment_id`. */
function amendmentId(topic, scope) {
  const key = scopeKey(scope);
  return key ? stableId('amendment', `${topic}\u0000${key}`) : stableId('amendment', topic);
}

/** Mirrors rust/src/dreaming/learning.rs `bounded_rule`: statements joined while the rule stays in bounds. */
function boundedRule(statements) {
  const limit = ruleMaxChars();
  let rule = '';
  for (const statement of statements) {
    const next = rule ? `${rule}; ${statement}` : statement;
    if ([...next].length > limit) continue;
    rule = next;
  }
  return rule;
}

/** Mirrors rust/src/dreaming/learning.rs `normalize`. */
function normalize(value) {
  return splitWhitespace(String(value)).join(' ');
}

/** Mirrors rust/src/dreaming/learning.rs `retained_record`. */
function retainedRecord(amendment) {
  return { id: amendment.id, topic: amendment.topic, rule: amendment.rule };
}

/** Mirrors rust/src/dreaming/learning.rs `amendment_reproduces_specific`. */
function amendmentReproducesSpecific(amendment, event, replay) {
  if (event.inputs === null || event.inputs === undefined) return false;
  if (event.outputs === null || event.outputs === undefined) return false;
  return normalize(replay(event.inputs, [retainedRecord(amendment)])) === normalize(event.outputs);
}

/** Mirrors rust/src/dreaming/learning.rs `in_scope_of`: the amendment applies where `event` was made. */
function appliesTo(amendment, event) {
  return inScope(amendment.scope, scopeOf(event));
}

/** Mirrors rust/src/dreaming/learning.rs `records_for`: the amendments one event's replay sees. */
function recordsFor(amendments, event) {
  return amendments.filter((amendment) => appliesTo(amendment, event)).map(retainedRecord);
}

/** Mirrors rust/src/dreaming/learning.rs `covered_events`. */
function coveredEvents(amendment, events, specificIndices, replay) {
  return (specificIndices.get(amendment.topic) ?? [])
    .filter((index) => appliesTo(amendment, events[index]))
    .filter((index) => amendmentReproducesSpecific(amendment, events[index], replay))
    .map((index) => events[index].id);
}

/** Mirrors rust/src/dreaming/learning.rs `build_amendment`. */
function buildAmendment(topic, scope, grouped, generalized) {
  const sources = [];
  for (const requirement of grouped) {
    for (const id of requirement.source_event_ids) if (!sources.includes(id)) sources.push(id);
  }
  return {
    id: amendmentId(topic, scope),
    topic,
    rule: boundedRule(grouped.map((requirement) => requirement.statement)),
    source_requirement_ids: sources,
    covered_event_ids: [],
    scope: scope.map(([key, value]) => [key, value]),
    generalized,
  };
}

/**
 * Mirrors rust/src/dreaming/learning.rs `generalized_statements`: the
 * statements of `topic` made in two or more distinct scopes, which are
 * candidates for one amendment that applies everywhere (issue #1559 §4).
 */
function generalizedStatements(grouped) {
  const scopesByStatement = new Map();
  for (const requirement of grouped) {
    const key = scopeKey(requirement.scope);
    if (!key) continue;
    const statement = requirement.statement.toLowerCase();
    if (!scopesByStatement.has(statement)) scopesByStatement.set(statement, new Set());
    scopesByStatement.get(statement).add(key);
  }
  return new Set([...scopesByStatement].filter(([, scopes]) => scopes.size >= 2).map(([statement]) => statement));
}

/**
 * Mirrors rust/src/dreaming/learning.rs `generalize_amendments`: one
 * amendment per topic and scope. A statement made in two or more scopes is
 * generalized into the amendment without scope, which is kept only when its
 * replay re-derives at least one stored task.
 */
function generalizeAmendments(events, requirements, specificIndices, replay) {
  const byTopic = new Map();
  for (const requirement of requirements) {
    if (!byTopic.has(requirement.topic)) byTopic.set(requirement.topic, []);
    byTopic.get(requirement.topic).push(requirement);
  }
  const amendments = [];
  for (const [topic, grouped] of sortedEntries(byTopic)) {
    const general = generalizedStatements(grouped);
    const unscoped = grouped.filter((requirement) => requirement.scope.length === 0);
    let generalizedAway = new Set();
    if (general.size > 0) {
      const lifted = [...unscoped];
      for (const requirement of grouped) {
        if (requirement.scope.length > 0 && general.has(requirement.statement.toLowerCase())
          && !lifted.some((entry) => entry.statement.toLowerCase() === requirement.statement.toLowerCase())) {
          lifted.push({ ...requirement, scope: [] });
        }
      }
      const candidate = buildAmendment(topic, [], lifted, true);
      for (const requirement of grouped) {
        if (requirement.scope.length > 0 && general.has(requirement.statement.toLowerCase())) {
          for (const id of requirement.source_event_ids) {
            if (!candidate.source_requirement_ids.includes(id)) candidate.source_requirement_ids.push(id);
          }
        }
      }
      candidate.covered_event_ids = coveredEvents(candidate, events, specificIndices, replay);
      if (candidate.covered_event_ids.length > 0) {
        amendments.push(candidate);
        generalizedAway = general;
      }
    }
    if (generalizedAway.size === 0 && unscoped.length > 0) amendments.push(buildAmendment(topic, [], unscoped, false));
    const byScope = new Map();
    for (const requirement of grouped) {
      const key = scopeKey(requirement.scope);
      if (!key || generalizedAway.has(requirement.statement.toLowerCase())) continue;
      if (!byScope.has(key)) byScope.set(key, []);
      byScope.get(key).push(requirement);
    }
    for (const [, scoped] of sortedEntries(byScope)) amendments.push(buildAmendment(topic, scoped[0].scope, scoped, false));
  }
  for (const amendment of amendments) {
    if (!amendment.generalized) amendment.covered_event_ids = coveredEvents(amendment, events, specificIndices, replay);
  }
  return amendments.filter((amendment) => amendment.rule);
}

/** Mirrors rust/src/dreaming/learning.rs `replay_candidate_tasks`. */
function replayCandidateTasks(events, observations, amendments, interactions, replay) {
  const candidates = [];
  events.forEach((event, index) => {
    if (!isTaskEvent(event)) return;
    const topic = observations[index].topic;
    if (topic === null || event.inputs === null || event.inputs === undefined
      || event.outputs === null || event.outputs === undefined) return;
    const simulated = replay(event.inputs, recordsFor(amendments, event));
    candidates.push({
      topic,
      source_event_id: event.id,
      input: event.inputs,
      expected_output: event.outputs,
      passed: normalize(simulated) === normalize(event.outputs),
      simulated_output: simulated,
    });
  });
  // Array.prototype.sort is stable, like Rust's `sort_by`.
  return candidates.sort((left, right) => cmpNum(interactions.get(right.topic) ?? 0, interactions.get(left.topic) ?? 0)
    || cmpStr(left.topic, right.topic)
    || cmpStr(left.source_event_id, right.source_event_id));
}

/**
 * The lowercase projection marker `learned standing requirement ({topic}):`,
 * read from the same wording the production path appends.
 */
function standingMarker(topic) {
  const template = serverMessage('standing_requirement_line', { rule: '' }).replace(/\s+$/u, '').toLowerCase();
  return template.replace('{topic}', () => topic);
}

/**
 * Mirrors rust/src/dreaming/learning.rs `missing_statements_for_topic`: the
 * marked statements of a stored output the current rule lacks, each inside
 * the bounds of a learned rule.
 */
function missingStatementsForTopic(expectedOutput, current, topic) {
  const currentRule = current ? current.rule.toLowerCase() : '';
  const marker = standingMarker(topic);
  const markerBytes = stringBytes(marker);
  const statements = [];
  for (const raw of rustStrLines(expectedOutput)) {
    const line = trim(raw);
    if (!line || !line.toLowerCase().startsWith(marker)) continue;
    const statement = trim(Buffer.from(line, 'utf8').subarray(markerBytes).toString('utf8'));
    if (statement && isBoundedRule(statement) && !currentRule.includes(statement.toLowerCase())
      && !statements.includes(statement)) {
      statements.push(statement);
    }
  }
  return statements;
}

/** Mirrors rust/src/dreaming/learning.rs `refine_amendments_from_failures`. */
function refineAmendmentsFromFailures(events, specificIndices, requirements, amendments, candidates, replay) {
  const eventById = new Map();
  for (const event of events) if (event.id && !eventById.has(event.id)) eventById.set(event.id, event);
  const sourceOf = (candidate) => eventById.get(candidate.source_event_id) ?? {};
  const refined = new Set();
  const failures = candidates.filter((candidate) => !candidate.passed && isValidTopic(candidate.topic))
    .map((candidate) => [candidate.topic, candidate.source_event_id, candidate.expected_output, sourceOf(candidate)]);
  for (const [topic, sourceEventId, expectedOutput, source] of failures) {
    const scope = scopePairs(source.evidence);
    const key = scopeKey(scope);
    let amendment = amendments.find((candidate) => candidate.topic === topic && scopeKey(candidate.scope) === key);
    const missing = missingStatementsForTopic(expectedOutput, amendment, topic);
    if (missing.length === 0) continue;
    if (!amendment) {
      amendment = {
        id: amendmentId(topic, scope), topic, rule: '', source_requirement_ids: [], covered_event_ids: [], scope, generalized: false,
      };
      amendments.push(amendment);
    }
    for (const statement of missing) {
      const next = amendment.rule ? `${amendment.rule}; ${statement}` : statement;
      if ([...next].length > ruleMaxChars()) continue;
      amendment.rule = next;
      requirements.push({ topic, statement, source_event_ids: [sourceEventId], occurrences: 1, scope });
    }
    if (sourceEventId && !amendment.source_requirement_ids.includes(sourceEventId)) {
      amendment.source_requirement_ids.push(sourceEventId);
    }
    refined.add(amendment);
  }
  if (refined.size === 0) return;
  for (const candidate of candidates.filter((entry) => !entry.passed)) {
    candidate.simulated_output = replay(candidate.input, recordsFor(amendments, sourceOf(candidate)));
    candidate.passed = normalize(candidate.simulated_output) === normalize(candidate.expected_output);
  }
  for (const amendment of amendments.filter((entry) => refined.has(entry))) {
    if (!specificIndices.has(amendment.topic)) continue;
    amendment.covered_event_ids = coveredEvents(amendment, events, specificIndices, replay);
  }
  // A refinement whose every statement fell outside the bounds teaches nothing.
  for (let index = amendments.length - 1; index >= 0; index -= 1) if (!amendments[index].rule) amendments.splice(index, 1);
}

/** Mirrors rust/src/dreaming/learning.rs `skeleton_tokens`. */
function skeletonTokens(input) {
  return splitWhitespace(input).map((token) => {
    const lowered = token.toLowerCase();
    if (!/[0-9]/.test(lowered)) return lowered;
    return lowered.replace(/[0-9]+/g, '#');
  });
}

/** Mirrors rust/src/dreaming/learning.rs `aligned_template`. */
function alignedTemplate(inputs) {
  const tokenized = inputs.map(skeletonTokens);
  const width = tokenized.reduce((max, tokens) => Math.max(max, tokens.length), 0);
  const template = [];
  for (let position = 0; position < width; position += 1) {
    let shared = null;
    let agree = true;
    for (const tokens of tokenized) {
      if (position >= tokens.length) {
        agree = false;
        break;
      }
      if (shared === null) shared = tokens[position];
      else if (shared !== tokens[position]) {
        agree = false;
        break;
      }
    }
    template.push(agree ? (shared ?? '*') : '*');
  }
  return template.join(' ');
}

/** Mirrors rust/src/dreaming/learning.rs `mine_patterns`. */
function minePatterns(events, observations) {
  const groups = new Map();
  events.forEach((event, index) => {
    if (!isTaskEvent(event)) return;
    const topic = observations[index].topic;
    if (topic === null || event.inputs === null || event.inputs === undefined) return;
    const head = splitWhitespace(event.inputs)[0];
    if (head === undefined) return;
    const key = [topic, head.toLowerCase()];
    const mapKey = JSON.stringify(key);
    if (!groups.has(mapKey)) groups.set(mapKey, { key, members: [] });
    groups.get(mapKey).members.push([event.id, event.inputs]);
  });
  return [...groups.values()]
    .sort((left, right) => cmpStr(left.key[0], right.key[0]) || cmpStr(left.key[1], right.key[1]))
    .filter(({ members }) => members.length >= 2)
    .map(({ key, members }) => ({
      topic: key[0],
      structure: alignedTemplate(members.map(([, input]) => input)),
      occurrences: members.length,
      source_event_ids: members.map(([id]) => id),
    }));
}

/** Mirrors rust/src/dreaming/learning.rs `advance_numbers` (`u64` parse, `+1`). */
function advanceNumbers(input) {
  return splitWhitespace(input).map((token) => {
    if (!/^\+?[0-9]+$/.test(token)) return token;
    const value = BigInt(token.replace(/^\+/, ''));
    return value > 0xffffffffffffffffn ? token : (value + 1n).toString();
  }).join(' ');
}

/** Mirrors rust/src/dreaming/learning.rs `synthesize_trials`. */
function synthesizeTrials(topics, patterns, events, amendments, replay) {
  const topTopics = new Set(topics.slice(0, 3).map((frequency) => frequency.topic));
  // `BTreeMap::collect` keeps the last input for a repeated id.
  const inputsById = new Map();
  for (const event of events) {
    if (event.inputs !== null && event.inputs !== undefined) inputsById.set(event.id, event.inputs);
  }
  const records = amendments.map(retainedRecord);
  const trials = [];
  for (const pattern of patterns) {
    if (!topTopics.has(pattern.topic) || !pattern.structure.includes('#')) continue;
    const exemplarId = pattern.source_event_ids.find((id) => inputsById.has(id));
    if (exemplarId === undefined) continue;
    const exemplar = inputsById.get(exemplarId);
    const synthesized = advanceNumbers(exemplar);
    if (synthesized === exemplar || pattern.source_event_ids
      .some((id) => inputsById.has(id) && inputsById.get(id) === synthesized)) continue;
    trials.push({
      topic: pattern.topic,
      structure: pattern.structure,
      input: synthesized,
      answer: replay(synthesized, records),
    });
  }
  return trials;
}

/** Mirrors rust/src/dreaming/learning.rs `learn_from_memory`. */
export function learnFromMemory(events, observations, deletedConversations, replay) {
  const interactions = new Map();
  const taskCounts = new Map();
  const requirementCounts = new Map();
  const requirementsByKey = new Map();
  const specificIndices = new Map();
  const bump = (map, key) => map.set(key, (map.get(key) ?? 0) + 1);
  events.forEach((event, index) => {
    if (event.conversation_id !== null && event.conversation_id !== undefined
      && deletedConversations.has(event.conversation_id)) return;
    const topic = observations[index].topic;
    if (topic === null) return;
    bump(interactions, topic);
    if (isTaskEvent(event)) {
      bump(taskCounts, topic);
      if (event.id) {
        if (!specificIndices.has(topic)) specificIndices.set(topic, []);
        specificIndices.get(topic).push(index);
      }
    }
    const statement = requirementStatement(event);
    const learnedTopic = statement === null ? null : requirementTopic(event, statement);
    if (learnedTopic !== null) {
      bump(requirementCounts, topic);
      const scope = scopePairs(event.evidence);
      const key = [learnedTopic, scopeKey(scope), statement.toLowerCase()];
      const mapKey = JSON.stringify(key);
      if (!requirementsByKey.has(mapKey)) {
        requirementsByKey.set(mapKey, {
          key, requirement: { topic: learnedTopic, statement, source_event_ids: [], occurrences: 0, scope },
        });
      }
      const entry = requirementsByKey.get(mapKey).requirement;
      entry.occurrences += 1;
      if (event.id) entry.source_event_ids.push(event.id);
    }
  });
  const topics = sortedEntries(interactions).map(([topic, count]) => ({
    topic,
    interactions: count,
    task_events: taskCounts.get(topic) ?? 0,
    requirement_events: requirementCounts.get(topic) ?? 0,
  })).sort((left, right) => cmpNum(right.interactions, left.interactions) || cmpStr(left.topic, right.topic));
  const requirements = [...requirementsByKey.values()]
    .sort((left, right) => cmpStr(left.key[0], right.key[0]) || cmpStr(left.key[1], right.key[1])
      || cmpStr(left.key[2], right.key[2]))
    .map(({ requirement }) => requirement);
  const amendments = generalizeAmendments(events, requirements, specificIndices, replay);
  const candidateTasks = replayCandidateTasks(events, observations, amendments, interactions, replay);
  refineAmendmentsFromFailures(events, specificIndices, requirements, amendments, candidateTasks, replay);
  const patterns = minePatterns(events, observations);
  // A synthesized trial belongs to no one request, so it sees only amendments without scope.
  const unscoped = amendments.filter((amendment) => amendment.scope.length === 0);
  const synthesizedTasks = synthesizeTrials(topics, patterns, events, unscoped, replay);
  return {
    topics,
    requirements,
    amendments,
    candidate_tasks: candidateTasks,
    patterns,
    synthesized_tasks: synthesizedTasks,
  };
}
