// Issue #1487 audit: which corpus duplicates the old term-subset `restates`
// removed that the checked relation keeps, and why.
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
installTextHost();
const { readDocuments } = await import('../../scripts/measure-dependency-summarization.mjs');
const { sentenceStatements, splitDuplicates } = await import('../../js/agentic/crate/dependency_summarization.mjs');
const { contentIds, statementIdentity } = await import('../../js/agentic/crate/text_formalization.mjs');
const { typedMeaningEntails } = await import('../../js/agentic/crate/typed_meaning.mjs');
const { checkedRelation } = await import('../../js/agentic/crate/statement_relation.mjs');
const oldRestates = (later, earlier) => {
  if (later.language !== earlier.language || !typedMeaningEntails(earlier.dialogue, later.dialogue)) return false;
  if (statementIdentity(later) === statementIdentity(earlier)) return true;
  const ids = contentIds(later); const held = new Set(contentIds(earlier));
  return ids.length > 0 && later.polarity === earlier.polarity && ids.every((id) => held.has(id));
};
for (const document of await readDocuments()) {
  const entries = sentenceStatements(document.text, document.language);
  const unique = [];
  for (const entry of entries) {
    const earlier = unique.find((candidate) => oldRestates(entry.statement, candidate.statement));
    if (earlier === undefined) { unique.push(entry); continue; }
    const relation = checkedRelation(entry.statement, earlier.statement);
    if (relation.verdict !== 'equivalent' && relation.verdict !== 'entailed') {
      console.log(`${document.kind} ${document.name} [${relation.verdict}/${relation.reason}]\n  later:   ${JSON.stringify(entry.statement.text)}\n  earlier: ${JSON.stringify(earlier.statement.text)}`);
    }
  }
}
process.exit(0);
