// Issue #1487: count duplicates per corpus kind under the pre-#1487 term-subset
// `restates` and under the shipped checked relation, to reconcile ratchet moves
// after merging main. node experiments/issue-1487/dedup-count.mjs
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
installTextHost();
const { readDocuments } = await import('../../scripts/measure-dependency-summarization.mjs');
const { sentenceStatements, splitDuplicates } = await import('../../js/agentic/crate/dependency_summarization.mjs');
const { contentIds, statementIdentity } = await import('../../js/agentic/crate/text_formalization.mjs');
const { typedMeaningEntails } = await import('../../js/agentic/crate/typed_meaning.mjs');
const oldRestates = (later, earlier) => {
  if (later.language !== earlier.language || !typedMeaningEntails(earlier.dialogue, later.dialogue)) return false;
  if (statementIdentity(later) === statementIdentity(earlier)) return true;
  const ids = contentIds(later); const held = new Set(contentIds(earlier));
  return ids.length > 0 && later.polarity === earlier.polarity && ids.every((id) => held.has(id));
};
const counts = {};
for (const document of await readDocuments()) {
  const entries = sentenceStatements(document.text, document.language);
  const unique = []; let old = 0;
  for (const entry of entries) {
    if (unique.some((candidate) => oldRestates(entry.statement, candidate.statement))) { old += 1; continue; }
    unique.push(entry);
  }
  const shipped = splitDuplicates(entries).removed.length;
  const row = (counts[document.kind] ??= { old: 0, shipped: 0 });
  row.old += old; row.shipped += shipped;
}
console.log(JSON.stringify(counts));
