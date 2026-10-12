// Deterministic ids and token counts shared by every protocol adapter
// (rust/src/web_engine_core.rs `stable_id`, rust/src/engine.rs `estimate_tokens`).

const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;
const MASK = (1n << 64n) - 1n;
const encoder = new TextEncoder();

/**
 * `stable_id`: FNV-1a 64 over the UTF-8 bytes, as `{prefix}_{hash:016x}`.
 * @param {string} prefix
 * @param {string} text
 * @returns {string}
 */
export function stableId(prefix, text) {
  let hash = FNV_OFFSET;
  for (const byte of encoder.encode(String(text))) {
    hash ^= BigInt(byte);
    hash = (hash * FNV_PRIME) & MASK;
  }
  return `${prefix}_${hash.toString(16).padStart(16, '0')}`;
}

/** `estimate_tokens`: the Unicode scalar count. @param {string} text @returns {number} */
export function estimateTokens(text) {
  let count = 0;
  for (const _char of String(text || '')) count += 1;
  return count;
}
