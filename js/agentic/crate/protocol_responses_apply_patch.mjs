// `apply_patch_input` from rust/src/protocol_responses.rs: lower a planner
// write (`{path, content}`) onto the freeform `apply_patch` grammar.

import { rustLines } from '../content.mjs';

const PATCH_BEGIN = '*** Begin Patch';
const PATCH_ADD_FILE = '*** Add File:';
const PATCH_END = '*** End Patch';

/** A JSON object parsed from `text`, or null: Rust dependency `serde_json::from_str::<Value>`. */
export function jsonObject(text) {
  try {
    const value = JSON.parse(text);
    return value && typeof value === 'object' && !Array.isArray(value) ? value : null;
  } catch {
    return null;
  }
}

/**
 * Mirrors `fn apply_patch_input` in rust/src/protocol_responses.rs.
 * @param {string} argumentsText
 * @returns {string|null}
 */
export function applyPatchInput(argumentsText) {
  const value = jsonObject(argumentsText);
  if (!value) return null;
  const key = ['path', 'filePath', 'file_path'].find((name) => typeof value[name] === 'string');
  if (key === undefined) return null;
  const path = value[key];
  const content = value.content;
  if (typeof content !== 'string') return null;
  if (!path || /[\r\n]/.test(path)) return null;
  let rendered = `${PATCH_BEGIN}\n${PATCH_ADD_FILE} ${path}\n`;
  if (!content) rendered += '+\n';
  else for (const line of rustLines(content)) rendered += `+${line}\n`;
  return `${rendered}${PATCH_END}\n`;
}
