// The scoped workspace rewrite (rust/src/workspace_change_learning.rs
// `execute_scoped_workspace_rewrite`, `word_scoped_matches`,
// `is_identifier_word`). The recipe-learning ledger is not ported.

import { isAlphanumeric, matchIndices, trimMatches } from '../write_str.mjs';
import { stableId } from './engine_stable_id.mjs';
import { executeRewrite, rewriteProgram, rewriteRule } from './normal_markov.mjs';

const MAX_REWRITE_STEPS = 100000;
const REWRITE_FRAME = '\u0000';

/** Mirrors `RewriteScope`. */
export const RewriteScope = Object.freeze({ Substring: 'substring', Word: 'word' });

const isIdentifierCharacter = (character) => character !== undefined && (isAlphanumeric(character) || character === '_');
const charBefore = (text, at) => Array.from(text.slice(Math.max(0, at - 2), at)).pop();
const charAfter = (text, at) => Array.from(text.slice(at, at + 2))[0];

/** Mirrors `fn word_scoped_matches`: string indices of whole-word matches. */
export function wordScopedMatches(source, pattern) {
  return matchIndices(source, pattern).filter((at) =>
    !isIdentifierCharacter(charBefore(source, at)) && !isIdentifierCharacter(charAfter(source, at + pattern.length)));
}

/** Mirrors `fn is_identifier_word`. */
export function isIdentifierWord(text) {
  return text !== '' && Array.from(text).every(isIdentifierCharacter);
}

function wordScopedRules(framed, pattern, replacement) {
  const rules = [];
  for (const at of wordScopedMatches(framed, pattern)) {
    const before = charBefore(framed, at) ?? REWRITE_FRAME;
    const after = charAfter(framed, at + pattern.length) ?? REWRITE_FRAME;
    const rule = rewriteRule(`${before}${pattern}${after}`, `${before}${replacement}${after}`);
    if (!rules.some((seen) => seen.pattern === rule.pattern && seen.replacement === rule.replacement)) rules.push(rule);
  }
  return rules;
}

/**
 * Mirrors `fn execute_scoped_workspace_rewrite`:
 * `{ok: {pattern, replacement, output, steps, source_fingerprint}}` or `{error: <slug>}`.
 */
export function executeScopedWorkspaceRewrite(source, pattern, replacement, scope) {
  if (pattern === '' || pattern === replacement) return { error: 'workspace_rewrite_operands_unsafe' };
  let rules;
  let subject;
  if (scope === RewriteScope.Substring) {
    if (replacement.includes(pattern)) return { error: 'workspace_rewrite_operands_unsafe' };
    rules = [rewriteRule(pattern, replacement)];
    subject = source;
  } else {
    if (!isIdentifierWord(pattern) || !isIdentifierWord(replacement)) return { error: 'workspace_rewrite_operands_unsafe' };
    if (source.includes(REWRITE_FRAME)) return { error: 'workspace_rewrite_frame_conflict' };
    subject = `${REWRITE_FRAME}${source}${REWRITE_FRAME}`;
    rules = wordScopedRules(subject, pattern, replacement);
  }
  if (!rules.length) return { error: 'workspace_rewrite_no_match' };
  const outcome = executeRewrite(rewriteProgram(rules, MAX_REWRITE_STEPS), subject);
  if (outcome.halt.kind === 'step_limit') return { error: 'workspace_rewrite_step_limit' };
  const output = scope === RewriteScope.Substring ? outcome.output : trimMatches(outcome.output, (character) => character === REWRITE_FRAME);
  if (!outcome.trace.length || output === source) return { error: 'workspace_rewrite_no_match' };
  return {
    ok: {
      pattern, replacement, output, steps: outcome.trace.length, source_fingerprint: stableId('workspace_rewrite_source', source),
    },
  };
}
