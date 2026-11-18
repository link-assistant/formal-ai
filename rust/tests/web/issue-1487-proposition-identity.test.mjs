// Issue #1487: statements that share their words but not their proposition —
// operand order (`a - b` / `b - a`, `1 < 2` / `2 < 1`), comparison strength
// (`<` / `<=`), connective (`and` / `or`), negation scope and an explicit
// count (`twice`) — keep distinct identities, are never merged as duplicates,
// and a negated theorem request never receives the positive theorem's proof.
//
// The shared fixture rust/tests/fixtures/proposition-structure-parity.json is
// also read by the native test rust/tests/unit/proposition_structure_parity.rs,
// so the JavaScript and native readers are held to the same answers.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import { before, describe, it } from 'node:test';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { formalizeText, statementIdentity, statementStructure } from '../../../js/agentic/crate/text_formalization.mjs';
import { structureKey } from '../../../js/agentic/crate/proposition_structure.mjs';
import { certifiesRestatement, checkedRelation } from '../../../js/agentic/crate/statement_relation.mjs';
import { sentenceStatements, splitDuplicates } from '../../../js/agentic/crate/dependency_summarization.mjs';
import { signatureDenies, signatureKey, signatureOf } from '../../../js/agentic/crate/summarization_dedup.mjs';
import { ClaimBinding, theoremMatch, bindTheoremClaim } from '../../../js/agentic/crate/theorem_claim_binding.mjs';

const FIXTURE = JSON.parse(fs.readFileSync(new URL('../fixtures/proposition-structure-parity.json', import.meta.url), 'utf8'));

let host;
before(async () => {
  host = new WorkerHost();
  await installNodeHost(host);
  host.context.fetch = () => { throw new Error('offline test: no network'); };
});

const one = (text, language = 'en') => {
  const statements = formalizeText(text, language);
  assert.equal(statements.length, 1, `one statement for ${text}`);
  return statements[0];
};

const DISTINCT_PAIRS = [
  ['1 is less than 2.', '2 is less than 1.'],
  ['a-b', 'b-a'],
  ['Compute a - b.', 'Compute b - a.'],
  ['1 < 2', '1 <= 2'],
  ['The program returns true or false.', 'The program returns true and false.'],
  ['Not all primes are odd.', 'All primes are not odd.'],
  ['Print hello twice.', 'Print hello.'],
];

describe('issue #1487 shared JS/native fixture', () => {
  it('reads every fixture structure key', () => {
    assert.ok(FIXTURE.structures.length > 0);
    for (const entry of FIXTURE.structures) {
      assert.equal(structureKey(statementStructure(one(entry.text, entry.language))), entry.key, entry.text);
    }
  });

  it('checks every fixture relation', () => {
    assert.ok(FIXTURE.relations.length > 0);
    for (const entry of FIXTURE.relations) {
      const relation = checkedRelation(one(entry.later, entry.language), one(entry.earlier, entry.language));
      const label = `${entry.later} against ${entry.earlier}`;
      assert.equal(relation.verdict, entry.verdict, label);
      assert.equal(relation.reason, entry.reason, label);
      assert.equal(certifiesRestatement(relation), entry.restates, label);
    }
  });

  it('keys every fixture statement-merge signature', () => {
    assert.ok(FIXTURE.signatures.length > 0);
    for (const entry of FIXTURE.signatures) assert.equal(signatureKey(signatureOf(entry.text)), entry.key, entry.text);
  });
});

