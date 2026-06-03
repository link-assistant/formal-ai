// Issue #386: rewrite the self_awareness known-facts recognizer to query
// meaning roles instead of hardcoded per-language word lists. Operates by
// line-splice anchored on ASCII function signatures so the mixed
// Cyrillic/Devanagari/Han body lines never need to be matched literally
// (the Edit tool can fail to match such lines). Each replacement re-finds its
// anchor on the mutated array; processed bottom-to-top so indices stay valid.
import { readFileSync, writeFileSync } from 'node:fs';

const path = 'src/solver_handlers/self_awareness.rs';
let lines = readFileSync(path, 'utf8').split('\n');

// Replace a whole top-level fn: from its signature line to the first
// column-zero `}` that closes it (body braces are always indented).
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
  'fn is_known_fact_query(normalized: &str) -> bool {',
  '    if is_self_fact_query(normalized) {',
  '        return false;',
  '    }',
  '',
  '    // Issue #386: a known-facts inventory query is recognised by composing',
  '    // meaning roles, not by matching raw words per language. The universal',
  '    // algorithm is identical for every language: the prompt either names the',
  '    // knowledge `fact` noun together with an enumerating interrogative and a',
  '    // second-person attribution of knowing, or it matches one of the complete',
  '    // standalone phrasings that ask what the assistant knows even without the',
  '    // noun. The prompt is re-normalised first so the boundary-aware matcher',
  '    // sees punctuation collapsed to spaces.',
  '    let cleaned = normalize_prompt(normalized);',
  '    let lexicon = seed::lexicon();',
  '    let composed = lexicon.mentions_role(seed::ROLE_KNOWLEDGE_INVENTORY_NOUN, &cleaned)',
  '        && lexicon.mentions_role(seed::ROLE_KNOWLEDGE_INVENTORY_INTERROGATIVE, &cleaned)',
  '        && lexicon.mentions_role(seed::ROLE_KNOWLEDGE_POSSESSION, &cleaned);',
  '',
  '    composed || lexicon.mentions_role(seed::ROLE_KNOWLEDGE_INVENTORY_PHRASE, &cleaned)',
  '}',
];

const language = [
  'fn self_awareness_language(prompt: &str, normalized: &str) -> &\'static str {',
  '    // Issue #386: language is detected purely by Unicode script ranges. The',
  '    // Cyrillic range below already subsumes the former second-person pronoun',
  '    // list (ty/tebya/tvoy/vy/...), every member of which is Cyrillic, so no raw',
  '    // word list is needed -- the script range is the universal signal.',
  '    let lower = format!("{} {}", prompt.to_lowercase(), normalized);',
  "    if has_char_in_range(&lower, '\\u{0400}', '\\u{04ff}') {",
  '        return "ru";',
  '    }',
  "    if has_char_in_range(&lower, '\\u{0900}', '\\u{097f}') {",
  '        return "hi";',
  '    }',
  "    if has_char_in_range(&lower, '\\u{4e00}', '\\u{9fff}') {",
  '        return "zh";',
  '    }',
  '    detect_language(prompt).slug()',
  '}',
];

// Bottom-to-top: known-facts fn (lowest), then contains_any, then language fn.
replaceFn('fn is_known_fact_query(normalized: &str) -> bool {', knownFacts);

// Remove the now-unused contains_any helper plus one adjacent blank line.
const caStart = lines.findIndex(
  (l) => l === 'fn contains_any(normalized: &str, needles: &[&str]) -> bool {',
);
if (caStart < 0) throw new Error('contains_any not found');
let caEnd = -1;
for (let i = caStart + 1; i < lines.length; i++) {
  if (lines[i] === '}') {
    caEnd = i;
    break;
  }
}
if (caEnd < 0) throw new Error('contains_any close not found');
// Drop the trailing blank line after the fn if present, else the leading one.
let removeEnd = caEnd;
let removeStart = caStart;
if (lines[caEnd + 1] === '') removeEnd = caEnd + 1;
else if (lines[caStart - 1] === '') removeStart = caStart - 1;
lines = [...lines.slice(0, removeStart), ...lines.slice(removeEnd + 1)];

replaceFn(
  "fn self_awareness_language(prompt: &str, normalized: &str) -> &'static str {",
  language,
);

writeFileSync(path, lines.join('\n'));
console.log('rewrote is_known_fact_query + self_awareness_language, removed contains_any');
