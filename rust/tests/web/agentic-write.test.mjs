// Parity tests for the write-side agentic modules under js/agentic/ (agent
// "write"): expectations are copied from the Rust tests named beside each case.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { PLAN_PATH, composeGeneralChangePlan, planLinksNotation } from '../../../js/agentic/general_planner.mjs';
import { gapAnswer, nextStep, obligations } from '../../../js/agentic/task_obligations.mjs';
import { planStructuredEditStep } from '../../../js/agentic/structured_edit.mjs';
import { explicitStdout, programContractAnswer } from '../../../js/agentic/crate/coding_program_contract.mjs';
import { baselineVersionSet, fillWorkflowVersions } from '../../../js/agentic/crate/version_resolution.mjs';
import { parseSubstitutionQuery, renderSubstitutionQuery, substitutionEffect } from '../../../js/agentic/crate/links_substitution_query.mjs';
import { executeRewrite, rewriteProgram, rewriteRule } from '../../../js/agentic/crate/normal_markov.mjs';
import { planSymbolicCommandReroute } from '../../../js/agentic/command_reroute.mjs';
import { attach } from '../../../js/agentic/ci_workflow.mjs';
import { formalizeDiagnostic } from '../../../js/agentic/repair_loop.mjs';
import { sourceFromReadResult } from '../../../js/agentic/code_artifact.mjs';
import { executeScopedWorkspaceRewrite, RewriteScope } from '../../../js/agentic/crate/workspace_change_learning.mjs';
import { splitOnceCheckable } from '../../../js/agentic/crate/task_decomposition.mjs';
import { readText, parseLino } from '../../../js/agentic/host.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const EN_TASK = 'Create file notes/general-demo.txt containing planner fallback works';
const EN_TASK_ALT = 'Write file artifacts/unseen-case.md with text capability composed plan';
const RU_TASK = 'Создай файл output/пример.txt с текстом общий план работает';

describe('general_planner (rust/tests/unit/agentic-coding/agentic_general_planner.rs)', () => {
  test('composes capability steps and verification (line 20)', () => {
    const plan = composeGeneralChangePlan(EN_TASK);
    assert.ok(plan.steps.length >= 3);
    assert.ok(plan.steps.every((step) => step.expected_evidence !== ''));
    assert.equal(plan.verification_command, 'cat notes/general-demo.txt');
    assert.deepEqual(composeGeneralChangePlan(EN_TASK), plan);
  });

  test('accepts three unpinned phrasings in two languages (line 40)', () => {
    for (const request of [EN_TASK, EN_TASK_ALT, RU_TASK]) {
      const plan = composeGeneralChangePlan(request);
      assert.ok(plan && plan.goal !== '', request);
      assert.match(planLinksNotation(plan), /expected_evidence/u);
    }
  });

  test('rejects unsafe, ambiguous, dot-directory and non-referential requests (lines 49, 55, 177)', () => {
    for (const request of [
      'Create file ../escape.txt containing no',
      'Please improve the repository',
      'When you create debug or example scripts while fixing an issue, keep them in ./examples and/or ./experiments. Continue with the following investigation details and solve the issue completely.',
      'save it to handler-precedence-learning-report.lino',
      'please write this to notes/output.txt',
    ]) assert.equal(composeGeneralChangePlan(request), null, request);
    assert.equal(composeGeneralChangePlan('write to notes/quote.txt saying to be or not to be').content, 'to be or not to be');
  });

  test('keeps exact multi-line and backticked payloads (lines 228, 244, 269)', () => {
    const lino = 'substitution_rules\n  id "learned_program_plan_rules"\n  rule "reverse_sort"';
    assert.equal(composeGeneralChangePlan(`Create file data/seed/learned-program-rules.lino containing\n${lino}`).content, lino);
    const backticked = 'coding_discovery_recipe\n  coding_discovery_step_verify';
    const plan = composeGeneralChangePlan(`Create file data/meta/coding-discovery-recipe.lino with \`${backticked}\``);
    assert.equal(plan.target, 'data/meta/coding-discovery-recipe.lino');
    assert.equal(plan.content, backticked);
    const payload = 'prefix rename X to Y suffix';
    assert.equal(composeGeneralChangePlan(`Create file issue_708_memory_program.rs with exactly this content:\n${payload}`).content, payload);
  });

  test('command stdout requests run the command (line 306)', () => {
    const plan = composeGeneralChangePlan("Run 'printf learned-output' and write its exact stdout to reports/learned.txt");
    assert.equal(plan.target, 'reports/learned.txt');
    assert.equal(plan.content, '');
    assert.deepEqual(plan.steps.map((step) => step.capability), ['write', 'run', 'run']);
    assert.equal(plan.steps[1].command, "printf learned-output > 'reports/learned.txt'");
    assert.ok(PLAN_PATH.length > 0);
  });
});

