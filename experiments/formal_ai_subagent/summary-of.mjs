#!/usr/bin/env node
// Prints the dependency summary of the corpus documents whose name contains
// the argument, for comparing two trees.
//
// Usage: node experiments/formal_ai_subagent/summary-of.mjs <name-fragment>
import { measureDependencySummarization } from '../../scripts/measure-dependency-summarization.mjs';

const fragment = process.argv[2] ?? '';
const { summaries } = await measureDependencySummarization();
for (const { document, summary } of summaries) {
  if (document.name.includes(fragment)) console.log(`== ${document.name}\n${summary.text}\n`);
}
