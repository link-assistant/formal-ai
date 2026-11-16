// Builds rust/tests/fixtures/issue-1166/acceptance.json: the case definitions
// below with the expected values the JavaScript reader and checker produce.
// Review the printed summary before committing; both roots' tests pin these.
//   node experiments/issue-1166/build-acceptance-fixture.mjs
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { specifyTask, checkTask } = await import('../../js/agentic/task_acceptance.mjs');
const fixtures = new URL('../../rust/tests/fixtures/issue-1166/', import.meta.url);
const read = (name) => readFileSync(new URL(name, fixtures), 'utf8');
const P = 'Write a Python program. ';
const tasks = {
  once: 'Write a Python program that prints `Hi`.',
  restated: 'Write a Python program that prints `Hi`. The output must be `Hi`.',
  mentioned_twice: 'Write a Python program that prints `Hi` and `Hi`.',
  twice: 'Write a Python program that prints `Hi` twice.',
  exact_backslash_unicode: 'Write a Python program that prints `C:\\temp\\ünï "q"` twice.',
  exact_cyrillic_cjk: 'Write a Python program that prints "Привет, 世界!".',
  escaped_quote: 'Write a Python program that prints "a\\"b".',
  greater: `${P}When x is greater than y, print "big", otherwise print "small".`,
  reversed_less: `${P}Print "big" if y is less than x, else print "small".`,
  symbol_greater: `${P}Print "big" when x > y; otherwise print "small".`,
  less: `${P}When x is less than y, print "big", otherwise print "small".`,
  conjunction: `${P}Print "yes" when a is less than b and b is less than c.`,
  disjunction: `${P}Print "yes" when a is less than b or b is less than c.`,
  positive: `${P}Print "yes" when a is less than b.`,
  unless: `${P}Print "yes" unless a is less than b.`,
  not_less: `${P}Print "yes" when a is not less than b.`,
  unsupported_relation: `${P}Print "yes" when a is roughly bigger than b.`,
};
const specificationCases = Object.entries(tasks).map(([id, task]) => {
  const specification = specifyTask(task);
  return { id, task, output: specification.output, meaning: specification.meaning };
});
const paraphraseGroups = [
  { id: 'repeated_mentions_are_one_line', cases: ['once', 'restated', 'mentioned_twice'] },
  { id: 'reversed_and_symbolic_comparisons', cases: ['greater', 'reversed_less', 'symbol_greater'] },
  { id: 'unless_is_a_negated_condition', cases: ['unless', 'not_less'] },
];
const distinctGroups = [
  { id: 'explicit_count_differs_from_one_line', cases: ['once', 'twice'] },
  { id: 'reversed_operands_differ', cases: ['greater', 'less'] },
  { id: 'conjunction_differs_from_disjunction', cases: ['conjunction', 'disjunction'] },
  { id: 'negation_differs_from_the_condition', cases: ['positive', 'unless'] },
];
const multilingual = ['en', 'paraphrase-en', 'es', 'ru', 'zh', 'hi'].map((tag) => `hello-world-kotlin-${tag}.txt`);
const meanings = new Set(multilingual.map((name) => specifyTask(read(name)).meaning));
if (meanings.size !== 1) throw new Error(`multilingual meanings differ: ${[...meanings].join('\n')}`);

const program = '// Build: kotlinc Main.kt -include-runtime -d Main.jar\n// Run: java -jar Main.jar\n\n// Prints the greeting.\nfun main() {\n    println("Hello, World!")\n}\n';
const workflow = 'name: Test Hello World\non:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: fwilhe2/setup-kotlin@v1\n      - run: kotlinc Main.kt -include-runtime -d Main.jar\n      - run: test "$(java -jar Main.jar)" = "Hello, World!"\n';
const good = { program_path: 'Main.kt', files: [{ path: 'Main.kt', source: program }, { path: 'ci/test-hello-world.yml', source: workflow }], observations: [{ args: [], stdout: 'Hello, World!\n' }] };
const candidateCases = [
  { id: 'issue_1450_without_an_observed_conclusion', candidate: good },
  { id: 'issue_1450_green', candidate: { ...good, workflow_conclusion: 'success' } },
  { id: 'issue_1450_red', candidate: { ...good, workflow_conclusion: 'failure' } },
  { id: 'issue_1450_without_comments', candidate: { ...good, files: [{ path: 'Main.kt', source: 'fun main() {\n    println("Hello, World!")\n}\n' }, good.files[1]] } },
  { id: 'issue_1450_workflow_under_github', candidate: { ...good, files: [good.files[0], { path: '.github/workflows/test-hello-world.yml', source: workflow }] } },
  { id: 'issue_1450_without_pull_request_trigger', candidate: { ...good, files: [good.files[0], { path: 'ci/test-hello-world.yml', source: workflow.replace('  pull_request:\n', '') }] } },
  { id: 'issue_1450_wrong_output', candidate: { ...good, observations: [{ args: [], stdout: 'Hello World\n' }] } },
  { id: 'issue_1450_other_task', candidate: { ...good, task_identity: 'task_0' } },
].map((entry) => {
  const specification = specifyTask(read('issue-1450-body.md'));
  const report = checkTask(specification, { task_identity: specification.task_identity, ...entry.candidate });
  return { ...entry, task_file: 'issue-1450-body.md', complete: report.complete, verdicts: report.verdicts.map((v) => `${v.span[0]}-${v.span[1]} ${v.verdict} ${v.justification}`) };
});

