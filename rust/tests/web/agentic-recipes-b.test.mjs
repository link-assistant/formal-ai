// The JavaScript port of the agentic recipes in rust/src/agentic_coding/
// (algorithm_learning, procedure, learning_report and its reports, self_ast,
// dreaming_audit, google_trends_*, question_catalog). Expectations mirror
// rust/tests/unit/issue_531_algorithm_discovery.rs,
// rust/tests/unit/specification/arbitrary_skill_compilation.rs,
// rust/tests/unit/agentic-coding/issue_686_agent_cli.rs, issue_657/709 learning reports,
// issue_538/540/527/498/499 recipes, and the committed Agent CLI sessions
// those Rust tests pin byte-for-byte (docs/case-studies/*/agent-cli-session-*.json).

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import * as algorithmLearning from '../../../js/agentic/algorithm_learning.mjs';
import * as procedure from '../../../js/agentic/procedure.mjs';
import * as learningReport from '../../../js/agentic/learning_report.mjs';
import * as associativeLearning from '../../../js/agentic/associative_learning.mjs';
import * as selfHostingLearning from '../../../js/agentic/learning_report/self_hosting_learning.mjs';
import * as searchFusionLearning from '../../../js/agentic/learning_report/search_fusion_learning.mjs';
import * as selfAst from '../../../js/agentic/self_ast.mjs';
import * as dreamingAudit from '../../../js/agentic/dreaming_audit.mjs';
import * as questionCatalog from '../../../js/agentic/question_catalog.mjs';
import * as googleTrendsCatalog from '../../../js/agentic/google_trends_catalog.mjs';
import * as googleTrendsLearning from '../../../js/agentic/google_trends_learning.mjs';
import {
  candidateFromLinksNotation, candidateLinksNotation, candidatesEqual, conformanceLinksNotation,
  discoverAlgorithms, validatedCandidates,
} from '../../../js/agentic/crate/algorithm_discovery.mjs';
import { MemoryStore, exportLinksNotationWithSchema, parseLinksNotation } from '../../../js/agentic/crate/memory.mjs';
import {
  PROCEDURE_CONFORMANCE_TRIGGER, compileProcedure, compileProcedureResult, conformanceLinksNotation as procedureConformance,
  procedureLinksNotation, restateSteps, stepArguments,
} from '../../../js/agentic/crate/skill_procedure.mjs';
import { artifactLinksNotation, extractCompiledProcedureArtifact, proceduresEqual } from '../../../js/agentic/crate/skill_procedure_artifact.mjs';
import { impulseIdFor } from '../../../js/agentic/crate/intent_formalization.mjs';

const ROOT = new URL('../../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, ROOT), 'utf8');
const session = (path) => JSON.parse(read(path));

const user = (content) => ({ role: 'user', content, tool_calls: [], tool_call_id: null, name: null });
function answerToolCall(messages, call, result) {
  const id = `call_${messages.length}`;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }], tool_call_id: null, name: null });
  messages.push({ role: 'tool', content: result, tool_calls: [], tool_call_id: id, name: call.tool });
}
const bytes = (text, [start, end]) => Buffer.from(text, 'utf8').subarray(start, end).toString('utf8');

before(async () => {
  await installNodeHost(new WorkerHost());
});

// ---- algorithm discovery / algorithm_learning (issue #531) -----------------

const step = (operation, args) => ({ operation, arguments: [...args].sort(([a], [b]) => (a < b ? -1 : 1)) });
const trace = (id, subject, format) => ({
  id,
  steps: [step('fetch', [['subject', subject]]), step('normalize', [['subject', subject], ['format', format]]), step('persist', [['subject', subject]])],
});
const threeRuns = () => discoverAlgorithms([trace('run-alpha', 'alpha', 'json'), trace('run-beta', 'beta', 'json'), trace('run-gamma', 'gamma', 'json')]);

