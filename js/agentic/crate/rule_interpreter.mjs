// `handler_matches` from rust/src/rule_interpreter.rs.
//
// The worker realm already interprets data/seed/handler-rules.lino
// (js/worker/formal_ai_worker_handler_rules.js: the parsed document,
// `Condition::holds` and `resolve_values` twins over the same meaning
// surfaces), so this reuses those and supplies the context Rust's
// `matches_with_source` builds: `normalized` is the prompt lowercased, and a
// rule matches when its condition holds and its values resolve.

import { realm } from '../host.mjs';
import { normalizePrompt } from './engine.mjs';
import { detect } from './language.mjs';
import { trim } from './rust_str.mjs';

/**
 * Mirrors `fn handler_matches` in rust/src/rule_interpreter.rs.
 * @param {string} name
 * @param {string} prompt
 * @returns {boolean}
 */
export function handlerMatches(name, prompt) {
  const scope = realm();
  const rules = scope.handlerRulesDocument()[name];
  if (!Array.isArray(rules)) return false;
  const source = String(prompt);
  const normalized = source.toLowerCase();
  const language = detect(source);
  const context = {
    prompt: source,
    language,
    languages: language === 'en' ? ['en'] : [language, 'en'],
    history: [],
    subjects: {
      normalized,
      cleaned: normalizePrompt(normalized),
      lowercase: source.toLowerCase(),
      prompt: source,
      trimmed: trim(normalized),
      padded: ` ${normalized} `,
    },
  };
  return rules.some((rule) =>
    scope.handlerRulesHolds(rule.when, context) && scope.handlerRulesResolveValues(rule, context) !== null);
}
