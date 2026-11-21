// Keep contributor-controlled diagnostics as data while preserving exact twins.
// Twin: scripts/lib/github-actions-log.rs::safe_log_text.
// Intentional Actions annotations must still use their existing property/data escaping.
export function safeLogText(value) {
  if (!value.includes('::') && !value.includes('##[') && !/\p{Cc}/u.test(value)) return value;
  return JSON.stringify(value).replaceAll(':', '\\u003a').replaceAll('#', '\\u0023')
    .replace(/[\u007f-\u009f]/gu, character => '\\u' + character.charCodeAt(0).toString(16).padStart(4, '0'));
}
