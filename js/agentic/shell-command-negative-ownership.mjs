// Seed-owned negated policy scope; quoted program data grants no command authority.
import { callerContextVocabulary } from './crate/seed_caller_context.mjs';
import { containsCjk, mentionsRole } from './crate/seed_meanings.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { rawLowercaseSpan } from './write_request/lowercase_spans.mjs';
import { isAlphanumeric } from './crate/rust_str.mjs';

export function negatedCommandPolicy(sentence) {
  if (typeof sentence !== 'string') return false;
  const lower = sentence.toLowerCase(), quotes = quotedSegmentSpans(sentence);
  const word = c => c !== undefined && (isAlphanumeric(c) || c === '_');
  for (const surface of callerContextVocabulary().policy_leads) {
    const lead = surface.trim().toLowerCase();
    if (!lead || !mentionsRole('statement_negation_cue', lead)) continue;
    for (let at = lower.indexOf(lead); at >= 0; at = lower.indexOf(lead, at + 1)) {
      const span = rawLowercaseSpan(sentence, [at, at + lead.length]);
      if (span === null) continue;
      const [start, end] = span;
      if (sentence.slice(0, start).trim() !== '') continue;
      if (quotes.some(quote => start < quote.end && end > quote.start)) continue;
      if (!containsCjk(lead) && (word(Array.from(sentence.slice(0, start)).at(-1)) || word(Array.from(sentence.slice(end))[0]))) continue;
      return true;
    }
  }
  return false;
}