test('repeated event sequences become a validated parameterized algorithm', () => {
  const run = threeRuns();
  assert.ok(run.associative_compression_lossless);
  assert.ok(run.associative_compression_steps > 0);
  const candidates = validatedCandidates(run);
  assert.equal(candidates.length, 1);
  const [candidate] = candidates;
  assert.equal(candidate.steps.length, 3);
  assert.deepEqual(candidate.support_trace_ids, ['run-alpha', 'run-beta']);
  assert.equal(candidate.held_out.length, 1);
  assert.ok(candidate.held_out[0].passed);
  const argument = (index, name) => candidate.steps[index].arguments.find(([key]) => key === name)[1];
  assert.deepEqual(argument(0, 'subject'), argument(1, 'subject'));
  assert.ok(argument(0, 'subject').parameter !== undefined);
  assert.deepEqual(argument(1, 'format'), { constant: 'json' });
});

test('held-out constant drift is preserved as a failed candidate', () => {
  const run = discoverAlgorithms([trace('run-alpha', 'alpha', 'json'), trace('run-beta', 'beta', 'json'), trace('run-gamma', 'gamma', 'xml')]);
  assert.equal(validatedCandidates(run).length, 0);
  const candidate = run.candidates.find((entry) => entry.steps.length === 3);
  assert.equal(candidate.held_out.length, 1);
  assert.ok(!candidate.held_out[0].passed);
});

test('artifact round trip and conformance do not implicitly approve execution', () => {
  const [candidate] = validatedCandidates(threeRuns());
  const document = candidateLinksNotation(candidate);
  assert.ok(candidatesEqual(candidateFromLinksNotation(document), candidate));
  const parameter = candidate.steps[0].arguments.find(([key]) => key === 'subject')[1].parameter;
  const conformance = conformanceLinksNotation(candidate, 'dry-trigger', new Map([[parameter, 'delta']]));
  assert.ok(conformance.includes('side_effects "false"'));
  assert.ok(conformance.includes('result "passed"'));
  assert.equal(candidateFromLinksNotation(document.replace(candidate.id, 'algorithm_tampered')), null);
  assert.equal(candidateFromLinksNotation(document.replace('test "run-gamma"', 'test "forged-run"')), null);
});

test('the memory reader round-trips the issue-531 observation document', () => {
  const text = read('data/benchmarks/issue-531-algorithm-traces.lino');
  const store = new MemoryStore();
  store.replaceFromLinksNotation(text);
  assert.ok(store.len() > 0);
  const again = new MemoryStore();
  again.replaceFromLinksNotation(store.exportLinksNotation());
  assert.equal(again.exportLinksNotation(), store.exportLinksNotation());
  assert.deepEqual(parseLinksNotation(store.exportLinksNotation()), store.events());
});

test('Agent CLI algorithm learning: write, discover, read back, conformance', () => {
  const store = new MemoryStore();
  store.replaceFromLinksNotation(read('data/benchmarks/issue-531-algorithm-traces.lino'));
  const memory = exportLinksNotationWithSchema(store.events(), 2);
  const prompt = `Derive any reusable execution algorithm from these recorded events and verify it.\n${memory}`;
  assert.equal(algorithmLearning.compileTask('Derive any reusable execution algorithm from these recorded events.'), null);
  const task = algorithmLearning.compileTask(prompt);
  assert.ok(task);
  const tools = ['write_file', 'run_command'];
  const messages = [user(prompt)];
  const write = algorithmLearning.planStep(messages, tools, task);
  assert.equal(write.calls[0].tool, 'write_file');
  assert.equal(JSON.parse(write.calls[0].arguments).path, algorithmLearning.OBSERVATIONS_PATH);
  answerToolCall(messages, write.calls[0], 'wrote algorithm-observations.lino');
  const learn = algorithmLearning.planStep(messages, tools, task);
  assert.ok(learn.calls[0].arguments.includes('formal-ai learn algorithms'));
  answerToolCall(messages, learn.calls[0], 'learned 1 held-out validated algorithm');
  const readback = algorithmLearning.planStep(messages, tools, task);
  assert.equal(JSON.parse(readback.calls[0].arguments).command, 'cat discovered-algorithms.lino');
  answerToolCall(messages, readback.calls[0], candidateLinksNotation(task.candidate));
  const conformance = algorithmLearning.planStep(messages, tools, task);
  assert.ok(conformance.calls[0].arguments.includes('formal-ai algorithm conformance'));
  answerToolCall(messages, conformance.calls[0], algorithmLearning.expectedConformance(task.candidate));
  const final = algorithmLearning.planStep(messages, tools, task);
  assert.equal(final.kind, 'final');
  assert.ok(final.answer.includes('status "conformance_passed"'));
  assert.ok(final.answer.includes('human_gated "true"'));
  assert.ok(final.answer.includes('associative_compression_lossless "true"'));
});

