// A finite independent assertion grammar over lossless observed lexical tokens.
/** @param {string[]} texts @param {string[]} kinds @param {number} index @returns {boolean} */
export function researchAcceptanceLiteral(texts, kinds, index) {
  if (index < 0 || index >= texts.length || index >= kinds.length) { return false; }
  return kinds[index] === 'numeric' || kinds[index] === 'string'
    || texts[index] === 'true' || texts[index] === 'false' || texts[index] === 'null';
}

/**
 * @param {string[]} texts
 * @param {string[]} kinds
 * @param {string} name
 * @param {string} moduleURL
 * @param {string} moduleDirectory
 * @param {string} testDirectory
 * @param {string} testPath
 * @param {string} command
 * @param {number} arity
 * @param {boolean} syntaxValid
 * @returns {boolean}
 */
export function independentAcceptanceKernel(texts, kinds, name, moduleURL, moduleDirectory,
  testDirectory, testPath, command, arity, syntaxValid) {
  if (!syntaxValid || texts.length !== kinds.length || moduleDirectory !== testDirectory
    || command !== 'node --test ' + testPath || !Number.isSafeInteger(arity) || arity < 0) { return false; }
  let callable = false;
  let assertionImport = false;
  let assertions = 0;
  let index = 0;
  while (index < texts.length) {
    if (texts[index] === ';') { index += 1; continue; }
    if (texts[index] === 'import') {
      if (index + 5 < texts.length && texts[index + 1] === '{' && texts[index + 2] === name
        && texts[index + 3] === '}' && texts[index + 4] === 'from'
        && (texts[index + 5] === "'" + moduleURL + "'" || texts[index + 5] === '"' + moduleURL + '"')) {
        callable = true;
        index += 6;
      } else if (index + 3 < texts.length && texts[index + 1] === 'assert' && texts[index + 2] === 'from'
        && (texts[index + 3] === "'node:assert/strict'" || texts[index + 3] === '"node:assert/strict"')) {
        assertionImport = true;
        index += 4;
      } else { return false; }
    } else {
      if (index + 5 >= texts.length || texts[index] !== 'assert' || texts[index + 1] !== '.'
        || !(texts[index + 2] === 'equal' || texts[index + 2] === 'strictEqual'
          || texts[index + 2] === 'deepEqual' || texts[index + 2] === 'deepStrictEqual')
        || texts[index + 3] !== '(' || texts[index + 4] !== name || texts[index + 5] !== '(') { return false; }
      index += 6;
      for (let argument = 0; argument < arity; argument += 1) {
        if (!researchAcceptanceLiteral(texts, kinds, index)) { return false; }
        index += 1;
        if (argument + 1 < arity) {
          if (index >= texts.length || texts[index] !== ',') { return false; }
          index += 1;
        }
      }
      if (index + 3 >= texts.length) { return false; }
      if (texts[index] !== ')') { return false; }
      if (texts[index + 1] !== ',') { return false; }
      if (!researchAcceptanceLiteral(texts, kinds, index + 2)) { return false; }
      if (texts[index + 3] !== ')') { return false; }
      index += 4;
      assertions += 1;
    }
    if (index < texts.length && texts[index] !== ';') { return false; }
  }
  return callable && assertionImport && assertions > 0;
}
