// Issue #1487 — bind a proof request to the exact proposition of a library
// theorem before the theorem's proof is offered as the answer. A keyword of
// data/seed/proof-library.lino only proposes a theorem (retrieval); the claim
// around the keyword decides what the request asks:
//
// * affirmed     no negation cue anywhere around the keyword: the request
//                asks for the theorem itself;
// * negated      exactly one negation cue before the keyword, none after it,
//                and only function words after it: the request asks for the
//                theorem's negation, which a theorem marked `negation contradiction`
//                refutes;
// * unsupported  any other placement or count of negation cues: the scope is
//                not read, so the theorem is neither proven nor refuted.
//
// A negated request never receives the theorem's positive success. The Rust
// twin is rust/src/proof_engine/claim_binding.rs.

import { callbackValue as canonicalUnaryValue } from '../callback_values.mjs';
import { cached, parseLino, readText } from '../host.mjs';
import { containsCjk } from './seed_meanings.mjs';
import { functionWords, negationCues, tokenize } from './summarization_vocabulary.mjs';

/** Mirrors `PROOF_LIBRARY_PATH` in rust/src/proof_engine/claim_binding.rs. */
export const PROOF_LIBRARY_PATH = 'data/seed/proof-library.lino';

/** Mirrors `enum ClaimBinding` in rust/src/proof_engine/claim_binding.rs. */
export const ClaimBinding = Object.freeze({ Affirmed: 'affirmed', Negated: 'negated', Unsupported: 'unsupported' });

/** Mirrors `fn library_theorems` in rust/src/proof_engine/claim_binding.rs: `{id, method, keywords, refutesNegation}` in lookup order. */
export function libraryTheorems() {
  return cached('proof-library-theorems', () => {
    const parsed = parseLino(readText(PROOF_LIBRARY_PATH));
    const section = parsed.name === 'proof_library' ? parsed : (parsed.children ?? []).find((node) => node.name === 'proof_library');
    return (section?.children ?? []).filter((node) => node.name === 'theorem').map((node) => {
      const children = node.children ?? [];
      const field = (name) => children.filter((child) => child.name === name).map(canonicalUnaryValue);
      return Object.freeze({
        id: canonicalUnaryValue(node),
        method: field('method')[0] ?? '',
        keywords: Object.freeze(field('keyword')),
        refutesNegation: field('negation').includes('contradiction'),
      });
    });
  });
}

/**
 * Mirrors `fn theorem_match` in rust/src/proof_engine/claim_binding.rs: the first theorem (lookup order) one of whose
 * keywords occurs in `claim`, with that keyword (`ProofLibrary::theorem_for` plus the keyword that matched).
 */
export function theoremMatch(claim) {
  for (const theorem of libraryTheorems()) {
    const keyword = theorem.keywords.find((candidate) => candidate !== '' && claim.includes(candidate));
    if (keyword !== undefined) return { theorem, keyword };
  }
  return null;
}

/**
 * Mirrors `fn negation_cue_count` in rust/src/proof_engine/claim_binding.rs: seeded negation cues in `text`; a CJK run is
 * scanned for the longest cue at each position, other tokens must equal a cue.
 */
export function negationCueCount(text) {
  const cues = negationCues();
  const cjk = cues.filter(containsCjk).sort((left, right) => Array.from(right).length - Array.from(left).length);
  let count = 0;
  for (const token of tokenize(text)) {
    if (!containsCjk(token)) {
      if (cues.includes(token)) count += 1;
      continue;
    }
    const characters = Array.from(token);
    let index = 0;
    while (index < characters.length) {
      const rest = characters.slice(index).join('');
      const cue = cjk.find((candidate) => rest.startsWith(candidate));
      if (cue === undefined) index += 1;
      else { count += 1; index += Array.from(cue).length; }
    }
  }
  return count;
}

/** Mirrors `fn only_function_words` in rust/src/proof_engine/claim_binding.rs. */
export function onlyFunctionWords(text) {
  return tokenize(text).every((token) => functionWords().includes(token));
}

/** Mirrors `fn bind_theorem_claim` in rust/src/proof_engine/claim_binding.rs: what `claim` asks of the keyword's theorem. */
export function bindTheoremClaim(claim, keyword) {
  const at = claim.indexOf(keyword);
  if (keyword === '' || at < 0) return ClaimBinding.Unsupported;
  const after = claim.slice(at + keyword.length);
  const before = negationCueCount(claim.slice(0, at));
  const trailing = negationCueCount(after);
  if (before === 0 && trailing === 0) return ClaimBinding.Affirmed;
  if (before === 1 && trailing === 0 && onlyFunctionWords(after)) return ClaimBinding.Negated;
  return ClaimBinding.Unsupported;
}
