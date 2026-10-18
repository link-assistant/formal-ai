// Recursive source content over the shared meta-language grammar front-end.
// The persisted document owns its text; no source buffer, spans or census reload it.
import {parseLinks, writeLink, textOf} from '../logging/lino-codec.mjs';

const QUOTES = {quotedText: 'json'};
const SUPPORTED = new Set(['JavaScript', 'TypeScript', 'Rust']);
const METADATA = new Set(['form', 'role', 'meaning', 'semantic-gap']);

/** @typedef {{quoted: string}} SourceText */
/** @typedef {string | SourceText | SourceLink} SourceItem */
/** @typedef {Array<SourceItem>} SourceLink */
/** @typedef {{term: string, tokenIndex?: number, field?: string|null, children: GrammarNode[], flags?: {isError?: boolean, hasError?: boolean, isMissing?: boolean}}} GrammarNode */
/** @typedef {{tree: GrammarNode, tokens: {text: string}[], leading?: GrammarNode[], trailing?: GrammarNode[]}} ParsedSource */
/** @typedef {{language: string, kind: string, concept: string}} SyntaxRule */

/** @param {string} text @returns {SourceText} */
const quoted = text => ({quoted: text});

/** @param {SourceItem} value @returns {value is SourceLink} */
const isLink = value => Array.isArray(value);

/** @param {SourceLink} node @param {string} name @returns {SourceLink | undefined} */
function field(node, name) {
  const matches = node.filter(item => isLink(item) && item[0] === name);
  if (matches.length > 1) throw Error('AmbiguousSourceField:' + name);
  return matches.find(isLink);
}

/** @param {SourceLink} node @param {string} name @returns {string} */
function fieldText(node, name) {
  const found = field(node, name);
  if (!found || found.length !== 2) throw Error('MissingSourceField:' + name);
  const value = textOf(found[1]);
  if (typeof value !== 'string') throw Error('SourceTextRequired:' + name);
  return value;
}

/** @param {string} source */
function validUnicode(source) {
  for (let index = 0; index < source.length; index += 1) {
    const value = source.charCodeAt(index);
    if (value >= 0xd800 && value <= 0xdbff) {
      const next = source.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) throw Error('UnsupportedUnpairedSurrogate');
      index += 1;
    } else if (value >= 0xdc00 && value <= 0xdfff) throw Error('UnsupportedUnpairedSurrogate');
  }
}

/** A declared concept is a syntactic interpretation, never an equivalence certificate.
 * @param {string} notation @returns {SyntaxRule[]} */
export function readSyntaxRules(notation) {
  const links = parseLinks(notation, QUOTES);
  if (links.length !== 1 || links[0][0] !== 'source-syntax-rules') throw Error('SourceSyntaxRulesRequired');
  const rules = [];
  for (const item of links[0].slice(1)) {
    if (!isLink(item) || item[0] !== 'parse-rule' || item.length !== 4) throw Error('UnsupportedSourceSyntaxRule');
    const language = fieldText(item, 'language'), kind = fieldText(item, 'kind');
    const concept = fieldText(item, 'concept');
    if (!SUPPORTED.has(language) || !/^[a-z][a-z0-9-]*$/u.test(concept)) throw Error('InvalidSourceSyntaxRule');
    if (rules.some(rule => rule.language === language && rule.kind === kind)) throw Error('AmbiguousSourceSyntaxRule');
    rules.push({language, kind, concept});
  }
  return rules;
}

/** Parse through a supplied actual meta-language grammar, not a lexical fallback.
 * @param {string} source @param {string} language
 * @param {{parse: (source: string, language: string) => ParsedSource|undefined, rules: SyntaxRule[]}} options
 * @returns {SourceLink} */
export function sourceToLinks(source, language, {parse, rules}) {
  if (!SUPPORTED.has(language)) throw Error('UnsupportedSourceLanguage:' + language);
  validUnicode(source);
  const parsed = parse(source, language);
  if (!parsed || parsed.tree.flags?.hasError || parsed.tree.flags?.isError) throw Error('UnrepresentedSourceSyntax');
  if (parsed.tokens.map(token => token.text).join('') !== source) throw Error('IncompleteSourcePartition');
  const visited = new Set();
  const convert = node => {
    if (node.flags?.hasError || node.flags?.isError || node.flags?.isMissing) throw Error('UnrepresentedSourceSyntax');
    const matched = rules.find(rule => rule.language === language && rule.kind === node.term);
    const concept = matched?.concept ?? 'syntax';
    const result = [concept, ['form', quoted(node.term)]];
    if (node.field) result.push(['role', quoted(node.field)]);
    if (node.tokenIndex !== undefined) {
      if (node.children.length !== 0 || visited.has(node.tokenIndex)) throw Error('AmbiguousSourceLeaf');
      const token = parsed.tokens[node.tokenIndex];
      if (!token) throw Error('MissingSourceLeaf');
      visited.add(node.tokenIndex);
      result.push(['text', quoted(token.text)]);
    } else {
      for (const child of node.children) result.push(convert(child));
    }
    return result;
  };
  const contents = [...(parsed.leading ?? []), parsed.tree, ...(parsed.trailing ?? [])].map(convert);
  if (visited.size !== parsed.tokens.length) throw Error('UnattachedSourceText');
  const result = ['source', ['language', quoted(language)], ...contents];
  if (sourceFromLinks(result) !== source) throw Error('SourceReconstructionMismatch');
  return result;
}