describe('issue #1487 statement identity and restatement', () => {
  it('keeps ordered, connective, scope and count pairs distinct', () => {
    for (const [left, right] of DISTINCT_PAIRS) {
      const later = one(right);
      const earlier = one(left);
      assert.notEqual(statementIdentity(later), statementIdentity(earlier), `${left} / ${right}`);
      assert.equal(certifiesRestatement(checkedRelation(later, earlier)), false, `${right} is not ${left}`);
      assert.equal(certifiesRestatement(checkedRelation(earlier, later)), false, `${left} is not ${right}`);
    }
  });

  it('still unifies a checked paraphrase and a repeated mention', () => {
    assert.deepEqual(checkedRelation(one('2 is greater than 1.'), one('1 is less than 2.')), { verdict: 'equivalent', reason: 'same-relations' });
    assert.deepEqual(checkedRelation(one('1 < 2'), one('1 is less than 2.')), { verdict: 'equivalent', reason: 'same-relations' });
    assert.deepEqual(checkedRelation(one('Print hello.'), one('Print hello.')), { verdict: 'equivalent', reason: 'same-identity' });
    assert.deepEqual(checkedRelation(one('1972'), one('1969 and 1972')), { verdict: 'entailed', reason: 'conjunction-elimination' });
  });

  it('does not drop a distinct sentence as a duplicate in dependency summarization', () => {
    const sentence = (text) => (/[.!?]$/u.test(text) ? text : `${text}.`);
    for (const [left, right] of DISTINCT_PAIRS) {
      const { unique, removed } = splitDuplicates(sentenceStatements(`${sentence(left)} ${sentence(right)}`, 'en'));
      assert.equal(removed.length, 0, `${left} / ${right} removed ${JSON.stringify(removed.map((entry) => entry.statement?.text))}`);
      assert.equal(unique.length, 2, `${left} / ${right}`);
    }
    const repeated = splitDuplicates(sentenceStatements('Print hello. Print hello.', 'en'));
    assert.equal(repeated.removed.length, 1, 'a repeated mention is still one statement');
  });
});

describe('issue #1487 statement-merge signatures', () => {
  it('gives each distinct pair its own signature', () => {
    for (const [left, right] of [...DISTINCT_PAIRS, ['Не все тесты проходят.', 'Все тесты не проходят.'], ['程序返回真或假。', '程序返回真和假。']]) {
      assert.notEqual(signatureKey(signatureOf(left)), signatureKey(signatureOf(right)), `${left} / ${right}`);
    }
  });

  it('still merges a repeated mention and reports a plain denial', () => {
    assert.equal(signatureKey(signatureOf('The parser is fast.')), signatureKey(signatureOf('The parser is fast.')));
    assert.equal(signatureDenies(signatureOf('The parser is not fast.'), signatureOf('The parser is fast.')), true);
    assert.equal(signatureDenies(signatureOf('The program does not return true or false.'), signatureOf('The program returns true and false.')), false);
  });
});

// The exact answers, identical to the native ones in rust/tests/unit/proof_request.rs
// (`negated_theorem_request_is_refuted_never_proven`, `seeded_euclid_proof_renders_with_seeded_labels`): both runtimes
// render the seeded Euclid proof, and a negated request receives it only as the refutation.
const EUCLID_PROOF_ANSWER = "How I interpreted the request: treating the request as the formal claim \"There are infinitely many prime numbers.\" and discharging it by proof by contradiction inside relative-meta-logic.\n\nProof (method: proof by contradiction).\n\nStatement: There are infinitely many prime numbers.\n\n1. Definition: Work in elementary number theory, formalizable in Peano arithmetic (PA): a prime is an integer greater than 1 whose only positive divisors are 1 and itself. The proof uses the PA theorem that every integer greater than 1 has a prime divisor; this is the formal context for the relative-meta-logic contradiction tactic.\n2. Hypothesis: Assume for contradiction that only finitely many primes exist; call them p₁, p₂, …, pₙ.\n3. Inference: Form the number N = p₁·p₂·…·pₙ + 1. Then N is an integer greater than 1.\n4. Inference: By the fundamental theorem of arithmetic, N has a prime divisor q. If q = pᵢ for some i, then pᵢ divides both p₁·p₂·…·pₙ and N, so pᵢ divides their difference, which is 1 — impossible.\n5. Inference: Hence q is a prime not in the list p₁, …, pₙ, contradicting the assumption that the list was complete.\nThe assumption fails, so there are infinitely many primes. ∎";
const EUCLID_REFUTATION_ANSWER = "How I interpreted the request: treating the request as a claim to be refuted; applying proof by contradiction and producing a counterexample.\n\nDisproof (method: proof by contradiction).\n\nCounterexample: The claim denies a proven theorem: There are infinitely many prime numbers. The proof below refutes the claim.\n\n1. Definition: Work in elementary number theory, formalizable in Peano arithmetic (PA): a prime is an integer greater than 1 whose only positive divisors are 1 and itself. The proof uses the PA theorem that every integer greater than 1 has a prime divisor; this is the formal context for the relative-meta-logic contradiction tactic.\n2. Hypothesis: Assume for contradiction that only finitely many primes exist; call them p₁, p₂, …, pₙ.\n3. Inference: Form the number N = p₁·p₂·…·pₙ + 1. Then N is an integer greater than 1.\n4. Inference: By the fundamental theorem of arithmetic, N has a prime divisor q. If q = pᵢ for some i, then pᵢ divides both p₁·p₂·…·pₙ and N, so pᵢ divides their difference, which is 1 — impossible.\n5. Inference: Hence q is a prime not in the list p₁, …, pₙ, contradicting the assumption that the list was complete.\nThe assumption fails, so there are infinitely many primes. ∎\n\nClarifying questions:\n1. do you want to weaken the claim into a checkable form (e.g. replace equality with an inequality, or restrict the domain)?\n2.";