// ---- skill procedures / procedure (issue #674) -----------------------------

const ENGLISH_PROCEDURE = 'When I paste a link, fetch its title, translate it to Russian, save both, and reply with the translation.';
const RUSSIAN_PROCEDURE = 'Когда я вставляю ссылку, получи её заголовок, переведи его на русский, сохрани оба и ответь переводом.';
const HINDI_PROCEDURE = 'जब मैं लिंक भेजूँ, उसका शीर्षक लाओ, उसे रूसी में अनुवाद करो, दोनों सहेजो और अनुवाद के साथ जवाब दो।';
const CHINESE_PROCEDURE = '当我粘贴链接，获取标题，翻译成俄语，保存两者，然后用译文回复。';

test('an arbitrary four-step procedure compiles and restates its steps', () => {
  const compiled = compileProcedure(ENGLISH_PROCEDURE);
  assert.deepEqual(compiled.steps.map((entry) => entry.kind),
    ['skill_procedure_fetch', 'skill_procedure_translate', 'skill_procedure_store', 'skill_procedure_reply']);
  assert.deepEqual(stepArguments(compiled.steps[0]), ['skill_procedure_object_title']);
  assert.deepEqual(stepArguments(compiled.steps[1]), ['language_russian']);
  assert.deepEqual(compiled.trigger.objects, ['skill_procedure_object_link']);
  assert.ok(procedureConformance(compiled, PROCEDURE_CONFORMANCE_TRIGGER).includes(
    'answer "skill_procedure_reply(skill_procedure_store(skill_procedure_translate(skill_procedure_fetch(https://example.com/article))))"'));
  const restated = restateSteps(compiled);
  for (const entry of compiled.steps) {
    assert.ok(restated.includes(entry.kind));
    assert.equal(bytes(ENGLISH_PROCEDURE, entry.source_span), entry.source_text);
  }
});

test('the same procedure in every supported language compiles to the same skill links', () => {
  const english = compileProcedure(ENGLISH_PROCEDURE);
  for (const [language, description] of [['ru', RUSSIAN_PROCEDURE], ['hi', HINDI_PROCEDURE], ['zh', CHINESE_PROCEDURE]]) {
    const other = compileProcedure(description);
    assert.ok(other, language);
    assert.equal(procedureLinksNotation(other), procedureLinksNotation(english), language);
    assert.equal(other.id, english.id, language);
    assert.notEqual(restateSteps(other), restateSteps(english), language);
    for (const entry of other.steps) assert.equal(bytes(description, entry.source_span), entry.source_text, language);
  }
});

test('an uncompilable step reports a named gap and compiles nothing', () => {
  const withGap = 'When I paste a link, fetch its title, print it on my printer, and reply with the title.';
  const { error } = compileProcedureResult(withGap);
  assert.equal(error.kind, 'uncompilable_step');
  assert.equal(error.step, 'print it on my printer');
  assert.equal(bytes(withGap, error.span), error.step);
  assert.equal(procedure.compileTask(withGap), null);
  assert.equal(procedure.compileTask('What is the capital of France?'), null);
});

