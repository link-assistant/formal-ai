// Mining the durable `draft_failure` records a candidate-solution portfolio
// leaves behind: rust/src/dreaming/draft_failures.rs. The lesson is a
// language-neutral slug, so nothing user-facing is minted here.

import { cmpNum, cmpStr, rustStrLines, trim } from './dreaming-support.mjs';

/** rust/src/draft_portfolio.rs `MAX_ATTEMPTS`. */
const MAX_ATTEMPTS = 3;

/** Mirrors rust/src/dreaming/draft_failures.rs `field`. */
function field(record, key) {
  for (const line of rustStrLines(record)) {
    const trimmed = trim(line);
    if (!trimmed.startsWith(key)) continue;
    const rest = trimmed.slice(key.length);
    if (!rest.startsWith(' ')) continue;
    return trim(rest.slice(1)).replace(/^"+|"+$/g, '');
  }
  return null;
}

/** Mirrors rust/src/dreaming/draft_failures.rs `is_draft_failure`. */
function isDraftFailure(event) {
  if (event.content === null || event.content === undefined) return null;
  const tagged = field(event.content, 'record_type') === 'draft_failure';
  return tagged || event.kind === 'draft_failure' ? event.content : null;
}

/** Rust `str::parse::<usize>()` / `parse::<u32>()`. */
function parseUnsigned(value, max) {
  if (value === null || !/^\+?\d+$/.test(value)) return null;
  const parsed = BigInt(value.replace(/^\+/, ''));
  return parsed > max ? null : Number(parsed);
}

const U32_MAX = 0xffffffffn;
const USIZE_MAX = 0xffffffffffffffffn;

/** Mirrors rust/src/dreaming/draft_failures.rs `conclusion`. */
function conclusion(lesson) {
  if (lesson.passed_tests > 0 && lesson.passed_tests < lesson.total_tests) return 'extend_strategy';
  if (lesson.exhausted_retry_budget) return 'deprioritize_strategy';
  return 'raise_draft_count';
}

/** Mirrors rust/src/dreaming/draft_failures.rs `draft_failure_lessons`. */
export function draftFailureLessons(events) {
  const byStrategy = new Map();
  for (const event of events) {
    const record = isDraftFailure(event);
    if (record === null) continue;
    const strategy = field(record, 'strategy') ?? 'unknown';
    const passed = parseUnsigned(field(record, 'passed_tests'), USIZE_MAX) ?? 0;
    const total = parseUnsigned(field(record, 'total_tests'), USIZE_MAX) ?? 0;
    const attempt = parseUnsigned(field(record, 'attempt'), U32_MAX) ?? 0;
    const maxAttempts = parseUnsigned(field(record, 'max_attempts'), U32_MAX) ?? MAX_ATTEMPTS;
    if (!byStrategy.has(strategy)) {
      byStrategy.set(strategy, {
        strategy,
        occurrences: 0,
        passed_tests: 0,
        total_tests: 0,
        attempts: 0,
        exhausted_retry_budget: true,
        lesson: '',
        source_event_ids: [],
      });
    }
    const entry = byStrategy.get(strategy);
    entry.occurrences += 1;
    entry.passed_tests += passed;
    entry.total_tests += total;
    entry.attempts = Math.min(entry.attempts + attempt, Number(U32_MAX));
    entry.exhausted_retry_budget = entry.exhausted_retry_budget && attempt >= maxAttempts;
    if (event.id) entry.source_event_ids.push(event.id);
  }
  const lessons = [...byStrategy.values()].sort((left, right) => cmpStr(left.strategy, right.strategy));
  for (const lesson of lessons) lesson.lesson = conclusion(lesson);
  return lessons.sort((left, right) => cmpNum(right.occurrences, left.occurrences)
    || cmpStr(left.strategy, right.strategy));
}
