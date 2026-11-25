// Issue #1632: print each corpus document's removed-duplicate count, so two
// runs (before/after a seed edit) can be diffed to find the affected document.
import { measureDependencySummarization } from '../../scripts/measure-dependency-summarization.mjs';

const { summaries } = await measureDependencySummarization();
for (const { document, summary } of summaries) {
  console.log(`${summary.duplicates}\t${document.language}\t${document.name}`);
}

// `--removed <document>` prints the statements removed as duplicates there.
const at = process.argv.indexOf('--removed');
if (at > 0) {
  const target = summaries.find(({ document }) => document.name === process.argv[at + 1]);
  for (const entry of target?.summary.removed ?? []) console.log(`removed: ${entry.statement.text ?? JSON.stringify(entry.statement).slice(0, 200)}`);
}
