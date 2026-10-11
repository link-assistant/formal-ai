// `crate::probability::symbolic_cosine_similarity` (rust/src/probability.rs),
// computed in f32 the way Rust does (`Math.fround` after each step).

const f32 = Math.fround;
const ALPHANUMERIC = /^[\p{Alphabetic}\p{N}]$/u;

/** Mirrors `fn tokenize_symbolic`: split on non-alphanumerics, lowercase. */
function tokenizeSymbolic(value) {
  const tokens = [];
  let current = '';
  for (const character of value) {
    if (ALPHANUMERIC.test(character)) {
      current += character;
    } else {
      if (current) tokens.push(current.toLowerCase());
      current = '';
    }
  }
  if (current) tokens.push(current.toLowerCase());
  return tokens;
}

/** Mirrors `fn bag_of_words`. */
function bagOfWords(tokens) {
  const counts = new Map();
  for (const token of tokens) counts.set(token, (counts.get(token) ?? 0) + 1);
  return counts;
}

/** Mirrors `fn count_to_f32` (saturating at `u16::MAX`). */
const countToF32 = (value) => f32(Math.min(value, 0xffff));

/** Mirrors `fn vector_norm`. */
function vectorNorm(counts) {
  let sum = 0;
  for (const count of counts.values()) {
    const value = countToF32(count);
    sum = f32(value * value + sum);
  }
  return f32(Math.sqrt(sum));
}

/**
 * Mirrors `fn symbolic_cosine_similarity` in rust/src/probability.rs.
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function symbolicCosineSimilarity(a, b) {
  const left = tokenizeSymbolic(a);
  const right = tokenizeSymbolic(b);
  if (!left.length || !right.length) return 0;
  const leftCounts = bagOfWords(left);
  const rightCounts = bagOfWords(right);
  let dot = 0;
  for (const [token, leftCount] of leftCounts) {
    const rightCount = rightCounts.get(token);
    if (rightCount !== undefined) dot = f32(countToF32(leftCount) * countToF32(rightCount) + dot);
  }
  const leftNorm = vectorNorm(leftCounts);
  const rightNorm = vectorNorm(rightCounts);
  const epsilon = 1.1920929e-7;
  if (leftNorm <= epsilon || rightNorm <= epsilon) return 0;
  return Math.min(Math.max(f32(dot / f32(leftNorm * rightNorm)), 0), 1);
}
