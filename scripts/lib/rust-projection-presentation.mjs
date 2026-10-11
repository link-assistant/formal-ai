import {createHash} from 'node:crypto';
import {lex} from '../self-translation/lexer.mjs';
import {readabilityProblems, MAXIMUM_AVERAGE} from '../check-readable-code.mjs';

const hash = source => createHash('sha256').update(source, 'utf8').digest('hex');
const receiptPrefix = '// formal-ai:presentation:v1 ';
const rawPrefix = '// formal-ai:presentation-raw ';
const rawEnd = '// formal-ai:presentation-raw end';
const identity = tokens => tokens.map(token => [token.type, token.text]);

function sourceTokens(source) {
  if (typeof source !== 'string' || Buffer.from(source).toString('utf8') !== source) {
    throw Error('PrimitiveUtf8RustSourceRequired');
  }
  const tokens = lex(source, 'Rust');
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index], following = tokens[index + 1];
    if (token.type === 'comment' && token.text.startsWith('/*') && token.text.slice(2).includes('/*')) {
      throw Error('OpaqueNestedRustComment');
    }
    if (token.type === 'ident' && ['r', 'br', 'b', 'c', 'cr'].includes(token.text)
        && following && token.end === following.start
        && (following.type === 'string' || following.text === '#')) {
      throw Error('OpaqueRustPrefixedLiteral');
    }
  }
  return tokens;
}

/** Reject source-position/opaque macro semantics rather than inferring them from text. */
export function validatePresentationScope(source) {
  const tokens = sourceTokens(source);
  const forbidden = new Set(['macro_rules!', 'macro', 'line!', 'column!', 'include!', 'include_str!',
    'include_bytes!', 'no_implicit_prelude']);
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index];
    if (token.type === 'ident' && forbidden.has(token.text)) throw Error('OpaquePresentationScope');
    if (token.type === 'punct' && token.text === '#') {
      let cursor = index + 1;
      const take = (text, type = 'punct') => {
        const part = tokens[cursor++];
        if (!part || part.type !== type || part.text !== text) throw Error('OpaqueRustAttribute');
      };
      take('!'); take('['); take('allow', 'ident'); take('(');
      let count = 0;
      while (tokens[cursor]?.text !== ')') {
        const lint = tokens[cursor++];
        if (lint?.type !== 'ident' || !/^[A-Za-z_][A-Za-z0-9_]*$/u.test(lint.text)) {
          throw Error('OpaqueRustAttribute');
        }
        count++;
        if (tokens[cursor]?.text !== ')') take(',');
      }
      if (!count) throw Error('OpaqueRustAttribute');
      take(')'); take(']'); index = cursor - 1;
    }

    if (token.type === 'ident' && token.text === 'use') {
      let end = index;
      while (end < tokens.length && tokens[end].text !== ';') end++;
      const declaration = tokens.slice(index, end);
      if (declaration.some(part => part.type === 'punct' && part.text === '*')
          || declaration.some(part => part.type === 'ident' && part.text === 'vec')) {
        throw Error('UnresolvedStandardMacroBinding');
      }
    }
  }
}

