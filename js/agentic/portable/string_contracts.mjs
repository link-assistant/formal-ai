// Portable contract: valid Unicode scalar text. NUL is preserved, not sanitized.
/** @param {string} value @returns {string} */
export function quoteShellWord(value) {
  return "'" + value.replaceAll("'", "'\\''") + "'";
}
/** @param {string} body @param {string} invitation @returns {string} */
export function joinFailureInvitation(body, invitation) {
  return invitation === '' ? body : body === '' ? invitation : body + '\n\n' + invitation;
}
