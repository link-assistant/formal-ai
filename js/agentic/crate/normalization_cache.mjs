// Exact-input normalization cache owned by the installed source host.
import { cached } from '../host.mjs';

export function memoizedNormalization(prompt, compute) {
  prompt = String(prompt);
  const cache = cached('bounded-normalized-prompts', () => ({entries: new Map(), bytes: 0}));
  if (cache.entries.has(prompt)) return cache.entries.get(prompt);
  const value = compute(prompt);
  // UTF-16 strings: use a conservative two-byte bound on retained characters.
  const bytes = 2 * (prompt.length + value.length), maximum = 4 * 1024 * 1024;
  if (bytes <= maximum / 4) {
    while (cache.entries.size >= 32 || cache.bytes + bytes > maximum) {
      const key = cache.entries.keys().next().value;
      cache.bytes -= 2 * (key.length + cache.entries.get(key).length);
      cache.entries.delete(key);
    }
    cache.entries.set(prompt, value); cache.bytes += bytes;
  }
  return value;
}