// The seed spells this gap name with doubled-quote escapes, which both Links
// Notation readers (rust/src/seed/parser.rs, js/seed_loader.js) collapse.
test('the gap name collapses the seed quote escape', () => {
  const { error } = compileProcedureResult('When I paste a link, fetch its title, print it on my printer, and reply with the title.');
  assert.equal(error.gap, 'no compiled capability for "print it on my printer"');
});

test('the compiler records every ordered requirement under the solver impulse', () => {
  const compiled = compileProcedure(ENGLISH_PROCEDURE);
  assert.equal(compiled.impulse_id, impulseIdFor(ENGLISH_PROCEDURE));
  assert.equal(compiled.requirements.length, 5);
  compiled.requirements.forEach((requirement, index) => {
    assert.equal(requirement.index, index + 1);
    assert.equal(bytes(ENGLISH_PROCEDURE, requirement.source_span), requirement.source_text);
  });
  assert.equal(compiled.trigger.requirement_id, compiled.requirements[0].id);
  compiled.steps.forEach((entry, index) => assert.equal(entry.requirement_id, compiled.requirements[index + 1].id));
});

test('the compiled artifact round-trips and rejects tampering', () => {
  const compiled = compileProcedure(ENGLISH_PROCEDURE);
  const artifact = artifactLinksNotation(compiled);
  assert.ok(proceduresEqual(extractCompiledProcedureArtifact(artifact), compiled));
  assert.equal(extractCompiledProcedureArtifact(artifact.replace(compiled.id, 'procedure_tampered')), null);
  assert.equal(extractCompiledProcedureArtifact(artifact.replace('fetch its title', 'fetch its titles')), null);
});

test('Agent CLI procedure: write, verify, execute, and return the same artifact', () => {
  const tools = ['write_file', 'run_command'];
  const messages = [user(ENGLISH_PROCEDURE)];
  const compiled = procedure.compileTask(ENGLISH_PROCEDURE);
  const write = procedure.planStep(messages, tools, compiled);
  assert.equal(write.calls.length, 1);
  assert.equal(write.calls[0].tool, 'write_file');
  const args = JSON.parse(write.calls[0].arguments);
  assert.equal(args.path, 'compiled-procedure.lino');
  assert.equal(args.content, artifactLinksNotation(compiled));
  answerToolCall(messages, write.calls[0], 'wrote compiled-procedure.lino');
  const verify = procedure.planStep(messages, tools, compiled);
  assert.equal(verify.calls[0].tool, 'run_command');
  assert.ok(verify.calls[0].arguments.includes('compiled-procedure.lino'));
  answerToolCall(messages, verify.calls[0], artifactLinksNotation(compiled));
  const execute = procedure.planStep(messages, tools, compiled);
  assert.ok(execute.calls[0].arguments.includes('formal-ai procedure conformance'));
  assert.ok(execute.calls[0].arguments.includes(PROCEDURE_CONFORMANCE_TRIGGER));
  const execution = procedureConformance(compiled, PROCEDURE_CONFORMANCE_TRIGGER);
  answerToolCall(messages, execute.calls[0], execution);
  const final = procedure.planStep(messages, tools, compiled);
  assert.equal(final.kind, 'final');
  assert.ok(final.answer.includes(compiled.id));
  assert.ok(final.answer.includes(restateSteps(compiled)));
  assert.ok(final.answer.includes(execution));
});

