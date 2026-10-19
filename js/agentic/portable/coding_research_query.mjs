/** @param {string} character @returns {boolean} */
export function researchQueryWhitespace(character) {
  return character === ' ' || character === '\t' || character === '\n' || character === '\r'
    || character === '\u000b' || character === '\u000c' || character === '\u00a0' || character === '\u1680'
    || character === '\u2000' || character === '\u2001' || character === '\u2002' || character === '\u2003'
    || character === '\u2004' || character === '\u2005' || character === '\u2006' || character === '\u2007'
    || character === '\u2008' || character === '\u2009' || character === '\u200a' || character === '\u2028'
    || character === '\u2029' || character === '\u202f' || character === '\u205f' || character === '\u3000'
    || character === '\ufeff';
}
/** @param {string} character @returns {boolean} */
export function researchQueryLetter(character) {
  const letters = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
  const lower = researchQueryLower(character);
  for (let index = 0; index < letters.length; index += 1) { if (letters[index] === lower) { return true; } }
  return false;
}
/** @param {string} character @returns {boolean} */
export function researchQueryIdentifierStart(character) {
  return researchQueryLetter(character) || character === '_' || character === '$';
}
/** @param {string} character @returns {boolean} */
export function researchQueryIdentifierPart(character) {
  if (researchQueryIdentifierStart(character)) { return true; }
  const digits = ['0','1','2','3','4','5','6','7','8','9'];
  for (let index = 0; index < digits.length; index += 1) { if (digits[index] === character) { return true; } }
  return false;
}
/** @param {string} character @returns {string} */
export function researchQueryLower(character) {
  const upper = ['A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'];
  const lower = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
  for (let index = 0; index < upper.length; index += 1) {
    if (upper[index] === character) { return lower[index]; }
  }
  return character;
}
/** @param {string[]} characters @returns {string[]} */
export function researchQueryWithoutSignatures(characters) {
  let clean = [''];
  for (let index = 0; index < characters.length; index += 1) {
    let tokenEnd = index;
    let path = false;
    for (; tokenEnd < characters.length && !researchQueryWhitespace(characters[tokenEnd]); tokenEnd += 1) {
      if (characters[tokenEnd] === '/' || characters[tokenEnd] === '\\' || characters[tokenEnd] === '.') { path = true; }
    }
    let end = index;
    if (path) { clean = [...clean, ' ']; index = tokenEnd - 1; }
    else if (researchQueryIdentifierStart(characters[index])) {
      for (; end < characters.length && researchQueryIdentifierPart(characters[end]); end += 1) {}
      for (; end < characters.length && researchQueryWhitespace(characters[end]); end += 1) {}
      if (end < characters.length && characters[end] === '(') {
        end += 1;
        for (; end < characters.length && characters[end] !== '(' && characters[end] !== ')'; end += 1) {}
        if (end < characters.length && characters[end] === ')') { index = end; clean = [...clean, ' ']; }
        else { clean = [...clean, characters[index]]; }
      } else { clean = [...clean, characters[index]]; }
    } else { clean = [...clean, characters[index]]; }
  }
  return clean;
}
/** Quote spans are removed by the public adapter; all query selection lives here.
 * @param {string} language @param {string[]} characters @param {string} name
 * @param {string[]} parameters @returns {string} */
export function codingResearchQueryKernel(language, characters, name, parameters) {
  const clean = researchQueryWithoutSignatures(characters);
  let terms = [''];
  let count = 0;
  let query = language + ' function';
  for (let index = 0; index < clean.length && count < 8; index += 1) {
    if (researchQueryLetter(clean[index])) {
      let word = '';
      let end = index;
      for (; end < clean.length && (researchQueryLetter(clean[end]) || clean[end] === '-'); end += 1) {
        word += researchQueryLower(clean[end]);
      }
      if (end - index >= 3) {
        let excluded = word === name;
        for (let at = 0; at < parameters.length; at += 1) { if (word === parameters[at]) { excluded = true; } }
        for (let at = 0; at < terms.length; at += 1) { if (word === terms[at]) { excluded = true; } }
        if (!excluded) { terms = [...terms, word]; count += 1; query += ' ' + word; }
      }
      index = end - 1;
    }
  }
  return query;
}