/** Only nonempty horizontal token gaps change; literals/comments and punctuation adjacency stay exact. */
export function presentRustCode(source) {
  const original = sourceTokens(source);
  if (!readabilityProblems(source, 'rust').length) return {raw: source, rendered: source, edits: []};
  validatePresentationScope(source);
  for (const token of original) {
    if (token.type === 'ident' && token.text.endsWith('!') && token.text !== 'vec!') {
      throw Error('OpaqueEditedMacro');
    }
  }
  const edits = [];
  for (let index = 0; index + 1 < original.length; index++) {
    const token = original[index], next = original[index + 1];
    const gap = source.slice(token.end, next.start);
    const lineStart = source.lastIndexOf('\n', token.end - 1) + 1;
    const newline = source.indexOf('\n', token.end);
    const lineEnd = newline < 0 ? source.length : newline;
    if ([...source.slice(lineStart, lineEnd)].length <= MAXIMUM_AVERAGE) continue;

    if (token.type !== 'punct' || ![',', ';', '||', '&&'].includes(token.text)
        || !/^[ \t]+$/u.test(gap)) continue;
    edits.push({start: token.end, end: next.start, before: gap, after: '\n    '});
  }
  let rendered = source;
  for (const edit of [...edits].reverse()) {
    rendered = rendered.slice(0, edit.start) + edit.after + rendered.slice(edit.end);
  }
  const after = sourceTokens(rendered);
  if (JSON.stringify(identity(after)) !== JSON.stringify(identity(original))) throw Error('RustTokenIdentityDrift');
  for (let index = 0; index + 1 < original.length; index++) {
    if ((original[index].end === original[index + 1].start) !== (after[index].end === after[index + 1].start)) {
      throw Error('RustTokenAdjacencyDrift');
    }
  }
  if (readabilityProblems(rendered, 'rust').length) throw Error('NoSafeReadablePresentation');
  let inverse = rendered, shift = 0;
  const adjusted = edits.map(edit => {
    const start = edit.start + shift;
    shift += edit.after.length - edit.before.length;
    return {...edit, start, end: start + edit.after.length};
  });
  for (const edit of adjusted.reverse()) {
    if (inverse.slice(edit.start, edit.end) !== edit.after) throw Error('PresentationInverseSpanDrift');
    inverse = inverse.slice(0, edit.start) + edit.before + inverse.slice(edit.end);
  }
  if (inverse !== source) throw Error('PresentationInverseDrift');
  return {raw: source, rendered, edits};
}

export function encodeRustPresentation(source) {
  const proof = presentRustCode(source);
  if (!proof.edits.length) return source;
  const bytes = Buffer.from(source, 'utf8');
  const receipt = receiptPrefix + `rawSHA256=${hash(source)} rawBytes=${bytes.length} renderedSHA256=${hash(proof.rendered)}`;
  const encoded = bytes.toString('base64');
  const chunks = [];
  for (let offset = 0; offset < encoded.length; offset += 96) chunks.push(rawPrefix + encoded.slice(offset, offset + 96));
  return [receipt, ...chunks, rawEnd, proof.rendered].join('\n');
}

/** Checked derivative, not an alternative caller-supplied authority for upstream content. */
export function decodeRustPresentation(lines, start, expectedRawHash) {
  const match = /^\/\/ formal-ai:presentation:v1 rawSHA256=([a-f0-9]{64}) rawBytes=(0|[1-9][0-9]*) renderedSHA256=([a-f0-9]{64})$/u.exec(lines[start]);
  if (!match || match[1] !== expectedRawHash) throw Error('InvalidPresentationReceipt');
  let cursor = start + 1, encoded = '';
  while (lines[cursor]?.startsWith(rawPrefix) && lines[cursor] !== rawEnd) {
    const chunk = lines[cursor].slice(rawPrefix.length);
    if (!/^[A-Za-z0-9+/]{1,96}={0,2}$/u.test(chunk) || chunk.length > 96) throw Error('InvalidPresentationEncoding');
    encoded += chunk; cursor++;
  }
  if (lines[cursor] !== rawEnd || !encoded) throw Error('MissingPresentationRawEnd');
  const bytes = Buffer.from(encoded, 'base64'), raw = bytes.toString('utf8');
  if (bytes.toString('base64') !== encoded || Buffer.from(raw).compare(bytes) !== 0
      || bytes.length !== Number(match[2]) || hash(raw) !== match[1]) throw Error('PresentationRawIdentityDrift');
  const proof = presentRustCode(raw);
  if (!proof.edits.length || hash(proof.rendered) !== match[3]) throw Error('PresentationRegenerationDrift');
  const renderedLines = proof.rendered.split('\n'), renderedStart = cursor + 1;
  if (lines.slice(renderedStart, renderedStart + renderedLines.length).join('\n') !== proof.rendered) {
    throw Error('PresentationRenderedIdentityDrift');
  }
  const encodedAgain = encodeRustPresentation(raw).split('\n');
  const end = renderedStart + renderedLines.length;
  if (lines.slice(start, end).join('\n') !== encodedAgain.join('\n')) throw Error('NoncanonicalPresentation');
  return {raw, rendered: proof.rendered, end, rawSHA256: match[1], renderedSHA256: match[3], edits: proof.edits};
}

export function projectionNeedsPresentation(source) {
  return readabilityProblems(source, 'rust').length !== 0;
}