/** @param {SourceLink} document @returns {string} */
export function sourceFromLinks(document) {
  if (document[0] !== 'source') throw Error('SourceDocumentRequired');
  if (document.slice(1).some(item => !isLink(item))) throw Error('UnexpectedSourcePayload');
  const language = fieldText(document, 'language');
  if (!SUPPORTED.has(language)) throw Error('UnsupportedSourceLanguage:' + language);
  const render = node => {
    if (!Array.isArray(node) || typeof node[0] !== 'string') throw Error('MalformedSourceNode');
    if (node.slice(1).some(item => !isLink(item))) throw Error('UnexpectedSourcePayload');
    fieldText(node, 'form');
    const text = field(node, 'text');
    const children = node.slice(1).filter(item => isLink(item) && !METADATA.has(String(item[0])) && item[0] !== 'text');
    if (text) {
      if (text.length !== 2 || children.length) throw Error('AmbiguousSourceLeaf');
      return fieldText(node, 'text');
    }
    if (node.slice(1).some(item => !isLink(item))) throw Error('UnexpectedSourcePayload');
    return children.map(render).join('');
  };
  const contents = document.slice(1).filter(item => isLink(item) && item[0] !== 'language');
  if (contents.length === 0) throw Error('MissingSourceTree');
  const source = contents.map(render).join('');
  validUnicode(source);
  return source;
}

/** @param {SourceLink} document @returns {string} */
export function writeSourceLinks(document) {
  sourceFromLinks(document);
  const pretty = (node, indent) => {
    const compact = node.slice(1).every(item => !isLink(item)) || field(node, 'text') !== undefined;
    if (compact) return indent + writeLink(node, QUOTES);
    const head = node[0];
    if (typeof head !== 'string' || !/^[a-z][a-z0-9-]*$/u.test(head)) throw Error('InvalidSourceConcept');
    return indent + '(' + head + '\n'
      + node.slice(1).map(item => {
        if (!isLink(item)) throw Error('UnexpectedSourcePayload');
        return pretty(item, indent + '  ');
      }).join('\n') + ')';
  };
  return pretty(document, '') + '\n';
}

/** @param {string} notation @returns {SourceLink} */
export function readSourceLinks(notation) {
  const links = parseLinks(notation, QUOTES);
  if (links.length !== 1) throw Error('OneSourceDocumentRequired');
  sourceFromLinks(links[0]);
  return links[0];
}

/** Reparse independently; stored kind labels do not authenticate their own structure.
 * @param {SourceLink} document
 * @param {{parse: (source: string, language: string) => ParsedSource|undefined, rules: SyntaxRule[]}} options
 * @returns {boolean} */
export function verifySourceLinks(document, options) {
  const fresh = sourceToLinks(sourceFromLinks(document), fieldText(document, 'language'), options);
  // Semantic annotations are validated by their own owner and are not CST authority.
  const concrete = node => node.map(item => isLink(item) ? concrete(item) : item)
    .filter(item => !isLink(item) || !['meaning', 'semantic-gap'].includes(String(item[0])));
  if (JSON.stringify(concrete(fresh)) !== JSON.stringify(concrete(document))) throw Error('SourceStructureMismatch');
  return true;
}

/** Full preimages and independent reparse guard structural edits; no offsets persist.
 * @param {SourceLink} document @param {{role?: string, form: string, expectedText: string, replacementText: string}} change
 * @param {{parse: (source: string, language: string) => ParsedSource|undefined, rules: SyntaxRule[]}} options
 * @returns {{document: SourceLink, inverse: {role?: string, form: string, expectedText: string, replacementText: string}}} */
export function replaceSourceText(document, change, options) {
  verifySourceLinks(document, options);
  const copy = structuredClone(document), matches = [];
  const visit = node => {
    if (node[0] !== 'source' && field(node, 'form') && fieldText(node, 'form') === change.form
      && (!change.role || (field(node, 'role') && fieldText(node, 'role') === change.role))) {
      const text = field(node, 'text');
      if (text && textOf(text[1]) === change.expectedText) matches.push(text);
    }
    for (const item of node.slice(1)) if (isLink(item) && !METADATA.has(String(item[0])) && item[0] !== 'text' && item[0] !== 'language') visit(item);
  };
  visit(copy);
  if (matches.length !== 1) throw Error('AmbiguousOrMissingSourceEdit');
  matches[0][1] = quoted(change.replacementText);
  const updated = sourceToLinks(sourceFromLinks(copy), fieldText(copy, 'language'), options);
  return {document: updated, inverse: {...change, expectedText: change.replacementText, replacementText: change.expectedText}};
}