test('Agent CLI procedure rejects a corrupted readback and a failed execution', () => {
  const tools = ['write_file', 'run_command'];
  const compiled = procedure.compileTask(ENGLISH_PROCEDURE);
  let messages = [user(ENGLISH_PROCEDURE)];
  answerToolCall(messages, procedure.planStep(messages, tools, compiled).calls[0], 'wrote compiled-procedure.lino');
  answerToolCall(messages, procedure.planStep(messages, tools, compiled).calls[0], 'compiled_procedure_artifact "corrupted"');
  const corrupted = procedure.planStep(messages, tools, compiled);
  assert.ok(corrupted.answer.includes('verification failed'), corrupted.answer);
  assert.ok(!corrupted.answer.includes('was written and verified'));

  messages = [user(ENGLISH_PROCEDURE)];
  answerToolCall(messages, procedure.planStep(messages, tools, compiled).calls[0], 'wrote compiled-procedure.lino');
  answerToolCall(messages, procedure.planStep(messages, tools, compiled).calls[0], artifactLinksNotation(compiled));
  answerToolCall(messages, procedure.planStep(messages, tools, compiled).calls[0], 'command exited with status 1\nstderr:\nconformance host failed');
  const failed = procedure.planStep(messages, tools, compiled);
  assert.ok(failed.answer.includes('conformance execution failed'), failed.answer);
  assert.ok(!failed.answer.includes('executed end to end'));
});

test('Agent CLI procedure localizes an honest unpersisted artifact', () => {
  const compiled = procedure.compileTask(RUSSIAN_PROCEDURE);
  const plan = procedure.planStep([user(RUSSIAN_PROCEDURE)], [], compiled);
  assert.equal(plan.kind, 'final');
  assert.ok(plan.answer.includes('Процедура скомпилирована'), plan.answer);
  assert.ok(plan.answer.includes('не сохранён и не проверен'), plan.answer);
  assert.ok(plan.answer.includes(compiled.id));
});

// ---- learning reports (issues #686, #657, #709, ...) -----------------------

test('the associative report and answer match the committed Agent CLI session', () => {
  const recorded = session('docs/case-studies/issue-686/agent-cli-session-associative-learning.json');
  assert.equal(learningReport.route(recorded.task), associativeLearning.REPORT);
  const document = associativeLearning.renderDocument();
  assert.equal(document, recorded.steps[0].arguments.content);
  assert.equal(associativeLearning.finalAnswer(document), recorded.final_answer);
  const plan = learningReport.planReportStep(associativeLearning.REPORT, [user(recorded.task)], recorded.tools_advertised);
  assert.equal(plan.calls[0].tool, recorded.steps[0].tool);
  assert.deepEqual(JSON.parse(plan.calls[0].arguments), recorded.steps[0].arguments);
});

test('the associative report is derived from persisted usage, not canned', () => {
  const baseline = read('data/meta/associative-learning-case.lino');
  const hotter = baseline.split('accessCount "2"').join('accessCount "8"');
  const first = associativeLearning.renderDocumentFrom(baseline);
  const second = associativeLearning.renderDocumentFrom(hotter);
  assert.notEqual(first, second);
  assert.ok(first.includes('retention_formula "reads + writes + incoming_links + outgoing_links"'));
  assert.ok(second.includes('reads "9"'));
  assert.ok(second.includes('multi_hop_recall'));
  for (const prompt of [associativeLearning.task(), 'Please produce ASSOCIATIVE-LEARNING-REPORT.LINO from the learned memory.',
    'Rank our linked facts, then save associative-learning-report.lino']) {
    assert.ok(associativeLearning.isAssociativeLearningTask(prompt), prompt);
  }
  assert.ok(!associativeLearning.isAssociativeLearningTask('Explain associative memory without writing an artifact'));
});

test('self-hosting and search-fusion reports are derived and review gated', () => {
  const baseline = read('data/meta/issue-657-self-hosting-learning.lino');
  const first = selfHostingLearning.renderDocumentFrom(baseline);
  assert.notEqual(first, selfHostingLearning.renderDocumentFrom(baseline.split('accessCount "9"').join('accessCount "14"')));
  for (const needle of ['self_hosting_learning_report', 'issue "657"', 'decision "awaiting_human_review"',
    'promotion_gate "metric_fixture_exact_share_and_honest_ledger_ratchet_pass"', 'lesson:trailer-provenance',
    'lesson:honest-baseline', 'lesson:changed-line-weighting', 'lesson:monotonic-window']) {
    assert.ok(first.includes(needle), needle);
  }
  assert.ok(!first.includes('decision "promoted"'));
  assert.ok(selfHostingLearning.finalAnswer(first).includes('human-review-gated report'));
  const report = searchFusionLearning.renderDocument();
  assert.ok(report.startsWith('search_fusion_learning_report\n  issue "709"\n'));
  assert.ok(report.includes('lesson:gated-recipe-replay'));
});