describe('issue #1487 proof requests bind to the theorem proposition', () => {
  it('binds affirmed, negated and unread-scope claims', () => {
    const cases = [
      ['Prove that there are infinitely many primes.', ClaimBinding.Affirmed],
      ['Prove there are not infinitely many primes.', ClaimBinding.Negated],
      ['Prove that it is not true that there are not infinitely many primes.', ClaimBinding.Unsupported],
      ['Prove there are infinitely many primes that are not even.', ClaimBinding.Unsupported],
    ];
    for (const [claim, expected] of cases) {
      const match = theoremMatch(claim);
      assert.ok(match, `a theorem keyword in ${claim}`);
      assert.equal(bindTheoremClaim(claim, match.keyword), expected, claim);
    }
  });

  it('never answers a negated theorem request with the positive proof', async () => {
    const proven = await host.solve('Prove that there are infinitely many primes.');
    assert.equal(proven.content, EUCLID_PROOF_ANSWER);
    assert.equal((await host.solve('Prove there are not infinitely many primes.')).content, EUCLID_REFUTATION_ANSWER);
    assert.ok(proven.evidence.includes('proof_outcome:proven'), JSON.stringify(proven.evidence));
    assert.ok(proven.evidence.includes('proof_binding:affirmed'), JSON.stringify(proven.evidence));
    for (const prompt of ['Prove there are not infinitely many primes.', '证明不存在无穷多素数。']) {
      const response = await host.solve(prompt);
      assert.ok(response.evidence.includes('proof_binding:negated'), `${prompt}: ${JSON.stringify(response.evidence)}`);
      assert.ok(response.evidence.includes('proof_outcome:disproven'), `${prompt}: ${JSON.stringify(response.evidence)}`);
      assert.ok(!response.evidence.includes('proof_outcome:proven'), prompt);
      assert.ok(!/(^|\n)(Proof|证明) \(/u.test(response.content), `${prompt} must not be presented as proven: ${response.content}`);
    }
    for (const prompt of ['Prove that it is not true that there are not infinitely many primes.', 'Prove there are infinitely many primes that are not even.', 'Докажи что простых чисел не бесконечно много.']) {
      const response = await host.solve(prompt);
      assert.ok(response.evidence.includes('proof_outcome:partial_plan'), `${prompt}: ${JSON.stringify(response.evidence)}`);
      assert.ok(!response.evidence.includes('proof_outcome:proven'), prompt);
      assert.ok(!response.evidence.includes('proof_outcome:disproven'), prompt);
    }
  });
});
