// Issue #1166: print each dependency-summarization corpus document's language,
// summary length and kept key facts, so two trees can be diffed document by document.
// Usage: node experiments/issue-1166/summarization-per-document.mjs <repo-root>
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
const root = resolve(process.argv[2] ?? '.');
process.chdir(root);
const { measureDependencySummarization } = await import(pathToFileURL(resolve(root, 'scripts/measure-dependency-summarization.mjs')).href);
const { summaries } = await measureDependencySummarization();
for (const { document, summary, facts, retained } of summaries) {
  console.log([document.kind, document.name, document.language, Array.from(summary.text).length, `${retained}/${facts.length}`].join('\t'));
}
