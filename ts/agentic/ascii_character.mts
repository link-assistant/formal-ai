/** Rust built-in `char::is_ascii_alphanumeric`. @param {string|undefined} character */
export function isAsciiAlphanumeric(character) {
  return character !== undefined && /^[A-Za-z0-9]$/.test(character);
}

