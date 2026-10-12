import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, realpathSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { declareTask } from '../../../experiments/formal_ai_subagent/task-relations.mjs';
import { sourceDigest } from '../../../experiments/formal_ai_subagent/coding-amplification.mjs';
import {
  observeDeclaredCategoryEvidence, categoryEvidenceObservation, summarizeDeclaredCategoryEvidence,
} from '../../../experiments/formal_ai_subagent/category-evidence-report.mjs';

function fixture(options = {}) {
  const directory = realpathSync(mkdtempSync(join(tmpdir(), 'category-evidence-')));
  const save = (name, value) => {
    const path = join(directory, name);
    writeFileSync(path, typeof value === 'string' ? value : JSON.stringify(value));
    return { path, sha256: sourceDigest(readFileSync(path)) };
  };
  const before = save('before.mjs',
    options.before ?? 'export function sum(first, second) { return 0; }\n');
  const after = save('candidate.mjs',
    options.after ?? 'export function sum(first, second) { return first + second; }\n');
  const assertion = options.assertion
    ?? 'assert.equal(sum(2, 3), 5); assert.equal(sum(-2, 7), 5); assert.equal(sum(5, -1), 4);';
  const oracle = candidate => "import test from 'node:test';\n"
    + "import assert from 'node:assert/strict';\n"
    + "import { sum } from './" + candidate + "';\n"
    + "test('original finite behavior', () => { " + assertion + " });\n";
  const beforeOracle = save('before.test.mjs', oracle('before.mjs'));
  const afterOracle = save('candidate.test.mjs', oracle('candidate.mjs'));
  const contract = (candidate, oracleBinding, name) => {
    const declaration = {
      version: 1, transport: 'pure-esm', candidate, oracle: oracleBinding,
      dependencies: [], interface: ['sum'],
    };
    return { transport: 'pure-esm', declaration: save(name, declaration),
      candidate, oracle: oracleBinding, dependencies: [] };
  };
  const task = declareTask({
    runId: directory, taskKind: options.taskKind ?? 'coding',
    category: options.category ?? 'feature-implementation',
    task: 'Implement the declared addition behavior.',
  });
  const declaration = {
    version: 1, task, expectedAttempts: ['first', 'retry'],
    baselineContract: contract(before, beforeOracle, 'before-contract.json'),
    candidateContract: contract(after, afterOracle, 'candidate-contract.json'),
    changes: [{ path: after.path, role: options.role ?? 'production',
      language: 'javascript', before, after }],
  };
  const preregistration = save('preregistration.json', declaration);
  const attempts = ['first', 'retry'].map((attemptId, index) => ({
    attemptId, origin: options.origin ?? 'autonomous', suppliedInputs: [],
    input: save(attemptId + '-input.json', {
      runId: task.runId, attemptId, task: task.task,
      observedContext: ['Exact baseline: ' + readFileSync(before.path, 'utf8')],
      providerUsage: { status: 'Captured', inputTokens: 12345 },
    }),
    output: save(attemptId + '-output.json', {
      runId: task.runId, attemptId,
      disposition: index === 0 ? 'failed' : options.disposition ?? 'final',
      response: '5', accepted: true, semanticUsefulness: 'HighQuality',
    }),
  }));
  return { save, directory, before, after, declaration, preregistration, attempts };
}
async function observed(options) {
  const value = fixture(options);
  const record = await observeDeclaredCategoryEvidence(value.preregistration, value.attempts);
  return { value, record, data: categoryEvidenceObservation(record),
    report: summarizeDeclaredCategoryEvidence([value.preregistration], [record]) };
}