// Independently executed Python candidates: each row's argv is run and only stdout is observed.
const executionDefinitions = [
  { id: 'shared_misunderstanding_of_twice', task: 'twice', program: 'print("Hi")\n', generated_test: 'import subprocess, sys\nassert subprocess.run([sys.executable, sys.argv[1]], capture_output=True, text=True).stdout == "Hi\\n"\n' },
  { id: 'twice_honoured', task: 'twice', program: 'print("Hi")\nprint("Hi")\n' },
  { id: 'restated_line_printed_once', task: 'restated', program: 'print("Hi")\n' },
  { id: 'exact_backslash_unicode_kept', task: 'exact_backslash_unicode', program: 'for _ in range(2):\n    print(r\'C:\\temp\\ünï "q"\')\n' },
  { id: 'backslash_escape_misread', task: 'exact_backslash_unicode', program: 'for _ in range(2):\n    print(\'C:\\temp\\ünï "q"\')\n' },
  { id: 'reversed_comparison_rejected', task: 'greater', program: 'import sys\nx, y = int(sys.argv[1]), int(sys.argv[2])\nprint("big" if x < y else "small")\n' },
  { id: 'comparison_honoured', task: 'reversed_less', program: 'import sys\nx, y = int(sys.argv[1]), int(sys.argv[2])\nprint("big" if x > y else "small")\n' },
  { id: 'disjunction_for_conjunction_rejected', task: 'conjunction', program: 'import sys\na, b, c = map(int, sys.argv[1:4])\nif a < b or b < c:\n    print("yes")\n' },
  { id: 'conjunction_honoured', task: 'conjunction', program: 'import sys\na, b, c = map(int, sys.argv[1:4])\nif a < b and b < c:\n    print("yes")\n' },
  { id: 'dropped_negation_rejected', task: 'unless', program: 'import sys\na, b = int(sys.argv[1]), int(sys.argv[2])\nif a < b:\n    print("yes")\n' },
  { id: 'negation_honoured', task: 'unless', program: 'import sys\na, b = int(sys.argv[1]), int(sys.argv[2])\nif not a < b:\n    print("yes")\n' },
  { id: 'unsupported_grammar_is_a_gap', task: 'unsupported_relation', program: 'print("yes")\n' },
];
const directory = mkdtempSync(join(tmpdir(), 'issue-1166-fixture-'));
const executionCases = executionDefinitions.map((entry) => {
  const specification = specifyTask(tasks[entry.task]);
  const path = join(directory, 'main.py');
  writeFileSync(path, entry.program);
  if (entry.generated_test) {
    writeFileSync(join(directory, 'test_main.py'), entry.generated_test);
    execFileSync('python3', [join(directory, 'test_main.py'), path]);
  }
  const rows = specification.output.table ? specification.output.table.rows.map((row) => row.args) : [[]];
  const observations = rows.map((args) => ({ args, stdout: execFileSync('python3', [path, ...args], { encoding: 'utf8' }) }));
  const report = checkTask(specification, { task_identity: specification.task_identity, program_path: 'main.py', files: [{ path: 'main.py', source: entry.program }], observations });
  return { ...entry, complete: report.complete, verdicts: report.verdicts.map((v) => `${v.span[0]}-${v.span[1]} ${v.verdict} ${v.justification}`) };
});
rmSync(directory, { recursive: true, force: true });
const fixture = { specificationCases, paraphraseGroups, distinctGroups, multilingual, candidateCases, executionCases };
writeFileSync(new URL('acceptance.json', fixtures), `${JSON.stringify(fixture, null, 2)}\n`);
for (const entry of specificationCases) console.log(entry.id, JSON.stringify(entry.output).slice(0, 120));
for (const group of paraphraseGroups) console.log('same', group.id, new Set(group.cases.map((id) => tasks[id]).map((t) => specifyTask(t).meaning)).size === 1);
for (const group of distinctGroups) console.log('distinct', group.id, new Set(group.cases.map((id) => tasks[id]).map((t) => specifyTask(t).meaning)).size === group.cases.length);
for (const entry of [...candidateCases, ...executionCases]) console.log(entry.id, entry.complete, entry.verdicts.filter((line) => !/ (discharged|inapplicable) /u.test(line) && !/judgment_required/u.test(line)).join(' | '));