describe('task_obligations (rust/tests/unit/agentic-coding/issue_1099_multiple_obligations.rs)', () => {
  const fileTargets = (found) => found.filter((node) => node.expectation.kind === 'file_bytes').map((node) => node.expectation.path);

  test('two named files are two obligations, in order (line 93)', () => {
    const found = obligations('Two files. First, create file notes/attribution.md containing Gemfile.lock. Second, create file changelog/fragment.md containing bump patch.');
    assert.deepEqual(fileTargets(found), ['notes/attribution.md', 'changelog/fragment.md']);
  });

  test('a cue inside a clause opens no obligation (line 108)', () => {
    const found = obligations('First, create file notes/a.txt containing alpha, whose first line is exactly alpha. Second, create file notes/b.txt containing beta.');
    assert.deepEqual(fileTargets(found), ['notes/a.txt', 'notes/b.txt']);
  });

  test('a single artifact request is unchanged; Russian is recognised (lines 139, 152)', () => {
    assert.equal(obligations(EN_TASK), null);
    assert.equal(obligations(EN_TASK_ALT), null);
    const found = obligations('Сначала создай файл notes/a.txt с текстом альфа. Затем создай файл notes/b.txt с текстом бета.');
    assert.deepEqual(fileTargets(found), ['notes/a.txt', 'notes/b.txt']);
  });

  test('gap answers carry the clause and its UTF-8 byte span', () => {
    const record = gapAnswer('obligation_x', 'описание', [3, 19], 'no_artifact_in_clause');
    assert.equal(record, 'obligation_x\n  record_type "obligation_gap"\n  clause "описание"\n  span_start "3"\n  span_end "19"\n  reason "no_artifact_in_clause"');
  });

  test('the first open file obligation is observed first, with its byte span (line 121)', () => {
    const task = 'Two files. First, create file notes/attribution.md containing Gemfile.lock. Second, create file changelog/fragment.md containing bump patch.';
    const next = nextStep(task, [{ role: 'user', content: task }]);
    assert.equal(next.kind, 'observe');
    assert.equal(next.node.clause, 'First, create file notes/attribution.md containing Gemfile.lock.');
    assert.deepEqual(next.node.span, [11, 75]);
    assert.equal(next.node.expectation.path, 'notes/attribution.md');
  });

  test('split_once_checkable declines an atomic clause', () => {
    assert.deepEqual(splitOnceCheckable('Think about it'), []);
  });
});

