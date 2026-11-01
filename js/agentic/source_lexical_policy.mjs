// Shared protocol grammar data, independent of task or component names.
import policy from '../../data/meta/source-lexical-rules.json' with { type: 'json' };
export function sourceLexicalRule(text, name) {
  if (typeof text !== 'string' || new TextEncoder().encode(text).length > 262144 || typeof name !== 'string' || policy.schema !== 'source-lexical-rules/v1' || !Object.hasOwn(policy.rules, name)) return null;
  const rule = policy.rules[name],
    patterns = rule.patterns;
  if (!['contains', 'prefix'].includes(rule.operator) || !Array.isArray(patterns) || !patterns.length || patterns.length > 8 || patterns.some(value => typeof value !== 'string' || !value.length || value.length > 64)) return null;
  return patterns.some(value => rule.operator === 'contains' ? text.includes(value) : text.startsWith(value));
}
