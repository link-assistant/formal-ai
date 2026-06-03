// Issue #386: mirror the Rust conversation-summary conversion in the
// hand-written JS worker. (1) Rename the misnamed-but-generic
// `calendarWordsForRole` to `wordsForRole` (it is already used for non-calendar
// roles, e.g. ROLE_SOFTWARE_REQUIREMENT_CATEGORY, and now mirrors Rust's
// Lexicon::words_for_role by name too). (2) Declare the four new
// ROLE_CONVERSATION_* consts. (3) Rewrite isSummarizePrompt to compose those
// meaning roles instead of hardcoding per-language phrase/regex arrays, plus a
// summaryDirectiveLeads helper. (4) Simplify the single call site to the
// single-arg signature and refresh two now-stale comments. Line-splice anchored
// on ASCII signatures so the mixed Cyrillic/Devanagari/Han body lines never
// need literal matching (the Edit tool can fail on such lines).
import { readFileSync, writeFileSync } from 'node:fs';

const path = 'src/web/formal_ai_worker.js';
let lines = readFileSync(path, 'utf8').split('\n');

// Replace a top-level `<signature>` line through its first column-zero `}`.
function replaceFn(signature, replacement) {
  const start = lines.findIndex((l) => l === signature);
  if (start < 0) throw new Error('signature not found: ' + signature);
  let end = -1;
  for (let i = start + 1; i < lines.length; i++) {
    if (lines[i] === '}') {
      end = i;
      break;
    }
  }
  if (end < 0) throw new Error('closing brace not found for: ' + signature);
  lines = [...lines.slice(0, start), ...replacement, ...lines.slice(end + 1)];
}

// Replace the inclusive span [firstExact .. lastExact] with `replacement`.
function replaceSpan(firstExact, lastExact, replacement) {
  const a = lines.findIndex((l) => l === firstExact);
  if (a < 0) throw new Error('first line not found: ' + firstExact);
  const b = lines.findIndex((l, idx) => idx > a && l === lastExact);
  if (b < 0) throw new Error('last line not found: ' + lastExact);
  lines = [...lines.slice(0, a), ...replacement, ...lines.slice(b + 1)];
}

// (1) Rename calendarWordsForRole -> wordsForRole everywhere (def + 7 calls).
lines = lines.map((l) => l.split('calendarWordsForRole').join('wordsForRole'));

// (2) Declare the four conversation-summary roles after the known-facts phrase
// role const (the last role const added for issue #386 self-awareness work).
const anchor = 'const ROLE_KNOWLEDGE_INVENTORY_PHRASE = "knowledge_inventory_phrase";';
const ai = lines.findIndex((l) => l === anchor);
if (ai < 0) throw new Error('ROLE_KNOWLEDGE_INVENTORY_PHRASE const not found');
const roleBlock = [
  '',
  '// Issue #386 conversation-summary roles — mirror the',
  '// ROLE_CONVERSATION_SUMMARY_DIRECTIVE / ROLE_CONVERSATION_REFERENCE /',
  '// ROLE_CONVERSATION_SUMMARY_PHRASE / ROLE_CONVERSATION_SUMMARY_COURTESY consts',
  '// in src/seed/roles.rs. Their per-language surface words live once in the',
  '// embedded MEANINGS_LINO above (data/seed/meanings-intent.lino); the',
  '// isSummarizePrompt recogniser composes these roles instead of hardcoding',
  '// per-language phrase / regex arrays.',
  'const ROLE_CONVERSATION_SUMMARY_DIRECTIVE = "conversation_summary_directive";',
  'const ROLE_CONVERSATION_REFERENCE = "conversation_reference";',
  'const ROLE_CONVERSATION_SUMMARY_PHRASE = "conversation_summary_phrase";',
  'const ROLE_CONVERSATION_SUMMARY_COURTESY = "conversation_summary_courtesy";',
];
lines = [...lines.slice(0, ai + 1), ...roleBlock, ...lines.slice(ai + 1)];

// (3) Rewrite isSummarizePrompt + add summaryDirectiveLeads. Anchor the replace
// on the old comment line so the stale comment block is replaced too.
const summarizeBlock = [
  '// Issue #386: a conversation-summary request is recognised by composing',
  '// meaning roles, not by matching raw words per language. The universal',
  '// algorithm is identical for every language: the prompt either carries a',
  '// complete standalone conversation-summary phrasing, an objectless courtesy',
  '// frame ("can you summarize", "подведи итог"), a summary directive together',
  '// with an explicit conversation reference, or it is itself a bare summary',
  '// directive. The prompt is re-normalised first so the boundary-aware matcher',
  '// sees punctuation collapsed to spaces (idempotent here, since `normalized`',
  '// is already normalised). Mirror of asks_for_conversation_summary in',
  '// src/solver_handlers/mod.rs.',
  'function isSummarizePrompt(normalized) {',
  '  const cleaned = normalizePrompt(normalized);',
  '  return (',
  '    lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_PHRASE, cleaned) ||',
  '    lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_COURTESY, cleaned) ||',
  '    (lexiconMentionsRole(ROLE_CONVERSATION_SUMMARY_DIRECTIVE, cleaned) &&',
  '      lexiconMentionsRole(ROLE_CONVERSATION_REFERENCE, cleaned)) ||',
  '    summaryDirectiveLeads(cleaned)',
  '  );',
  '}',
  '',
  '// A bare summary directive standing alone is itself a request to summarize the',
  '// running conversation ("summarize", "резюме", "总结", ...). For whitespace-',
  '// delimited scripts the directive must be the whole prompt; for CJK (no word',
  '// spaces) a leading directive suffices, mirroring the historical `^总结`',
  '// anchor and keeping compounds like "工作总结" (a work summary) out. Mirror of',
  '// summary_directive_leads in src/solver_handlers/mod.rs.',
  'function summaryDirectiveLeads(cleaned) {',
  '  return wordsForRole(ROLE_CONVERSATION_SUMMARY_DIRECTIVE).some((word) =>',
  '    containsCjk(word) ? cleaned.startsWith(word) : cleaned === word,',
  '  );',
  '}',
];
replaceFn(
  '// Issue #27: trigger the summarize skill on a wide range of natural phrasings',
  summarizeBlock,
);

// (4) Simplify the single call site to the new signature and refresh the two
// now-stale comment lines above it (normalizePrompt preserves non-Latin scripts;
// it does not strip them to an empty string).
replaceSpan(
  '  // Issue #27: summarize triggers can be in non-Latin scripts that normalize',
  '  if (isSummarizePrompt(prompt, normalized)) {',
  [
    '  // Issue #386: a conversation-summary request is recognised by composing',
    '  // meaning roles (see isSummarizePrompt) across every supported language, so',
    '  // we test it before the empty-normalized bail-out below.',
    '  if (isSummarizePrompt(normalized)) {',
  ],
);

writeFileSync(path, lines.join('\n'));
console.log(
  'worker: renamed calendarWordsForRole->wordsForRole, added 4 ROLE_CONVERSATION_ consts, rewrote isSummarizePrompt + summaryDirectiveLeads, simplified call site',
);
