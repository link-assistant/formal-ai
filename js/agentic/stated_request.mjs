// Which part of a prompt states the work (rust/src/agentic_coding/stated_request.rs).

import { trim } from './crate/rust_str.mjs';

/**
 * Mirrors `fn request_blocks` in rust/src/agentic_coding/stated_request.rs: the
 * prompt's blank-line-separated blocks (trimmed), or the whole prompt when it
 * has fewer than two.
 * @param {string} prompt
 * @returns {Array<string>}
 */
export function requestBlocks(prompt) {
  const blocks = prompt.split('\n\n').map(trim).filter((block) => block !== '');
  return blocks.length < 2 ? [prompt] : blocks;
}
