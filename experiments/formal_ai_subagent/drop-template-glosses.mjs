#!/usr/bin/env node
// Removes the gloss lexeme of response records, by rule (PR #1188, LEXEMES).
// A response record (role `localized_response_template`, or `defined-by
// answer`) is read by a program, and the hundreds of such records in data/seed
// own no lexeme; a few
// still carry one English surface that only spells their slug ("response set
// assistant name es" for `response_set_assistant_name_es`). That surface is a
// machine label nobody types, so it owes no translation: the rule drops it and
// the record joins the others. A record whose lexemes are real words (any
// other language, any other text) is left alone.
//
// Usage: node experiments/formal_ai_subagent/drop-template-glosses.mjs [--write] <seed.lino>...
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { basename } from 'node:path';

const MIRRORS = ['rust/embedded/data/seed', 'js/seed', 'packages/formal-ai-engine/assets/seed'];
const RESPONSE_MARKERS = ['role localized_response_template', 'defined-by answer'];

function indentOf(line) {
  return line.length - line.trimStart().length;
}

/** The [start, end] line ranges of the gloss lexemes in `lines`. */
function glossLexemes(lines) {
  const found = [];
  for (const [index, line] of lines.entries()) {
    if (indentOf(line) !== 2 || line.trim() === '' || line.trim().startsWith('#')) continue;
    const slug = line.trim();
    let end = index;
    while (end + 1 < lines.length && (lines[end + 1].trim() === '' || indentOf(lines[end + 1]) > 2)) end += 1;
    const body = lines.slice(index + 1, end + 1);
    const isResponse = body.some((child) => RESPONSE_MARKERS.includes(child.trim()));
    if (!isResponse) continue;
    const lexemeStarts = body
      .map((child, offset) => [child.trim(), offset])
      .filter(([text]) => text.startsWith('lexeme '));
    if (lexemeStarts.length !== 1 || lexemeStarts[0][0] !== 'lexeme en') continue;
    const start = index + 1 + lexemeStarts[0][1];
    let lexemeEnd = start;
    while (lexemeEnd + 1 <= end && indentOf(lines[lexemeEnd + 1]) > indentOf(lines[start])) lexemeEnd += 1;
    const texts = lines.slice(start, lexemeEnd + 1).filter((child) => child.trim().startsWith('text '));
    const gloss = slug.replaceAll('_', ' ');
    const spellsSlug = texts.length === 1 && texts[0].trim().replace(/^text "?|"$/gu, '') === gloss;
    if (spellsSlug) found.push({ slug, start, end: lexemeEnd });
  }
  return found;
}

function main() {
  const argv = process.argv.slice(2);
  const write = argv[0] === '--write' ? (argv.shift(), true) : false;
  for (const seed of argv) {
    const previous = readFileSync(seed, 'utf8');
    const lines = previous.split('\n');
    const glosses = glossLexemes(lines);
    console.log(`${seed}: ${glosses.length} gloss lexeme(s): ${glosses.map((gloss) => gloss.slug).join(', ')}`);
    if (!write || glosses.length === 0) continue;
    for (const gloss of [...glosses].reverse()) lines.splice(gloss.start, gloss.end - gloss.start + 1);
    writeFileSync(seed, lines.join('\n'));
    for (const directory of MIRRORS) {
      const target = `${directory}/${basename(seed)}`;
      if (existsSync(target) && readFileSync(target, 'utf8') === previous) copyFileSync(seed, target);
    }
  }
}

main();
