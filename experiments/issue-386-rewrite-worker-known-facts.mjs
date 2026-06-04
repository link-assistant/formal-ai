// Issue #386: mirror the Rust known-facts conversion in the hand-written JS
// worker. Replaces isKnownFactQuery + selfAwarenessLanguage by role-query
// versions and declares the four new ROLE_* consts. Line-splice anchored on
// ASCII signatures so the mixed Cyrillic/Devanagari/Han body lines never need
// literal matching (the Edit tool can fail on such lines). containsAny stays —
// it is a shared util used ~30 times elsewhere in the worker.
import { readFileSync, writeFileSync } from 'node:fs';

const path = 'src/web/formal_ai_worker.js';
let lines = readFileSync(path, 'utf8').split('\n');

// Replace a top-level `function name(...) {` through its first column-zero `}`.
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

const knownFacts = [
  '// Issue #386: a known-facts inventory query is recognised by composing meaning',
  '// roles, not by matching raw words per language. The universal algorithm is',
  '// identical for every language: the prompt either names the knowledge `fact`',
  '// noun together with an enumerating interrogative and a second-person',
  '// attribution of knowing, or it matches one of the complete standalone',
  '// phrasings that ask what the assistant knows even without the noun. The',
  '// prompt is re-normalised first so the boundary-aware matcher sees punctuation',
  '// collapsed to spaces. Mirror of is_known_fact_query in',
  '// src/solver_handlers/self_awareness.rs.',
  'function isKnownFactQuery(normalized) {',
  '  if (isSelfFactQuery(normalized)) return false;',
  '  const cleaned = normalizePrompt(normalized);',
  '  const composed =',
  '    lexiconMentionsRole(ROLE_KNOWLEDGE_INVENTORY_NOUN, cleaned) &&',
  '    lexiconMentionsRole(ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE, cleaned) &&',
  '    lexiconMentionsRole(ROLE_KNOWLEDGE_POSSESSION, cleaned);',
  '  return (',
  '    composed || lexiconMentionsRole(ROLE_KNOWLEDGE_INVENTORY_PHRASE, cleaned)',
  '  );',
  '}',
];

const language = [
  'function selfAwarenessLanguage(prompt, normalized) {',
  '  // Issue #386: language is detected purely by Unicode script ranges. The',
  '  // Cyrillic range below already subsumes the former second-person pronoun',
  '  // list (ty/tebya/tvoy/vy/...), every member of which is Cyrillic, so no raw',
  '  // word list is needed -- the script range is the universal signal. Mirror of',
  '  // self_awareness_language in src/solver_handlers/self_awareness.rs.',
  '  const text = `${String(prompt || "").toLowerCase()} ${String(normalized || "")}`;',
  '  if (/[\\u0400-\\u04ff]/u.test(text)) return "ru";',
  '  if (/[\\u0900-\\u097f]/u.test(text)) return "hi";',
  '  if (/[\\u4e00-\\u9fff]/u.test(text)) return "zh";',
  '  return detectLanguage(prompt);',
  '}',
];

// Replace the two recognizers (order independent — each re-finds its anchor).
replaceFn('function isKnownFactQuery(normalized) {', knownFacts);
replaceFn('function selfAwarenessLanguage(prompt, normalized) {', language);

// Declare the four new ROLE_* consts right after ROLE_SELF_INTRODUCTION_REQUEST.
const anchor = 'const ROLE_SELF_INTRODUCTION_REQUEST = "self_introduction_request";';
const ai = lines.findIndex((l) => l === anchor);
if (ai < 0) throw new Error('ROLE_SELF_INTRODUCTION_REQUEST const not found');
const roleBlock = [
  '// Issue #386 known-facts inventory roles — mirror the ROLE_KNOWLEDGE_INVENTORY_*',
  '// / ROLE_KNOWLEDGE_POSSESSION consts in src/seed/roles.rs. Their surface words',
  '// live in data/seed/meanings-intent.lino (the shared `fact` noun plus the',
  '// knowledge_inventory_probe / assistant_knowing / knowledge_inventory_query',
  '// meanings, embedded in MEANINGS_LINO above); isKnownFactQuery composes these',
  '// roles instead of hardcoding per-language phrase arrays.',
  'const ROLE_KNOWLEDGE_INVENTORY_NOUN = "knowledge_inventory_noun";',
  'const ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE = "knowledge_inventory_interrogative";',
  'const ROLE_KNOWLEDGE_POSSESSION = "knowledge_possession";',
  'const ROLE_KNOWLEDGE_INVENTORY_PHRASE = "knowledge_inventory_phrase";',
];
lines = [...lines.slice(0, ai + 1), ...roleBlock, ...lines.slice(ai + 1)];

writeFileSync(path, lines.join('\n'));
console.log('worker: rewrote isKnownFactQuery + selfAwarenessLanguage, added 4 ROLE_ consts');
