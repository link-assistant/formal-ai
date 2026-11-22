#!/usr/bin/env node
// Keep the required public entry points and their useful sections present.
import {readFileSync} from 'node:fs';
import {resolve, dirname} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
export const requirements = JSON.parse(readFileSync(
  resolve(ROOT, 'data/meta/required-documentation.json'), 'utf8'));

/** Level-two headings outside Markdown fences; inner/shorter fences stay code. */
export function sectionHeadings(markdown) {
  const headings = new Set();
  let fence = null;
  for (const line of markdown.split(/\r?\n/u)) {
    if (fence) {
      const close = /^ {0,3}(`+|~+)\s*$/u.exec(line)?.[1];
      if (close?.[0] === fence[0] && close.length >= fence.length) fence = null;
      continue;
    }
    const open = /^ {0,3}(`{3,}|~{3,})(.*)$/u.exec(line);
    if (open && (open[1][0] !== '`' || !open[2].includes('`'))) {
      fence = open[1];
      continue;
    }
    const heading = /^ {0,3}##[ \t]+(.+?)(?:[ \t]+#+)?[ \t]*$/u.exec(line)?.[1];
    if (heading) headings.add(heading);
  }
  return headings;
}

/** Read every requirement; missing/unreadable files are failures, never omissions. */
export function documentationProblems(root = ROOT, contract = requirements) {
  const problems = [];
  for (const [file, required] of Object.entries(contract)) {
    let markdown;
    try { markdown = readFileSync(resolve(root, file), 'utf8'); }
    catch (error) {
      problems.push(`${file}: cannot read required documentation (${error.code})`);
      continue;
    }
    const headings = sectionHeadings(markdown);
    for (const heading of required) {
      if (!headings.has(heading)) problems.push(`${file}: missing required heading "## ${heading}"`);
    }
  }
  return problems;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === '--list') console.log(JSON.stringify(requirements, null, 2));
  else if (args.length) {
    console.error('Usage: node scripts/check-required-documentation.mjs [--list]');
    process.exitCode = 2;
  } else {
    const problems = documentationProblems();
    for (const problem of problems) console.error(problem);
    console.log(`required documentation: ${Object.keys(requirements).length} files, ${problems.length} problem(s)`);
    process.exitCode = problems.length ? 1 : 0;
  }
}
