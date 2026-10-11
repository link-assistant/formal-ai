// The fenced-block helper final answers share (rust/src/issue_report.rs).

/** Mirrors `const LINO_FENCE_LANGUAGE`. */
export const LINO_FENCE_LANGUAGE = 'lino';

/**
 * Mirrors `fn fenced_block` in rust/src/issue_report.rs.
 * @param {string} language
 * @param {string} content
 */
export function fencedBlock(language, content) {
  const body = content.trimEnd();
  let fence = '```';
  while (body.includes(fence)) fence += '`';
  return `${fence}${language}\n${body}\n${fence}`;
}