test('every registered report routes from its own task and identities are unique', () => {
  const reports = learningReport.reports();
  assert.equal(reports.length, 11);
  for (const report of reports) assert.equal(learningReport.route(report.task)?.head, report.head, report.head);
  assert.equal(new Set(reports.map((report) => report.head)).size, reports.length);
  assert.equal(new Set(reports.map((report) => report.path)).size, reports.length);
  assert.equal(learningReport.route('What is the capital of France?'), null);
});

test('committed Agent CLI evidence reports match a fresh render', () => {
  for (const [module, path] of [
    ['learning_report/handler_precedence_learning', 'docs/case-studies/issue-663/agent-cli-evidence/handler-precedence-learning-report.lino'],
    ['learning_report/lexeme_import_learning', 'docs/case-studies/issue-660/agent-cli-evidence/lexeme-import-learning-report.lino'],
    ['learning_report/hardcoded_language_learning', 'docs/case-studies/issue-659/agent-cli-evidence/hardcoded-language-learning-report.lino'],
  ]) {
    const report = learningReport.reports().find((entry) => path.endsWith(entry.path));
    assert.ok(report, module);
    assert.equal(learningReport.renderDocument(report), read(path), path);
  }
});

// ---- document recipes (issues #538, #540, #527, #498, #499) ----------------

test('the dreaming audit is derived from the recipe and matches the committed session', () => {
  const recorded = session('docs/case-studies/issue-540/agent-cli-session-dreaming-audit.json');
  assert.ok(dreamingAudit.isDreamingAuditTask(dreamingAudit.dreamingAuditTask()));
  assert.equal(recorded.task, dreamingAudit.dreamingAuditTask());
  assert.ok(!dreamingAudit.isDreamingAuditTask('Perform a gap analysis for issue 540 dreaming'));
  assert.ok(!dreamingAudit.isDreamingAuditTask('Please audit my dreaming journal app'));
  assert.ok(dreamingAudit.isDreamingAuditTask('Regenerate dreaming-gap-analysis.lino from the recipe'));
  const document = dreamingAudit.renderDocument();
  assert.equal(document, recorded.steps[0].arguments.content);
  assert.equal(document, read('docs/case-studies/issue-540/dreaming-gap-analysis.lino'));
  assert.equal(dreamingAudit.finalAnswer(document), recorded.final_answer);
});

test('the self-AST recipe routes narrowly and answers like the committed session', () => {
  assert.ok(selfAst.isSelfAstTask(selfAst.astTask()));
  assert.ok(selfAst.isSelfAstTask('record the abstract-syntax of our planner in our data'));
  for (const prompt of ['make the tomato meaning more detailed', 'generate the mermaid diagrams of our recipes', 'What is the capital of France?']) {
    assert.ok(!selfAst.isSelfAstTask(prompt), prompt);
  }
  const recorded = session('docs/case-studies/issue-538/agent-cli-session-self-ast.json');
  assert.equal(recorded.task, selfAst.astTask());
  const document = selfAst.renderDocument();
  assert.equal(document, read('data/meta/self-ast.lino'));
  assert.equal(selfAst.finalAnswer(recorded.steps[0].arguments.content), recorded.final_answer);
  assert.equal(selfAst.renderCensusDocument(selfAst.TARGET_MODULE_PATH, {
    total_link_count: 1, named_node_count: 1, text_preserved: true, clean: true, node_kinds: [{ kind: 'identifier', count: 1 }],
  }).split('\n').slice(-3).join('\n'), '  node_kinds\n    identifier 1\n');
});

