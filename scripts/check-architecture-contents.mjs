#!/usr/bin/env node
// ARCHITECTURE.md is a table of contents (R1188-U2, the "Small Public
// Documentation" principle of link-foundation/code-architecture-principles).
//
// Each architecture topic lives in one file under docs/architecture/, and
// ARCHITECTURE.md links to it. This check keeps the contents and the topic
// files from drifting apart:
// - every docs/architecture/*.md file is linked from ARCHITECTURE.md;
// - every linked topic file exists;
// - every `#anchor` of a link names a heading of its topic file, by the
//   anchor rule GitHub renders headings with.
//
// Usage:
//   node scripts/check-architecture-contents.mjs   print problems, exit 1 on any

import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
export const CONTENTS = 'ARCHITECTURE.md';
export const TOPICS = 'docs/architecture';

/**
 * The anchor GitHub gives a heading: lowercase, punctuation other than `-`
 * and `_` removed, each space a `-`.
 * @param {string} heading the heading text without its `#` marks
 */
export function headingAnchor(heading) {
  return heading
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{M}\p{N} _-]/gu, '')
    .replace(/ /gu, '-');
}

/**
 * Every anchor a Markdown document's headings render to, in order. A repeated
 * anchor gets `-1`, `-2`, … as GitHub numbers it. Headings inside fenced code
 * are not headings.
 * @param {string} markdown
 */
export function documentAnchors(markdown) {
  const anchors = new Set();
  const seen = new Map();
  let fenced = false;
  for (const line of markdown.split('\n')) {
    if (/^\s*(```|~~~)/u.test(line)) {
      fenced = !fenced;
      continue;
    }
    const heading = fenced ? null : /^#{1,6}\s+(.*?)\s*#*\s*$/u.exec(line);
    if (!heading) continue;
    const anchor = headingAnchor(heading[1]);
    const count = seen.get(anchor) ?? 0;
    seen.set(anchor, count + 1);
    anchors.add(count === 0 ? anchor : `${anchor}-${count}`);
  }
  return anchors;
}

/**
 * The links of `markdown` into the topic directory: `{ file, anchor }`, the
 * anchor `''` when the link names none.
 * @param {string} markdown
 * @param {string} topics the topic directory, relative to the repository root
 */
export function topicLinks(markdown, topics = TOPICS) {
  const links = [];
  const pattern = new RegExp(`\\]\\((${topics.replace(/[/.]/gu, '\\$&')}/[^)#\\s]+\\.md)(?:#([^)\\s]*))?\\)`, 'gu');
  for (const match of markdown.matchAll(pattern)) links.push({ file: match[1], anchor: match[2] ?? '' });
  return links;
}

/**
 * The problems of a contents document against its topic files.
 * @param {string} contents the text of ARCHITECTURE.md
 * @param {Map<string, string>} topicFiles every topic file's path and text
 * @returns {string[]}
 */
export function contentsProblems(contents, topicFiles) {
  const problems = [];
  const links = topicLinks(contents);
  const linked = new Set(links.map((link) => link.file));
  for (const file of topicFiles.keys()) {
    if (!linked.has(file)) problems.push(`${file} is not linked from ${CONTENTS}`);
  }
  const anchorsOf = new Map();
  for (const link of links) {
    const text = topicFiles.get(link.file);
    if (text === undefined) {
      problems.push(`${CONTENTS} links ${link.file}, which does not exist`);
      continue;
    }
    if (link.anchor === '') continue;
    if (!anchorsOf.has(link.file)) anchorsOf.set(link.file, documentAnchors(text));
    if (!anchorsOf.get(link.file).has(link.anchor)) {
      problems.push(`${CONTENTS} links ${link.file}#${link.anchor}, which names no heading of that file`);
    }
  }
  return problems;
}

function main() {
  const directory = join(ROOT, TOPICS);
  const topicFiles = new Map();
  for (const name of readdirSync(directory).filter((file) => file.endsWith('.md')).sort()) {
    topicFiles.set(`${TOPICS}/${name}`, readFileSync(join(directory, name), 'utf8'));
  }
  const contentsPath = join(ROOT, CONTENTS);
  const contents = existsSync(contentsPath) ? readFileSync(contentsPath, 'utf8') : '';
  const problems = contentsProblems(contents, topicFiles);
  for (const problem of problems) console.error(`::error file=${CONTENTS}::${problem}`);
  const links = topicLinks(contents).length;
  console.log(`architecture contents: ${topicFiles.size} topic files, ${links} links, ${problems.length} problem(s)`);
  return problems.length === 0 ? 0 : 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main();
}
