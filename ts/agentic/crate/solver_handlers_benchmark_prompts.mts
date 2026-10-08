// `fact_store_resolves` from rust/src/solver_handlers/benchmark_prompts.rs,
// over the fact records of rust/src/seed/facts.rs: the written ones of
// data/seed/facts.lino, then those derived from the committed Wikidata
// captures (fact_derivation.mjs, issue #1172 R9). Only the fields the match
// reads are kept: `{subject_aliases, question_keywords}`.

import { cached, readText } from '../host.mjs';
import { normalizePrompt } from './engine.mjs';
import { derivedFacts } from './fact_derivation.mjs';
import { splitPipeList } from './seed.mjs';
import { findChildValue, parseRoot } from './seed_parser.mjs';
import { containsCjk } from './seed_meanings.mjs';
import { splitWhitespace } from './rust_str.mjs';

/** Mirrors `fn facts` in rust/src/seed/facts.rs (match fields only). */
function factRecords() {
  return cached('fact-records', () => {
    const tree = parseRoot(readText('data/seed/facts.lino'));
    const entries = tree.name === '' ? tree.children : [tree];
    const out = [];
    for (const entry of entries) {
      if (!entry.name.startsWith('fact_')) continue;
      if (!findChildValue(entry, 'summary') && !findChildValue(entry, 'release_timeline')) continue;
      out.push({
        subject_aliases: splitPipeList(findChildValue(entry, 'subject_aliases')).map((alias) => alias.toLowerCase()),
        question_keywords: splitPipeList(findChildValue(entry, 'question_keywords')).map((word) => word.toLowerCase()),
      });
    }
    for (const record of derivedFacts()) {
      out.push({ subject_aliases: record.subjectAliases, question_keywords: record.questionKeywords });
    }
    return out;
  });
}

/** Mirrors `FactRecord::contains_word_sequence`. */
export function containsWordSequence(normalized, phrase) {
  if (!phrase) return false;
  if (containsCjk(phrase)) return normalized.includes(phrase);
  const phraseTokens = splitWhitespace(normalizePrompt(phrase));
  if (!phraseTokens.length) return false;
  const tokens = splitWhitespace(normalizePrompt(normalized));
  for (let start = 0; start + phraseTokens.length <= tokens.length; start += 1) {
    if (phraseTokens.every((token, offset) => tokens[start + offset] === token)) return true;
  }
  return false;
}

/** Mirrors `FactRecord::matches_normalized`. */
function matchesNormalized(record, normalized) {
  if (!record.subject_aliases.some((alias) => containsWordSequence(normalized, alias))) return false;
  if (!record.question_keywords.length) return true;
  return record.question_keywords.some((keyword) => containsWordSequence(normalized, keyword));
}

/** Mirrors `fn fact_store_resolves`. */
export function factStoreResolves(prompt) {
  const normalized = normalizePrompt(prompt);
  return factRecords().some((record) => matchesNormalized(record, normalized));
}
