// Issue #1562: write the obligation text (citations blanked) of each fixture
// and of the real Hello World work items to <out>/, for solving.
// Usage: node experiments/issue-1562/obligation-texts.mjs <out>
import { readFileSync, writeFileSync } from 'node:fs';
import { obligationText } from '../../js/agentic/work_item_citations.mjs';

const out = process.argv[2];
for (const n of [4031, 4034, 4035]) {
  const text = readFileSync(`data/benchmarks/fixtures/issue-1562/hive-mind-${n}.txt`, 'utf8');
  writeFileSync(`${out}/${n}.full.txt`, text);
  writeFileSync(`${out}/${n}.obl.txt`, obligationText(text));
}
for (const issue of JSON.parse(readFileSync('rust/tests/fixtures/issue-1364-hello-world-issues.json', 'utf8'))) {
  const name = issue.title.split(' ').at(-1);
  const text = `${issue.title}\n\n${issue.body}`;
  writeFileSync(`${out}/${name}.full.txt`, text);
  writeFileSync(`${out}/${name}.obl.txt`, obligationText(text));
}
