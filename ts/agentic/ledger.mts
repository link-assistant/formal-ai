import { cached, childrenNamed, host, parseLino, childValue } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { quote } from './crate/change_request.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** The top-level `name` node (the shared parser returns a lone top node itself). */
const topNode = (parsed, name) => (parsed?.name === name ? parsed : childrenNamed(parsed, name)[0]);

/** Mirrors `const LEDGER_PATH`. */
export const LEDGER_PATH = 'learning-ledger.lino';

/** Mirrors `const LEDGER_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const ledgerKeywords = () => cached('ledger-keywords', () => agenticMessage('ledger_keywords').split('|'));

/**
 * Mirrors `fn is_ledger_task` in rust/src/agentic_coding/ledger.rs.
 * @param {string} prompt
 */
export function isLedgerTask(prompt) {
  const lower = prompt.toLowerCase();
  return ledgerKeywords().some((keyword) => lower.includes(keyword))
    || (lower.includes('promote') && (lower.includes('lesson') || lower.includes('learning')) && lower.includes('ledger'));
}

/** Project the operator's approved lesson records into the reviewable ledger. */
export function renderDocument() {
  const text = host().readReviewDocument?.('approved_lessons') ?? null;
  const root = text === null ? null : topNode(parseLino(text), 'approved_lessons');
  if (text !== null && !root) throw Error('InvalidApprovedLessons');
  const lessons = root ? childrenNamed(root, 'lesson') : [];
  const keys = ['case_id', 'failure_prompt', 'module_path', 'rule_id', 'modifier', 'resolved_task', 'benchmark_suite', 'benchmark_passed', 'reviewer'];
  const identifiers = new Set();
  const field = (key, value) => `    ${key} "${quote(value)}"\n`;
  let document = `learning_ledger\n  engine "meta_language"\n  human_gated "true"\n  lesson_count "${lessons.length}"\n`;
  for (const lesson of lessons) {
    const identifier = lesson.id || lesson.value;
    const values = keys.map(key => childValue(lesson, key)?.trim());
    const passed = Number(values[7]);
    if (!identifier || identifiers.has(identifier) || values.some(value => !value) || !/^\d+$/u.test(values[7]) || !Number.isSafeInteger(passed)) throw Error('InvalidApprovedLessons');
    identifiers.add(identifier);
    values[7] = String(passed);
    document += '  lesson\n' + field('lesson_id', identifier);
    keys.forEach((key, index) => { document += field(key, values[index]); });
  }
  return document;
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/ledger.rs (`LearningLedger::len`). */
export function finalAnswer(document) {
  const ledger = topNode(parseLino(document), 'learning_ledger');
  return agenticMessage('ledger_final_answer', {
    count: childrenNamed(ledger, 'lesson').length,
    path: LEDGER_PATH,
    document: trimEnd(document),
  });
}
