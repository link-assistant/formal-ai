#!/usr/bin/env node
// Issue #1569: per-document dump of the dependency-summarization measurement,
// so two commits can be diffed to find which documents moved a ratchet.
// Usage (from a checkout root): node <this file> > out.tsv
import path from 'node:path';
import { pathToFileURL } from 'node:url';
const root = process.cwd();
const { measureDependencySummarization } = await import(pathToFileURL(path.join(root, 'scripts/measure-dependency-summarization.mjs')).href);
const { summaries } = await measureDependencySummarization();
for (const { document, facts, retained, summary } of summaries) {
  console.log([document.kind, document.name, document.language, facts.length, retained, summary.duplicates, Array.from(summary.text).length].join('\t'));
}
