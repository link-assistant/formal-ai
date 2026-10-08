// Spelling correction by discovery (rust/src/agentic_coding/spelling.rs, PR #1188
// dogfooding): "Fix the typo 'smal' in README.md." states the misspelled word
// and not its correction, so the correction is the unique most frequent word
// one edit (Damerau: insert, delete, substitute, transpose) away from it in a
// vocabulary counted from the bundled seed files — the same files both roots
// embed, read once and cached. No word list is authored for this.

import { cached, childrenNamed, parseLino, readText } from '../host.mjs';

/** Mirrors `fn vocabulary`: lowercase alphabetic word -> occurrence count. */
export function vocabulary() {
  return cached('spelling:vocabulary', () => {
    const counts = new Map();
    const root = parseLino(readText('data/meta/seed-registry.lino'));
    const registry = childrenNamed(root, 'seed').length ? root : root.children[0];
    for (const seed of childrenNamed(registry, 'seed')) {
      if (!childrenNamed(seed, 'bundle').some((flag) => flag.value === 'true')) continue;
      const text = readText(`data/seed/${seed.value}.lino`) ?? '';
      for (const match of text.matchAll(/\p{Alphabetic}+/gu)) {
        const word = match[0].toLowerCase();
        counts.set(word, (counts.get(word) ?? 0) + 1);
      }
    }
    return counts;
  });
}

/** Mirrors `fn one_edit_apart`: exactly one Damerau edit between `left` and `right`. */
export function oneEditApart(left, right) {
  const a = Array.from(left);
  const b = Array.from(right);
  if (Math.abs(a.length - b.length) > 1 || left === right) return false;
  let start = 0;
  while (start < a.length && start < b.length && a[start] === b[start]) start += 1;
  if (a.length === b.length) {
    const rest = (from) => a.slice(from).join('') === b.slice(from).join('');
    return rest(start + 1)
      || (start + 1 < a.length && a[start] === b[start + 1] && a[start + 1] === b[start] && rest(start + 2));
  }
  const [longer, shorter] = a.length > b.length ? [a, b] : [b, a];
  return longer.slice(start + 1).join('') === shorter.slice(start).join('');
}

/**
 * Mirrors `fn corrected_spelling`: the unique most frequent vocabulary word
 * one edit from `word`, in `word`'s capitalisation, or null.
 */
export function correctedSpelling(word) {
  const lowered = word.toLowerCase();
  let best = null;
  let bestCount = 0;
  let tied = false;
  for (const [candidate, count] of vocabulary()) {
    if (!oneEditApart(lowered, candidate)) continue;
    if (count > bestCount) {
      best = candidate;
      bestCount = count;
      tied = false;
    } else if (count === bestCount) tied = true;
  }
  if (best === null || tied) return null;
  const first = Array.from(word)[0];
  return first !== first.toLowerCase() ? best.charAt(0).toUpperCase() + best.slice(1) : best;
}