describe('structured_edit (rust/tests/unit/agentic-coding/issue_1069_structural_edit.rs)', () => {
  const TOOLS = ['read_file', 'write_file', 'run_shell_command'];
  const argument = (plan, key) => JSON.parse(plan.calls[0].arguments)[key];

  function writtenSource(prompt, path, source) {
    const messages = [{ role: 'user', content: prompt }];
    const read = planStructuredEditStep(prompt, messages, TOOLS);
    assert.equal(read.kind, 'tool_calls');
    assert.equal(argument(read, 'path'), path);
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id: 'call_1', type: 'function', function: { name: read.calls[0].tool, arguments: read.calls[0].arguments } }] });
    messages.push({ role: 'tool', tool_call_id: 'call_1', name: read.calls[0].tool, content: source });
    const write = planStructuredEditStep(prompt, messages, TOOLS);
    assert.equal(argument(write, 'path'), path);
    return argument(write, 'content');
  }

  const IGNORED_MATCHES = 'fn ignored(path: &Path, root: &Path) -> bool {\n    path.strip_prefix(root).is_ok_and(|relative| {\n        relative.components().next().is_some_and(|part| {\n            matches!(\n                part.as_os_str().to_str(),\n                Some(".git" | "target" | ".formal-ai" | ".formal-ai-orchestration")\n            )\n        })\n    })\n}\n';

  test('inserts into a matches! alternation (line 87)', () => {
    const updated = writtenSource(
      'In this repository, edit the existing tracked file src/orchestration/workspace.rs. It has a private function `ignored` whose `matches!` arm lists the directory names that are skipped when the workspace is walked: ".git", "target", ".formal-ai" and ".formal-ai-orchestration". Add "node_modules" to that same list so dependency directories are skipped too.',
      'src/orchestration/workspace.rs',
      IGNORED_MATCHES,
    );
    assert.ok(updated.includes('Some(".git" | "target" | ".formal-ai" | ".formal-ai-orchestration" | "node_modules")'), updated);
    assert.equal(updated.split('node_modules').length - 1, 1);
  });

  test('keeps the named array insertion (line 113)', () => {
    const updated = writtenSource('Write "uv.lock" into the LOCKFILE_NAMES array in scripts/metric.rs', 'scripts/metric.rs', 'const LOCKFILE_NAMES: &[&str] = &["Cargo.lock", "bun.lock"];\n');
    assert.ok(updated.includes('"Cargo.lock", "bun.lock", "uv.lock"'), updated);
  });

  test('finds an unnamed list by its members; parenthesised lists (lines 129, 149)', () => {
    const updated = writtenSource('Edit config.rs so the list holding "alpha" and "beta" also holds "gamma".', 'config.rs', 'let modes = vec!["alpha", "beta"];\nlet other = vec!["delta"];\n');
    assert.ok(updated.includes('vec!["alpha", "beta", "gamma"]'), updated);
    assert.ok(updated.includes('vec!["delta"]'));
    const pair = writtenSource('Edit tuple.rs: add "middle" to the group that already contains "left" and "right".', 'tuple.rs', 'const PAIR: (&str, &str) = ("left", "right");\n');
    assert.ok(pair.includes('("left", "right", "middle")'), pair);
  });

  test('insertion is idempotent: the read that shows the value is the observation (line 165)', () => {
    const prompt = 'Edit names.rs: add "b" to the NAMES array that already contains "a".';
    const messages = [{ role: 'user', content: prompt }];
    const read = planStructuredEditStep(prompt, messages, TOOLS);
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id: 'call_1', type: 'function', function: { name: read.calls[0].tool, arguments: read.calls[0].arguments } }] });
    messages.push({ role: 'tool', tool_call_id: 'call_1', name: read.calls[0].tool, content: 'const NAMES: &[&str] = &["a", "b"];\n' });
    const answer = planStructuredEditStep(prompt, messages, TOOLS);
    assert.equal(answer.kind, 'final');
    assert.equal(answer.answer, '`names.rs` already lists "b", "a"; nothing needed to change.');
  });
});

