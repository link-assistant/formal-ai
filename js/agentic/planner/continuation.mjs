// Continuation and compaction helpers for the agentic planner: a port of
// rust/src/agentic_coding/planner/continuation.rs.

import { plainText } from '../content.mjs';
import { normalizePrompt } from '../crate/engine.mjs';
import { mentionsRole } from '../write_lexicon.mjs';
import { proseSentences } from '../shell_command_policy.mjs';
import { agentInfoValue } from '../crate/seed_agent_info.mjs';
import { handlerMatches } from '../crate/rule_interpreter.mjs';
import {
  isAlphanumeric, isWhitespace, rsplitOnce, splitOnce, stripPrefix, trim, trimEndMatches, trimMatches, trimStart,
} from '../crate/rust_str.mjs';

const isRole = (message, role) => message.role.toLowerCase() === role;

/** Mirrors `fn is_continuation_cue`. */
export function isContinuationCue(text) {
  return handlerMatches('agentic_continuation', text);
}

/** Mirrors `fn evidence_window_start`. */
export function evidenceWindowStart(messages) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (isRole(message, 'user') && !isContinuationCue(plainText(message.content))) return index + 1;
  }
  return 0;
}

/** Mirrors `fn continued_agent_task`. */
export function continuedAgentTask(messages, latest) {
  if (!isContinuationCue(latest)) return null;
  let latestUser = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (isRole(messages[index], 'user')) {
      latestUser = index;
      break;
    }
  }
  if (latestUser < 0) return null;
  const earlier = messages.slice(0, latestUser);
  const compacted = compactedAgentTask(earlier);
  if (compacted !== null) return compacted;
  for (let index = earlier.length - 1; index >= 0; index -= 1) {
    if (!isRole(earlier[index], 'user')) continue;
    const text = plainText(earlier[index].content);
    if (trim(text) && !isContinuationCue(text)) return text;
  }
  return null;
}

/**
 * Mirrors `fn compacted_agent_task`. Agent may compact an already compacted
 * conversation, and each compaction summarizes the previous summary turn, so
 * envelopes nest; every `Conversation summary:` envelope is read, the latest
 * first, until one names a standing task (PR #1188 dogfooding: the second
 * compaction listed "What did we do so far?" first and the task was lost).
 */
export function compactedAgentTask(earlier) {
  for (let index = earlier.length - 1; index >= 0; index -= 1) {
    const message = earlier[index];
    if (!isRole(message, 'assistant')) continue;
    const envelopes = plainText(message.content).split('Conversation summary:').slice(1).reverse();
    for (const envelope of envelopes) {
      const summary = trimStart(envelope);
      let task = preservedFirstUserTurn(summary);
      if (task === null) {
        const titled = splitOnce(summary, '\n\nTitle:');
        task = titled ? standingSentences(trim(titled[0])) : null;
      }
      if (task !== null) return repairCompactedDotPaths(task);
    }
  }
  return null;
}

/**
 * Mirrors `fn standing_sentences`: the summary head's sentences that state
 * work, joined — a re-summarized head trails the client's own residue
 * (`… What did we do so far? Title: … User turns:.`) after the task.
 */
function standingSentences(head) {
  const kept = proseSentences(head)
    .filter((sentence) => isStandingTask(sentence.text)
      && !mentionsEnvelopeMarker(sentence.text, 'compaction_title_marker')
      && !mentionsEnvelopeMarker(sentence.text, 'compaction_turns_marker'))
    .map((sentence) => trim(head.slice(sentence.span.start, sentence.span.end)));
  return kept.length ? kept.join(' ') : null;
}

/**
 * Mirrors `fn is_standing_task`: a turn that states work, not the client's
 * own protocol — a continuation cue, a seeded request to summarize the
 * conversation, or a nested summary envelope.
 */
export function isStandingTask(text) {
  return !isContinuationCue(text)
    && !mentionsRole('conversation_summary_phrase', normalizePrompt(text))
    && !mentionsEnvelopeMarker(text, 'compaction_summary_marker');
}

/** Mirrors `fn mentions_envelope_marker`: a compaction envelope label from data/seed/agent-info.lino. */
function mentionsEnvelopeMarker(text, key) {
  const marker = agentInfoValue(key);
  return Boolean(marker) && text.includes(marker);
}

/** Mirrors `fn safe_relative_path` in rust/src/agentic_coding/write_request.rs. */
function safeRelativePath(path) {
  return !path.startsWith('/')
    && !path.startsWith('-')
    && !path.split('/').some((part) => part === '..' || part === '')
    && Array.from(path).every((character) => isAlphanumeric(character) || '/._-'.includes(character));
}

/** Mirrors `fn repair_compacted_dot_paths`. */
export function repairCompactedDotPaths(original) {
  let task = original;
  let searchFrom = 0;
  for (;;) {
    const dot = task.indexOf('. ', searchFrom);
    if (dot < 0) break;
    const previous = Array.from(task.slice(0, dot)).pop();
    const standalone = previous === undefined || isWhitespace(previous);
    const afterDot = dot + 1;
    let pathStart = -1;
    for (let index = afterDot; index < task.length; index += 1) {
      if (!isWhitespace(task[index])) {
        pathStart = index;
        break;
      }
    }
    if (pathStart < 0) break;
    let pathEnd = task.length;
    for (let index = pathStart; index < task.length; index += 1) {
      if (isWhitespace(task[index])) {
        pathEnd = index;
        break;
      }
    }
    const token = trimEndMatches(
      trimMatches(task.slice(pathStart, pathEnd), (character) => '`"\',;'.includes(character)),
      (character) => '.!?'.includes(character),
    );
    if (standalone && token.includes('/') && safeRelativePath(`.${token}`)) {
      task = task.slice(0, afterDot) + task.slice(pathStart);
      searchFrom = dot + 1;
    } else {
      searchFrom = pathStart;
    }
  }
  return task;
}

/** Mirrors `fn preserved_first_user_turn`. */
export function preservedFirstUserTurn(summary) {
  const split = splitOnce(summary, '\n\nUser turns:\n');
  if (!split) return null;
  let rest = split[1];
  for (let number = 1; ; number += 1) {
    const body = stripPrefix(rest, `  ${number}. `);
    if (body === null) return null;
    const next = body.indexOf(`\n  ${number + 1}.`);
    const end = next < 0 ? body.length : next;
    const task = trim(body.slice(0, end));
    if (task && isStandingTask(task)) return task;
    if (next < 0) return null;
    rest = body.slice(end + 1);
  }
}

/**
 * Mirrors `fn trace_route`. The Rust twin writes to stderr when the
 * `FORMAL_AI_TRACE_REQUESTS` environment flag is set; the planner host has no
 * environment (the browser worker must load this module), so this is a no-op.
 */
export function traceRoute(route, value) {
  void route;
  void value;
}
