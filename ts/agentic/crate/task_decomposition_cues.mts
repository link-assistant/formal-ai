// Issue #1580: quoted passage cues use the shared seed in both runtimes.
// The native twin is rust/src/meta_executor/decomposition.rs.
import { containsCjk, wordsForRole } from './seed_meanings.mjs';
import { isWhitespace } from './rust_str.mjs';

// A seeded passage cue is one unit only when it directly introduces a quote.
/**
 * @param {string[]} lowered
 * @param {string[]} original
 * @param {boolean[]} mask
 * @param {(kind: string) => string[]} ruleValues
 * @param {function(string[], string[], boolean[], number, string[], boolean): boolean} markerMatches
 */
export function maskQuotedCues(lowered, original, mask, ruleValues, markerMatches) {
  const separators = ruleValues('quote-cue-separator');
  const openers = ruleValues('quote-pair').map((pair) => Array.from(pair)[0]);
  const cues = ruleValues('quote-cue-role').flatMap(wordsForRole)
    .map((word) => ({ chars: Array.from(word).map((character) => character.toLowerCase()),
      cjk: containsCjk(word) }))
    .filter((cue) => cue.chars.length > 0);
  for (let at = 0; at < original.length; at += 1) {
    for (const cue of cues) {
      if (!markerMatches(lowered, original, mask, at, cue.chars, cue.cjk)) continue;
      const end = at + cue.chars.length;
      let next = end;
      while (next < original.length
        && (isWhitespace(original[next]) || separators.includes(original[next]))) next += 1;
      if (openers.includes(original[next]) && mask[next]) {
        mask.fill(true, at, end);
        break;
      }
    }
  }
}