test('the question catalog recipe routes narrowly and answers like the committed session', () => {
  assert.ok(questionCatalog.isQuestionCatalogTask(questionCatalog.questionCatalogTask()));
  for (const prompt of ['Please generate every possible question and answer each one.',
    'Build the question catalog and record it in Links Notation.', 'Enumerate questions from smallest to largest and answer them.']) {
    assert.ok(questionCatalog.isQuestionCatalogTask(prompt), prompt);
  }
  assert.ok(!questionCatalog.isQuestionCatalogTask('what files are in this folder?'));
  assert.ok(!questionCatalog.isQuestionCatalogTask('what is formal ai?'));
  const recorded = session('docs/case-studies/issue-527/agent-cli-session-question-catalog.json');
  assert.equal(questionCatalog.renderDocument(), recorded.steps[0].arguments.content);
  assert.equal(questionCatalog.finalAnswer(questionCatalog.renderDocument()), recorded.final_answer);
  const catalog = questionCatalog.catalog();
  const [first] = catalog.answered;
  assert.equal(questionCatalog.answerFor(catalog, `  ${first.question.toUpperCase()} `), first);
  assert.equal(questionCatalog.answerFor(catalog, 'what colour is the sky?'), null);
});

test('the Google Trends recipes route and answer like the committed sessions', () => {
  assert.ok(googleTrendsCatalog.isGoogleTrendsCatalogTask(googleTrendsCatalog.googleTrendsCatalogTask()));
  assert.ok(googleTrendsCatalog.isGoogleTrendsCatalogTask('Build a Google Trends catalog from the top searches and answer each request.'));
  assert.ok(googleTrendsCatalog.isGoogleTrendsCatalogTask('Convert trending searches into multilingual Formal AI test prompts.'));
  assert.ok(!googleTrendsCatalog.isGoogleTrendsCatalogTask('what is formal ai?'));
  assert.ok(!googleTrendsCatalog.isGoogleTrendsCatalogTask(questionCatalog.questionCatalogTask()));
  assert.ok(!questionCatalog.isQuestionCatalogTask(googleTrendsCatalog.googleTrendsCatalogTask()));
  assert.ok(googleTrendsLearning.isGoogleTrendsLearningTask(googleTrendsLearning.googleTrendsLearningTask()));
  assert.ok(googleTrendsLearning.isGoogleTrendsLearningTask('Map the Google Trends learning frontier and hand it to the self-improvement loop.'));
  assert.ok(googleTrendsLearning.isGoogleTrendsLearningTask('Which trending searches can Formal AI not yet resolve? Route them to the gated learner.'));
  assert.ok(!googleTrendsLearning.isGoogleTrendsLearningTask(googleTrendsCatalog.googleTrendsCatalogTask()));
  assert.ok(!googleTrendsCatalog.isGoogleTrendsCatalogTask(googleTrendsLearning.googleTrendsLearningTask()));
  assert.ok(!googleTrendsLearning.isGoogleTrendsLearningTask('what is formal ai?'));
  assert.ok(!googleTrendsLearning.isGoogleTrendsLearningTask('Here you can learn a lot from reading books'));
  for (const [path, recipe] of [
    ['docs/case-studies/issue-498/agent-cli-session-google-trends.json', googleTrendsCatalog],
    ['docs/case-studies/issue-498/agent-cli-session-google-trends-learning.json', googleTrendsLearning],
    ['docs/case-studies/issue-499/agent-cli-session-learn-from-source.json', googleTrendsLearning],
  ]) {
    const recorded = session(path);
    if (recipe === googleTrendsLearning) assert.ok(recipe.isGoogleTrendsLearningTask(recorded.task), path);
    else assert.ok(recipe.isGoogleTrendsCatalogTask(recorded.task), path);
    assert.equal(recipe.renderDocument(), recorded.steps[0].arguments.content, path);
    assert.equal(JSON.parse(JSON.stringify(recorded.steps[1].arguments)).command, recipe.verificationCommand(), path);
    assert.equal(recipe.finalAnswer(recipe.renderDocument()), recorded.final_answer, path);
  }
});
