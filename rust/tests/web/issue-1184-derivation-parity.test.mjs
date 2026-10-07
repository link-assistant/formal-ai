// Issue #1184 R1184-9 parity: the JavaScript root projects the same
// derivation record as rust/src/derivation.rs, case for case with
// rust/tests/unit/issue_1184_derivation_records.rs. The expected Links
// Notation texts are the bytes `Derivation::to_lino` writes for the same
// event log, so the two roots emit one record shape.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import {
  answerDerivationId,
  appliedRule,
  appliedRuleKinds,
  explainText,
  fetchRecord,
  fromLino,
  parseVerificationPayload,
  recordFor,
  storePath,
  toLino,
  verificationPayload,
  verificationRecord,
} from '../../../js/agentic/crate/derivation.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const event = (kind, payload) => ({ kind, payload });

/** The log of an online answer: two queries, one fetch in each `source:http` spelling. */
const onlineAnswerLog = () => [
  event('web_search:request', 'how to compile a Kotlin program'),
  event(
    'source:http',
    'https://kotlinlang.org/docs/command-line.html fetched_at=2026-09-29T00:00:00Z '
      + 'sha256=9f86d081884c7d65a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08 cached=false',
  ),
  event(
    'source:http',
    'url=https://example.com/kotlin;fetched_at=2026-09-29T00:01:00Z;'
      + 'sha256=2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae;'
      + 'catalog_match=none',
  ),
  event('web_search:request', 'kotlinc command line options'),
];

test('derivation carries search queries, fetches and hashes from both source:http spellings', () => {
  const derivation = recordFor(onlineAnswerLog(), 'answer_0123456789abcdef');
  assert.deepEqual(derivation.search_queries, ['how to compile a Kotlin program', 'kotlinc command line options']);
  assert.equal(derivation.fetches.length, 2);
  assert.deepEqual(
    derivation.fetches[0],
    fetchRecord(
      'https://kotlinlang.org/docs/command-line.html',
      '9f86d081884c7d65a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
      '2026-09-29T00:00:00Z',
    ),
  );
  assert.deepEqual(
    derivation.fetches[1],
    fetchRecord(
      'https://example.com/kotlin',
      '2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae',
      '2026-09-29T00:01:00Z',
    ),
  );
});

test('the answer id is content-addressed and stable', () => {
  const once = answerDerivationId('Kotlin programs are compiled with kotlinc.');
  assert.equal(once, answerDerivationId('Kotlin programs are compiled with kotlinc.'));
  assert.notEqual(once, answerDerivationId('Scala programs are compiled with scalac.'));
  assert.match(once, /^answer_[0-9a-f]{16}$/u);
});

test('explain reports not recorded for stages a route did not populate', () => {
  const derivation = recordFor([event('calculation', '2+2=4')], 'answer_fedcba9876543210');
  assert.deepEqual(derivation.fetches, []);
  assert.deepEqual(derivation.search_queries, []);
  const explanation = explainText(derivation);
  for (const stage of ['stage search_queries', 'stage fetches', 'stage formalized_fragments', 'stage decomposed_parts', 'stage verification']) {
    const block = explanation.split(stage)[1];
    assert.ok(block !== undefined, `explanation names ${stage}`);
    assert.ok(block.startsWith('\n    not recorded'), `${stage} reports not recorded, got:${block}`);
  }
  assert.ok(explanation.includes('stage recomposition: not recorded'));
  assert.ok(explanation.includes('stage rendering: not recorded'));
  assert.equal(
    explanation,
    'derivation answer_fedcba9876543210\n'
      + '  stage search_queries\n    not recorded\n'
      + '  stage fetches\n    not recorded\n'
      + '  stage formalized_fragments\n    not recorded\n'
      + '  stage decomposed_parts\n    not recorded\n'
      + '  stage recomposition: not recorded\n'
      + '  stage rendering: not recorded\n'
      + '  stage verification\n    not recorded\n'
      + '  stage applied_rules\n    not recorded\n',
  );
});

