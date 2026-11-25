// Issue #1632: which earlier statement of issue 80 "restates" the sentence
// "Open formal-ai-memory.lino …", and how both are formalized. Run it with and
// without the отправь/отправить report-verb lexemes to see the tie-break shift.
import { readDocuments } from '../../scripts/measure-dependency-summarization.mjs';
import { restates, sentenceStatements } from '../../js/agentic/crate/dependency_summarization.mjs';

const document = (await readDocuments()).find((candidate) => candidate.name.endsWith('issue-80.md'));
const entries = sentenceStatements(document.text, document.language);
const target = entries.find((entry) => entry.statement.text === 'Open formal-ai-memory.');
for (const entry of entries) {
  if (entry !== target && restates(target.statement, entry.statement)) console.log('restated by:', entry.statement.text);
}
const earlier = entries.find((entry) => entry.statement.text.includes('save formal-ai-memory'));
const report = earlier?.statement.terms.find((term) => term.surface === 'report');
console.log(`"Open" names ${target.statement.subject.id}; "report" names ${report?.id}`);