describe('links_substitution_query (rust/tests/unit/issue_715_links_substitution_query.rs)', () => {
  const STEPS = 64;
  test('sides map to create, delete, update and read (lines 26-79)', () => {
    const create = parseSubstitutionQuery('() ((terminal: "hello"))', STEPS);
    assert.deepEqual(create.rules, [rewriteRule('', 'hello', true)]);
    assert.equal(substitutionEffect(create.rules[0]), 'create');
    assert.equal(executeRewrite(create, '').output, 'hello');
    const diverging = executeRewrite(parseSubstitutionQuery('() (("ha"))', STEPS), '');
    assert.equal(diverging.halt.kind, 'step_limit');
    assert.equal(diverging.output, 'ha'.repeat(STEPS));
    assert.equal(executeRewrite(parseSubstitutionQuery('(("hello ")) ()', STEPS), 'hello world').output, 'world');
    assert.equal(executeRewrite(parseSubstitutionQuery('(("old")) (("new"))', STEPS), 'an old thing').output, 'an new thing');
    assert.equal(substitutionEffect(parseSubstitutionQuery('(("x")) (("x"))', STEPS).rules[0]), 'read');
    assert.equal(executeRewrite(parseSubstitutionQuery('(("a") ("b")) (("A") ("B"))', STEPS), 'ba').output, 'BA');
    assert.deepEqual(parseSubstitutionQuery('(("say \\"hi\\"")) ((""))', STEPS).rules, [rewriteRule('say "hi"', '')]);
  });

  test('rendering round-trips and matches the canonical shorthands (lines 141, 167)', () => {
    for (const rules of [
      [rewriteRule('', 'created')], [rewriteRule('deleted', '')], [rewriteRule('old', 'new', true)],
      [rewriteRule('read', 'read')], [rewriteRule('gone', '', true)], [rewriteRule('', 'made', true)],
      [rewriteRule('a', 'A'), rewriteRule('b', 'B', true)], [rewriteRule('', '')], [],
    ]) {
      const program = rewriteProgram(rules, STEPS);
      assert.deepEqual(parseSubstitutionQuery(renderSubstitutionQuery(program), STEPS), program);
    }
    const render = (rules) => renderSubstitutionQuery(rewriteProgram(rules, STEPS));
    assert.equal(render([rewriteRule('', 'a')]), '() (("a"))');
    assert.equal(render([rewriteRule('a', '')]), '(("a")) ()');
    assert.equal(render([rewriteRule('a', 'b', true)]), '(("a")) ((terminal: "b"))');
    assert.equal(render([]), '() ()');
    assert.equal(render([rewriteRule('', '')]), '(("")) ((""))');
  });
});

describe('program contract, CI workflow and command reroute', () => {
  test('explicit_stdout reads every fixture case (rust/src/coding/program_contract.rs tests)', () => {
    const fixture = parseLino(readText('rust/tests/fixtures/program-contract/explicit-stdout.lino'));
    const cases = fixture.children.filter((node) => node.name === 'case');
    assert.equal(cases.length, 2);
    for (const entry of cases) {
      const child = (name) => entry.children.find((node) => node.name === name).value;
      assert.equal(explicitStdout(child('prompt')), child('expected'), entry.value);
    }
  });

  test('a Kotlin stdout contract becomes a verified recipe the reroute writes first', () => {
    const answer = programContractAnswer('Write a Kotlin program that prints "Hello, World!" to stdout.');
    const recipe = answer.execution_recipe;
    assert.equal(recipe.path, 'Main.kt');
    assert.deepEqual(recipe.commands, ['kotlinc Main.kt -include-runtime -d Main.jar', 'sh tests/verify-output.sh']);
    assert.match(recipe.source, /println\("Hello, World!"\)/u);
    attach(recipe);
    assert.equal(recipe.supporting_files[1].path, '.github/workflows/Main.yml');
    // Every step opens its own line: no fragment is glued onto the last line
    // of the one before it (checkout, setup, then each command).
    for (const line of recipe.supporting_files[1].source.split('\n')) {
      const markers = (line.match(/- (uses|run):/gu) ?? []).length;
      assert.ok(markers === 0 || (markers === 1 && line.trimStart().startsWith('-')), `glued step line: ${line}`);
    }
    const plan = planSymbolicCommandReroute([{ role: 'user', content: 'hi' }], ['write', 'bash'], answer);
    assert.equal(plan.calls[0].tool, 'write');
    assert.equal(JSON.parse(plan.calls[0].arguments).path, 'Main.kt');
  });

  test('fill_workflow_versions rewrites stale pins (rust/tests/unit/issue_1168_latest_versions.rs line 354)', () => {
    const versions = baselineVersionSet();
    assert.equal(
      fillWorkflowVersions("      - uses: actions/setup-java@b6effb05e454b25005698d916606bdc6ffcbf961\n        with:\n          distribution: temurin\n          java-version: '21'\n      - uses: fwilhe2/setup-kotlin@51a059ff08b95e2b83aa952b5b46b696d5b615a0\n        with:\n          version: '2.3.10'\n", versions),
      "      - uses: actions/setup-java@de7274f081f381c8f8158605e0321c36c376e2e6  # v6.0.1\n        with:\n          distribution: temurin\n          java-version: '25'\n      - uses: fwilhe2/setup-kotlin@ee9692514da313706b193d808526812102a344e4  # v2.0\n        with:\n          version: '2.4.20'",
    );
    assert.equal(fillWorkflowVersions('      - uses: actions/checkout@{checkout_ref} # {checkout_tag}\n', versions), '      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1  # v7.0.1');
    assert.equal(
      fillWorkflowVersions("      - uses: actions/setup-python@ece7cb06caefa5fff74198d8649806c4678c61a1\n        with:\n          python-version: '3.14'\n", versions),
      "      - uses: actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97  # v7.0.0\n        with:\n          python-version: '3.14.7'",
    );
    const unrelated = "      - uses: some/other-action@v1\n        with:\n          version: '9.9.9'\n";
    assert.equal(fillWorkflowVersions(unrelated, versions), unrelated.replace(/\n+$/u, ''));
  });
});