test('the full record writes the bytes Rust writes and round-trips', () => {
  const derivation = recordFor(onlineAnswerLog(), 'answer_0123456789abcdef');
  derivation.recomposition = 'bound literal=Hello, Formal AI!';
  derivation.rendering = 'kotlin';
  derivation.verification.push(verificationRecord('evidence_0000000000000000', 'python3 solution.py', 0));
  const text = toLino(derivation);
  assert.equal(
    text,
    'derivation\n'
      + '  answer_id "answer_0123456789abcdef"\n'
      + '  search_query "how to compile a Kotlin program"\n'
      + '  search_query "kotlinc command line options"\n'
      + '  fetch\n'
      + '    url "https://kotlinlang.org/docs/command-line.html"\n'
      + '    sha256 "9f86d081884c7d65a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"\n'
      + '    fetched_at "2026-09-29T00:00:00Z"\n'
      + '  fetch\n'
      + '    url "https://example.com/kotlin"\n'
      + '    sha256 "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae"\n'
      + '    fetched_at "2026-09-29T00:01:00Z"\n'
      + '  recomposition "bound literal=Hello, Formal AI!"\n'
      + '  rendering "kotlin"\n'
      + '  verification\n'
      + '    evidence_id "evidence_0000000000000000"\n'
      + '    command "python3 solution.py"\n'
      + '    exit "0"\n',
  );
  assert.deepEqual(fromLino(text), derivation, 'the lino round-trip is lossless');
  const explained = explainText(derivation);
  assert.ok(explained.includes('sha256 9f86d0'));
  assert.ok(explained.includes('python3 solution.py exit=0'));
  assert.equal(fromLino('sources_registry\n  answer_id "x"\n'), null, 'not a derivation record');
  assert.equal(fromLino('derivation\n  rendering "kotlin"\n'), null, 'no answer_id');
});

test('verification payload round-trips through its event spelling', () => {
  const record = verificationRecord('evidence_1234567890abcdef', 'python3 solution.py', 0);
  assert.equal(verificationPayload(record), 'evidence_id=evidence_1234567890abcdef;command=python3 solution.py;exit=0');
  assert.deepEqual(parseVerificationPayload(verificationPayload(record)), record);
  const derivation = recordFor([event('verify:evidence', verificationPayload(record))], 'answer_0123456789abcdef');
  assert.deepEqual(derivation.verification, [record]);
  assert.equal(parseVerificationPayload('command=x;exit=0'), null, 'no evidence_id field');
  assert.equal(parseVerificationPayload('evidence_id=;command=x;exit=0'), null, 'empty evidence_id');
  assert.equal(parseVerificationPayload('evidence_id=e;command=x;exit=none').exit_code, null);
});

test('verification payload preserves shell separators and the legacy spelling', () => {
  const record = verificationRecord('evidence_123', 'printf first; printf second exit=inside', 0);
  assert.deepEqual(parseVerificationPayload(verificationPayload(record)), record);
  const legacy = parseVerificationPayload('evidence_id=evidence_old command=python3 solution.py exit=0');
  assert.equal(legacy.command, 'python3 solution.py');
  assert.equal(legacy.exit_code, 0);
});

test('the store path refuses ids that could traverse the tree', () => {
  assert.equal(storePath('/repo', 'answer_ok'), '/repo/data/cache/derivations/answer_ok.lino');
  assert.equal(storePath('/repo', '../secrets'), null);
  assert.equal(storePath('/repo', ''), null);
  assert.equal(storePath('/repo', 'a/b'), null);
});

test('lino values survive quoting and line breaks', () => {
  const derivation = recordFor([], 'answer_0123456789abcdef');
  derivation.search_queries.push('what is "1 + 1"?\nsecond line');
  const text = toLino(derivation);
  assert.equal(
    text,
    'derivation\n  answer_id "answer_0123456789abcdef"\n  search_query \'what is "1 + 1"?\\nsecond line\'\n',
  );
  assert.deepEqual(fromLino(text), derivation, 'quoted and escaped values round-trip');
});

test('applied rules are projected from the schema, round-tripped and explained', () => {
  const kinds = appliedRuleKinds();
  assert.ok(kinds.has('register_rewrite') && kinds.has('grammar_correction') && kinds.has('translation_gap'));
  assert.ok(!kinds.has('web_search:request'), 'only stages that collect rule');
  const log = [
    event('register_rewrite', 'u -> you'),
    event('grammar_correction', "don't -> doesn't (third_person_agreement)"),
    event('thinking:note', 'not a rule event'),
  ];
  const derivation = recordFor(log, 'answer_00000000000000aa');
  assert.deepEqual(derivation.applied_rules, [
    appliedRule('register_rewrite', 'u -> you'),
    appliedRule('grammar_correction', "don't -> doesn't (third_person_agreement)"),
  ]);
  const text = toLino(derivation);
  assert.equal(
    text,
    'derivation\n'
      + '  answer_id "answer_00000000000000aa"\n'
      + '  rule\n'
      + '    kind "register_rewrite"\n'
      + '    detail "u -> you"\n'
      + '  rule\n'
      + '    kind "grammar_correction"\n'
      + '    detail "don\'t -> doesn\'t (third_person_agreement)"\n',
  );
  assert.deepEqual(fromLino(text), derivation, 'applied rules round-trip through Links Notation');
  const explanation = explainText(derivation);
  assert.ok(explanation.includes('stage applied_rules') && explanation.includes("grammar_correction don't -> doesn't"));
  assert.ok(explainText(recordFor([], 'answer_00000000000000ab')).includes('stage applied_rules\n    not recorded'));
});