test('actual baseline failure and physical candidate success remain scoped structural evidence', async () => {
  const { data, report } = await observed();
  assert.equal(data.baselineOracle.process.status, 1);
  assert.equal(data.candidateOracle.process.status, 0);
  assert.equal(data.structuralOutcome, 'PassedOracle');
  assert.equal(data.requestedPhysicalChangeObserved, true);
  assert.ok(data.lexicalNetCodeProxy > 0);
  assert.equal(report.expansion.ordinaryCoding.observedStructuralSuccessDenominator, 1);
  assert.equal(report.semanticUsefulness, 'Unknown');
  assert.equal(report.expansion.combined.usefulExpansionDenominator, 0);
  assert.equal(report.autonomousUsefulNetCodeBytes, 0);
});
test('a correct shorter feature does not fail for lack of growth', async () => {
  const { data, report } = await observed({
    before: 'export function sum(x, y) { return 0; }\n',
    after: 'export function sum(x, y) { return x + y; }\n',
  });
  assert.equal(data.structuralOutcome, 'PassedOracle');
  assert.ok(data.lexicalNetCodeProxy < 0);
  assert.equal(data.disposition, 'final');
  assert.equal(report.noGrowthFloor, true);
  assert.equal(report.expansion.combined.observedPositiveLexicalGrowth, 0);
});
test('literal payload copying cannot manufacture lexical growth or semantic success', async () => {
  const literal = 'exact authored literal '.repeat(300);
  const { data, report } = await observed({
    before: "export function sum() { return 'before'; }\n",
    after: "export function sum() { return " + JSON.stringify(literal) + "; }\n",
    assertion: 'assert.equal(sum(), ' + JSON.stringify(literal) + ');',
  });
  assert.equal(data.structuralOutcome, 'PassedOracle');
  assert.ok(data.rawPhysicalByteDelta > 5000);
  assert.ok(data.literalTokenByteDelta > 5000);
  assert.equal(data.lexicalNetCodeProxy, 0);
  assert.equal(report.expansion.combined.observedPositiveLexicalGrowth, 0);
});
test('caller quality flags cannot override an actual failed oracle', async () => {
  const { data, report } = await observed({
    after: 'export function sum(first, second) { return first - second; }\n',
  });
  assert.equal(data.candidateOracle.process.status, 1);
  assert.equal(data.structuralOutcome, 'FailedOracle');
  assert.equal(report.expansion.combined.observedStructuralSuccessDenominator, 0);
  assert.equal(report.dispositions.failedOracle, 1);
});
test('unknown candidate operations are refused rather than observed successes', async () => {
  const { data, report } = await observed({
    after: 'export function sum(first, second) { return globalThis.process.exit(0); }\n',
  });
  assert.equal(data.structuralOutcome, 'RefusedOracle');
  assert.equal(report.expansion.combined.observedStructuralSuccessDenominator, 0);
  assert.equal(report.dispositions.refusedOracle, 1);
});
test('unissued foreign and proxied records keep explicit refused inventory rows', async () => {
  const first = fixture();
  const second = await observed({ taskKind: 'self-coding' });
  for (const record of [{ accepted: true }, second.record, new Proxy(second.record, {})]) {
    const report = summarizeDeclaredCategoryEvidence([first.preregistration], [record]);
    assert.equal(report.declared, 1);
    assert.equal(report.cases[0].reason, 'UnissuedOrForeignObservation');
    assert.equal(report.dispositions.refused, 1);
    assert.equal(report.expansion.combined.observedStructuralSuccessDenominator, 0);
  }
});
test('missing or duplicated original retries cannot create complete observations', async () => {
  for (const change of [rows => rows.slice(1), rows => [rows[1], rows[1]]]) {
    const value = fixture();
    const record = await observeDeclaredCategoryEvidence(value.preregistration, change(value.attempts));
    assert.equal(categoryEvidenceObservation(record).disposition, 'refused');
    assert.equal(summarizeDeclaredCategoryEvidence([value.preregistration], [record]).declared, 1);
  }
});
for (const disposition of ['failed', 'refused', 'unfinished']) {
  test(disposition + ' latest attempts remain separate from real passed physical oracles', async () => {
    const { data, report } = await observed({ disposition });
    assert.equal(data.candidateOracle.process.status, 0);
    assert.equal(data.structuralOutcome, 'PassedOracle');
    assert.equal(data.disposition, disposition);
    assert.equal(report.dispositions[disposition], 1);
    assert.equal(report.expansion.combined.observedStructuralSuccessDenominator, 0);
  });
}
test('changing original oracle assertions refuses transport', async () => {
  const value = fixture();
  value.declaration.baselineContract.oracle = value.save('changed-before.test.mjs',
    readFileSync(value.declaration.baselineContract.oracle.path, 'utf8').replace('), 5);', '), 99);'));
  value.preregistration = value.save('preregistration.json', value.declaration);
  const record = await observeDeclaredCategoryEvidence(value.preregistration, value.attempts);
  assert.match(categoryEvidenceObservation(record).error, /assertions/);
});
test('physical input and candidate drift cannot be rescued by flags', async () => {
  for (const target of ['input', 'candidate']) {
    const value = fixture();
    writeFileSync(target === 'input' ? value.attempts[0].input.path : value.after.path, 'drift');
    const record = await observeDeclaredCategoryEvidence(value.preregistration, value.attempts);
    assert.equal(categoryEvidenceObservation(record).disposition, 'refused');
  }
});
test('symlink declarations cannot claim physical source identity', async () => {
  const value = fixture();
  const path = join(value.directory, 'alias.json');
  symlinkSync(value.preregistration.path, path);
  const record = await observeDeclaredCategoryEvidence({
    path, sha256: value.preregistration.sha256,
  }, value.attempts);
  assert.equal(categoryEvidenceObservation(record).disposition, 'refused');
});
for (const origin of ['reviewed-patch', 'generator']) {
  test(origin + ' effects stay observed without expansion credit', async () => {
    const { data, report } = await observed({ origin });
    assert.equal(data.structuralOutcome, 'PassedOracle');
    assert.equal(data.suppliedOrGenerated, true);
    assert.equal(data.autonomousCredit, 0);
    assert.equal(report.expansion.combined.observedPositiveLexicalGrowth, 0);
    assert.equal(report.dispositions.suppliedOrGenerated, 1);
  });
}
test('supplied literal inputs remain measured without buying growth credit', async () => {
  const value = fixture();
  value.attempts[1].suppliedInputs.push(value.save('supplied.txt', 'actual supplied source'.repeat(100)));
  const record = await observeDeclaredCategoryEvidence(value.preregistration, value.attempts);
  assert.ok(categoryEvidenceObservation(record).suppliedInputBytes > 1000);
  assert.equal(summarizeDeclaredCategoryEvidence(
    [value.preregistration], [record]).expansion.combined.observedPositiveLexicalGrowth, 0);
});
for (const category of ['repair', 'refactoring', 'deletion', 'translation']) {
  test(category + ' remains unrestricted with positive observed source growth', async () => {
    const { data, report } = await observed({ category });
    assert.equal(data.structuralOutcome, 'PassedOracle');
    assert.equal(data.expectedRelation, 'unrestricted');
    assert.equal(report.unrestricted.declared, 1);
    assert.equal(report.expansion.combined.declared, 0);
  });
}
for (const [category, taskKind] of [
  ['calculation', 'mathematics'], ['decision', 'other'],
  ['extraction', 'other'], ['summarization', 'other'],
]) {
  test(category + ' response bytes get no code or semantic response credit', async () => {
    const { data, report } = await observed({ category, taskKind });
    assert.equal(data.responseBytes, 1);
    assert.equal(data.responseSemanticAcceptance, 'Unknown');
    assert.equal(data.lexicalNetCodeProxy, null);
    assert.equal(report.contraction.declared, 1);
    assert.equal(report.contraction.observedStructuralSuccessDenominator, 0);
    assert.equal(report.expansion.combined.declared, 0);
  });
}
test('returned clones and public usage metadata cannot change issued accounting', async () => {
  const { value, record, data } = await observed();
  data.semanticUsefulness = true;
  data.lexicalNetCodeProxy = 99999999;
  const report = summarizeDeclaredCategoryEvidence([value.preregistration], [record]);
  assert.equal(report.modelUsage, 'Unknown');
  assert.equal(report.actualCost, null);
  assert.notEqual(report.cases[0].lexicalNetCodeProxy, 99999999);
  assert.equal(report.cases[0].declaredAttempts.length, 2);
  assert.ok(report.cases[0].observedRequestBytes > report.cases[0].declarationTaskBytes);
});
test('missing or duplicate inventory cannot drop declared rows', () => {
  const value = fixture();
  assert.throws(() => summarizeDeclaredCategoryEvidence([value.preregistration], []), /inventory/);
  assert.throws(() => summarizeDeclaredCategoryEvidence(
    [value.preregistration, value.preregistration], [{}, {}]), /inventory/);
});


test('a changed physical registration cannot reuse an old complete attempt observation', async () => {
  const value = fixture();
  const record = await observeDeclaredCategoryEvidence(value.preregistration, value.attempts);
  const changed = value.save('changed-preregistration.json', {
    ...value.declaration, expectedAttempts: ['first', 'retry', 'new-unfinished'],
  });
  const report = summarizeDeclaredCategoryEvidence([changed], [record]);
  assert.equal(report.cases[0].disposition, 'refused');
  assert.equal(report.expansion.combined.observedStructuralSuccessDenominator, 0);
  assert.equal(categoryEvidenceObservation(record).declaredAttempts.length, 2);
});

test('caller registration mutation cannot alter privately observed physical identity', async () => {
  const value = fixture();
  const record = await observeDeclaredCategoryEvidence(value.preregistration, value.attempts);
  const before = categoryEvidenceObservation(record);
  value.preregistration.sha256 = 'f'.repeat(64);
  const after = categoryEvidenceObservation(record);
  assert.deepEqual(after, before);
  assert.notEqual(after.preregistration.sha256, value.preregistration.sha256);
});
