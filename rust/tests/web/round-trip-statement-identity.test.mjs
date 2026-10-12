// Preserve typed statement roles and negation through round-trip translation.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { installTextHost } from '../../../scripts/lib/text-capability-measures.mjs';
import { sameStatementMeaning, roundTrip, roundTripStatement } from '../../../js/agentic/crate/round_trip_translation.mjs';
import { formalizeText, clauseStatement } from '../../../js/agentic/crate/text_formalization.mjs';

before(installTextHost);

test('a target negation preserves actual rendered and reparsed polarity', () => {
  const trip = roundTrip('1 is not 2.', 'en', 'es');
  const [source] = formalizeText('1 is not 2.', 'en');
  const [middle] = formalizeText(trip.forward, 'es');
  const [returned] = formalizeText(trip.backward, 'en');
  assert.equal(middle.polarity, 'denied');
  assert.equal(returned.polarity, 'denied');
  assert.equal(sameStatementMeaning(source, middle), true);
  assert.equal(sameStatementMeaning(source, returned), true);
  assert.equal(trip.survives, true);
  assert.equal(trip.knownTerms, 2);
});

test('changing polarity or exchanging subject and object changes the statement', () => {
  const [statement] = formalizeText('1 is not 2.', 'en');
  assert.equal(sameStatementMeaning(statement, { ...statement, polarity: 'asserted' }), false);
  const exchanged = { ...statement, subject: statement.terms[0], terms: [statement.subject] };
  assert.equal(sameStatementMeaning(statement, exchanged), false);
});

test('ordered terms retain multiplicity and type', () => {
  const [statement] = formalizeText('1 2 3.', 'en');
  assert.equal(sameStatementMeaning(statement, { ...statement, terms: [...statement.terms].reverse() }), false);
  assert.equal(sameStatementMeaning(statement, { ...statement, terms: [...statement.terms, statement.terms[0]] }), false);
  assert.equal(sameStatementMeaning(statement, { ...statement, subject: { ...statement.subject, kind: 'name' } }), false);
});

test('translated surfaces conserve the typed identity without comparing spelling', () => {
  const [statement] = formalizeText('1 2 3.', 'en');
  const translated = { ...statement, language: 'es', text: 'uno dos tres',
    subject: { ...statement.subject, surface: 'uno' },
    terms: statement.terms.map((term) => ({ ...term, surface: 'translated' })) };
  assert.equal(sameStatementMeaning(statement, translated), true);
});

test('a missing subject cannot become a term with the same bag of identities', () => {
  const [statement] = formalizeText('1 2 3.', 'en');
  const withoutSubject = { ...statement, subject: null, terms: [statement.subject, ...statement.terms] };
  assert.equal(sameStatementMeaning(statement, withoutSubject), false);
});

test('a nil subject remains distinct from its first translated content term', () => {
  const [source] = formalizeText('Which program or result should I modify?', 'en');
  assert.equal(source.subject, null);
  for (const [language, expectedSurvival] of [['ru', true], ['hi', false], ['zh', true], ['es', true]]) {
    const trip = roundTripStatement(source, 'en', language);
    const middle = clauseStatement(trip.forward, language, null, false);
    assert.equal(middle.subject, null);
    assert.equal(sameStatementMeaning(source, middle), expectedSurvival);
    assert.equal(trip.survives, expectedSurvival);
  }
});
