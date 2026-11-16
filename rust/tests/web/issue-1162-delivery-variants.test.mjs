// Issue #1162 acceptance item 5: the Hello World delivery recipe answers each
// predeclared variant of the request from the request itself, never from a
// completed program. Changed and Unicode strings, shell/printf escaping,
// alternate file names, paraphrased wording, repeated references to one line
// (#1156 coreference), an explicit print-twice task, and the literal
// ci/test-hello-world.yml path (#1450) are each checked on their own.
import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { boundOutputLiterals } from '../../../js/agentic/crate/intent_formalization_obligations.mjs';
import { attach, requestedIn } from '../../../js/agentic/ci_workflow.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const base = fs.readFileSync('rust/tests/fixtures/issue-1162/hello-world-kotlin-issue.md', 'utf8');

function delivery(issue) {
  const recipe = programContractAnswer(issue)?.execution_recipe;
  assert.ok(recipe, `no recipe for ${JSON.stringify(issue.slice(0, 80))}`);
  if (requestedIn(issue)) attach(recipe, issue);
  const verify = recipe.supporting_files.find(file => file.path === 'tests/verify-output.sh');
  assert.ok(verify, 'verify script');
  const workflows = recipe.supporting_files.filter(file => /\.ya?ml$/u.test(file.path)).map(file => file.path);
  return { recipe, verify: verify.source, workflows };
}

test('changed and Unicode output strings flow into the program and the verify script', () => {
  for (const literal of ['Hello, Formal AI!', 'Привет, мир! 🌍', '你好，世界']) {
    const { recipe, verify, workflows } = delivery(base.replaceAll('Hello, World!', literal));
    assert.equal(recipe.path, 'Main.kt', literal);
    assert.ok(recipe.source.includes(`println("${literal}")`), literal);
    assert.ok(verify.includes(`printf '%s\\n' '${literal}' > "$verification_dir/expected"`), literal);
    assert.deepEqual(workflows, ['.github/workflows/test-hello-world.yml'], literal);
  }
});

test('quotes, dollar signs, percent signs and backslashes are escaped for the language and the shell', () => {
  const literal = `It's 100% $HOME \\ "q"`;
  assert.deepEqual(boundOutputLiterals(`Write a Kotlin program that outputs \`${literal}\`.`), [literal]);
  const { recipe, verify } = delivery(`Write a Kotlin program that outputs \`${literal}\`.`);
  assert.ok(recipe.source.includes('println("It\'s 100% \\$HOME \\\\ \\"q\\"")'), recipe.source);
  assert.ok(verify.includes(`printf '%s\\n' 'It'"'"'s 100% $HOME \\ "q"'`) || verify.includes(`printf '%s\\n' 'It'\\''s 100% $HOME \\ "q"'`), verify);
});

test('an alternate file name named in the request is the program path', () => {
  const { recipe } = delivery(base.replace('Create a file with the appropriate extension for Kotlin', 'Create the file `Greeting.kt`'));
  assert.equal(recipe.path, 'Greeting.kt');
});

test('a paraphrased request with "outputs" still yields a verified recipe', () => {
  const issue = 'Write a Kotlin program that outputs exactly `Hello, World!` and add a GitHub Actions workflow that runs it on every push and pull request and checks the output.';
  assert.deepEqual(boundOutputLiterals(issue), ['Hello, World!']);
  const { recipe, verify, workflows } = delivery(issue);
  assert.ok(recipe.source.includes('println("Hello, World!")'));
  assert.ok(verify.includes(`printf '%s\\n' 'Hello, World!'`));
  assert.equal(workflows.length, 1);
});

test('repeated references to one desired line stay one line (#1156 coreference)', () => {
  for (const issue of [
    `${base}\nAgain: the program must print exactly "Hello, World!". The output is "Hello, World!".\n`,
    'Write a Kotlin program that prints `Hello, World!`. The program must print "Hello, World!".',
    'Write a Kotlin program that prints `Hello, World!`. Check again that it prints `Hello, World!`.',
    'Write a Kotlin program that prints `Hello, World!` and test it against the expected output.',
  ]) {
    assert.deepEqual(boundOutputLiterals(issue), ['Hello, World!'], issue);
    assert.ok(delivery(issue).recipe.source.includes('println("Hello, World!")'), issue);
  }
});

test('an explicit print-twice task produces the line twice, and the count is read from the grammar', () => {
  for (const [issue, count] of [
    ['Write a Kotlin program that prints `Hello, World!` twice.', 2],
    ['Write a Kotlin program that prints `Hello, World!` three times.', 3],
    ['Write a Kotlin program that first prints `Hello, World!` and then prints `Hello, World!` again.', 2],
    ['Напиши программу на Kotlin, которая выводит `Привет` дважды.', 2],
  ]) {
    const outputs = boundOutputLiterals(issue);
    assert.equal(outputs.length, count, issue);
    assert.equal(new Set(outputs).size, 1, issue);
    const { recipe } = delivery(issue);
    assert.ok(recipe.source.includes(`println("${Array(count).fill(outputs[0]).join('\\n')}")`), recipe.source);
  }
});

test('distinct sequenced literals keep their order', () => {
  assert.deepEqual(boundOutputLiterals('Write a Kotlin program that first prints `A` and then prints `B`.'), ['A', 'B']);
});

test('the literal ci/test-hello-world.yml path (#1450) stays separate from .github/workflows', () => {
  for (const language of ['kotlin', 'scala', 'rust']) {
    const issue = fs.readFileSync(`rust/tests/fixtures/generated-e2e/hello-${language}.md`, 'utf8');
    assert.deepEqual(delivery(issue).workflows, ['ci/test-hello-world.yml'], language);
  }
  assert.deepEqual(delivery(base).workflows, ['.github/workflows/test-hello-world.yml']);
});

test('the Scala and Rust variants keep their own language and print once', () => {
  for (const [language, path, line] of [['Scala', 'Main.scala', 'println("Hello, World!")'], ['Rust', 'main.rs', 'println!("{}", "Hello, World!");']]) {
    const { recipe, workflows } = delivery(base.replaceAll('Kotlin', language));
    assert.equal(recipe.path, path, language);
    assert.equal(recipe.source.split('\n').filter(text => text.includes(line)).length, 1, recipe.source);
    assert.deepEqual(workflows, ['.github/workflows/test-hello-world.yml'], language);
  }
});
