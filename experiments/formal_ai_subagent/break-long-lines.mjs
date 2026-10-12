#!/usr/bin/env node
// Break the over-long JSX lines that scripts/check-readable-code.mjs reports
// (issue #1188 R1188-U23), as a rule instead of hand edits.
//
// A line with more code characters than the gate allows is split only at
// element boundaries in code (never inside a string, template or comment):
// between `>` and `<`, `}` and `<`, or `>` and `{`, and at the space before a
// JSX attribute (`name={`, `name="`). JSX drops whitespace that holds a
// newline between two children, and whitespace inside a tag is a separator,
// so the split changes no rendered text; `bun build` of the entry point
// before and after proves it (the minified bundles are byte-identical).
// Pieces are packed up to TARGET_WIDTH characters and continue at the line's
// indentation plus two spaces.
//
// Usage: node experiments/formal_ai_subagent/break-long-lines.mjs [--check] FILE...

import { readFileSync, writeFileSync } from 'node:fs';

import { MAXIMUM_CODE_CHARACTERS, codeMask, familyOf } from '../../scripts/check-readable-code.mjs';

const TARGET_WIDTH = 120;
const BOUNDARIES = ['><', '}<', '>{'];
/** A JSX attribute after a space: the space is the cut. */
const ATTRIBUTE = /^ [A-Za-z][\w-]*=[{"]/u;
/** Words after which a line break would insert a semicolon. */
const RESTRICTED = /\b(?:return|throw|yield|break|continue)$/u;

/** A continuation piece starts at the prefix; the first keeps its own indentation. */
function piece(prefix, text) {
  return prefix ? prefix + text.trimStart() : text;
}

/**
 * Cut `text` (which starts at `prefix`) at the given offsets, packing each
 * piece up to the target width.
 * @returns {Array<{ text: string, start: number }>} the pieces with their offsets in `text`
 */
function pack(text, cuts, firstWidth, width) {
  const pieces = [];
  let start = 0;
  let lastCut = -1;
  let room = firstWidth;
  for (const cut of [...cuts, text.length]) {
    if (cut - start > room && lastCut > start) {
      pieces.push({ text: text.slice(start, lastCut), start });
      start = lastCut;
      room = width;
    }
    lastCut = cut;
  }
  pieces.push({ text: text.slice(start), start });
  return pieces;
}

/**
 * The text with every over-long line broken: first at element boundaries,
 * then, for a piece still wider than the target, at attribute boundaries.
 * @param {string} text
 * @param {string} family
 * @param {number} [width]
 * @returns {string}
 */
export function breakLongLines(text, family, width = TARGET_WIDTH) {
  const mask = codeMask(text, family);
  const out = [];
  let offset = 0;
  for (const line of text.split('\n')) {
    let code = 0;
    for (let index = 0; index < line.length; index += 1) code += mask[offset + index];
    if (code <= MAXIMUM_CODE_CHARACTERS) {
      out.push(line);
      offset += line.length + 1;
      continue;
    }
    const indent = /^\s*/u.exec(line)[0];
    const continuation = `${indent}  `;
    const elements = [];
    const attributes = [];
    for (let index = 1; index < line.length; index += 1) {
      if (mask[offset + index - 1] !== 1 || mask[offset + index] !== 1) continue;
      if (BOUNDARIES.includes(line[index - 1] + line[index])) elements.push(index);
      else if (line[index] === ' ' && ATTRIBUTE.test(line.slice(index, index + 64)) && !RESTRICTED.test(line.slice(0, index))) attributes.push(index);
    }
    let prefix = '';
    for (const element of pack(line, elements, width, width - continuation.length)) {
      const end = element.start + element.text.length;
      const inside = attributes.filter((cut) => cut > element.start && cut < end).map((cut) => cut - element.start);
      const room = width - prefix.length;
      for (const attribute of pack(element.text, inside, room, width - continuation.length)) {
        out.push(piece(prefix, attribute.text).trimEnd());
        prefix = continuation;
      }
    }
    offset += line.length + 1;
  }
  return out.join('\n');
}

if (process.argv[1] && import.meta.url.endsWith(process.argv[1].split('/').pop())) {
  const check = process.argv.includes('--check');
  let drifted = 0;
  for (const path of process.argv.slice(2).filter((argument) => argument !== '--check')) {
    const text = readFileSync(path, 'utf8');
    const broken = breakLongLines(text, familyOf(path) ?? 'script', Number(process.env.BREAK_WIDTH ?? TARGET_WIDTH));
    if (broken === text) continue;
    drifted += 1;
    if (check) console.log(`${path}: has lines to break`);
    else writeFileSync(path, broken);
  }
  process.exitCode = check && drifted > 0 ? 1 : 0;
}
