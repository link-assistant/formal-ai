// Issue #1562: write the obligation text (citations blanked, `*.obl.txt`) and
// the binding text (the same, with the code spans that are operands of an
// obligation clause kept, `*.bind.txt`) of each fixture, of the real Hello
// World work items and of the #1155 work item to <out>/, for solving, with
// the operands of each (`*.operands`, NUL-separated).
// Usage: node experiments/issue-1562/obligation-texts.mjs <out>
import { readFileSync, writeFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';

await ensureNodeHost({ worker: new WorkerHost() }, { sourceUsageRustGrammar: false });
const { bindingText, obligationText } = await import('../../js/agentic/work_item_citations.mjs');
const { boundOutputLiterals } = await import('../../js/agentic/crate/intent_formalization_obligations.mjs');

const out = process.argv[2];
const write = (name, text) => {
  writeFileSync(`${out}/${name}.full.txt`, text);
  writeFileSync(`${out}/${name}.obl.txt`, obligationText(text));
  writeFileSync(`${out}/${name}.bind.txt`, bindingText(text));
  // The operands, NUL-separated, for experiments/issue-1562/rust-obligation-twin.sh.
  writeFileSync(`${out}/${name}.operands`, boundOutputLiterals(text).join('\0'));
};
for (const n of [4031, 4034, 4035]) write(n, readFileSync(`data/benchmarks/fixtures/issue-1562/hive-mind-${n}.txt`, 'utf8'));
for (const issue of JSON.parse(readFileSync('rust/tests/fixtures/issue-1364-hello-world-issues.json', 'utf8'))) {
  write(issue.title.split(' ').at(-1), `${issue.title}\n\n${issue.body}`);
}
// The work item of rust/tests/web/issue-1155-read-validation.test.mjs: its
// only output literal is a code span.
write('issue-1155', 'Implement Hello World in Rust\n\n## Task\nPlease implement a "Hello World" program in Rust.\n\n## Requirements\n1. Create a file with the appropriate extension for Rust\n2. The program should print exactly: `Hello, World!`\n3. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n');
write('python-goodbye', 'Write a Python program that prints exactly: `Goodbye, moon`');
process.exit(0);
