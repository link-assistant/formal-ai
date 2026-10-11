#!/usr/bin/env node
// Prints one line per dependency-summarization document (summary length, key
// facts kept, duplicates removed), so two trees can be diffed to find which
// documents a seed change moved.
//
// Usage: node experiments/formal_ai_subagent/summary-per-document.mjs
import { measureDependencySummarization } from '../../scripts/measure-dependency-summarization.mjs';

const { summaries } = await measureDependencySummarization();
for (const { document, summary, facts, retained } of summaries) {
  const length = Array.from(summary.text).length;
  console.log(`${document.language}\t${document.name}\t${length}\t${retained}/${facts.length}\t${summary.duplicates}`);
}
