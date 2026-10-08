// Whether a token written in a request names a file at all
// (rust/src/agentic_coding/file_path_shape.rs).

/**
 * Mirrors `fn is_dotted_number` in rust/src/agentic_coding/file_path_shape.rs.
 * @param {string} token
 */
export function isDottedNumber(token) {
  return token.includes('.') && /[0-9]/.test(token) && /^[0-9.]*$/.test(token);
}

/**
 * Mirrors `fn trim_trailing_sentence_dot` in rust/src/agentic_coding/file_path_shape.rs.
 * @param {string} token
 */
export function trimTrailingSentenceDot(token) {
  const trimmed = token.replace(/\.+$/, '');
  return trimmed === '' || trimmed.endsWith('/') ? token : trimmed;
}

/**
 * Mirrors `fn peel_sentence_punctuation` in rust/src/agentic_coding/file_path_shape.rs:
 * peel `strip` and the sentence's terminating dot to a fixpoint.
 * @param {string} token
 * @param {(token: string) => string} strip
 */
export function peelSentencePunctuation(token, strip) {
  let current = token;
  for (;;) {
    const next = trimTrailingSentenceDot(strip(current));
    if (next === current) return current;
    current = next;
  }
}