describe('workspace rewrites, read results and repair diagnostics', () => {
  test('a word-scoped rewrite leaves longer identifiers alone', () => {
    const result = executeScopedWorkspaceRewrite('let total = total_count + total;', 'total', 'sum', RewriteScope.Word);
    assert.equal(result.ok.output, 'let sum = total_count + sum;');
    assert.equal(executeScopedWorkspaceRewrite('abc', 'x', 'y', RewriteScope.Substring).error, 'workspace_rewrite_no_match');
  });

  test('sourceFromReadResult decodes numbered read-tool output', () => {
    assert.equal(sourceFromReadResult('<file>\n00001| fn main() {\n00002| }\n\n(End of file - total 2 lines)\n</file>'), 'fn main() {\n}');
    assert.equal(sourceFromReadResult('<content>\n1: a\n2: b\n(End of file - total 2 lines)\n</content>'), 'a\nb\n');
  });

  test('formalize_diagnostic reads file, line, code and message (rust/tests/unit/agentic-coding/issue_1185_error_repair_loop.rs)', () => {
    assert.deepEqual(formalizeDiagnostic('rust', 'finished in 0.3s\nall good'), []);
    const [rust] = formalizeDiagnostic('rust', 'error[E0308]: mismatched types\n --> src/main.rs:6:33\n');
    assert.deepEqual([rust.file, rust.line, rust.code, rust.message], ['src/main.rs', 6, 'E0308', 'mismatched types']);
    // A pattern that names its own location binds it; Kotlin has no code.
    const [kotlin] = formalizeDiagnostic('kotlin', 'main.kt:3:5 error: unresolved reference: greeting');
    assert.deepEqual([kotlin.file, kotlin.line, kotlin.code, kotlin.message], ['main.kt', 3, null, 'unresolved reference: greeting']);
    // The traceback's innermost frame binds, not the outermost one.
    const traceback = 'Traceback (most recent call last):\n  File "main.py", line 4, in <module>\n    greet()\n  File "main.py", line 2, in greet\n    return gretting("hi")\nNameError: name \'gretting\' is not defined\n';
    const [python] = formalizeDiagnostic('python', traceback);
    assert.deepEqual([python.file, python.line, python.code, python.message], ['main.py', 2, null, "name 'gretting' is not defined"]);
  });
});
