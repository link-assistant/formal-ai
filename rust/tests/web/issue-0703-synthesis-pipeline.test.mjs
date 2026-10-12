// Issue #703 R703-8 / R703-9 (and the #844 statement-merge criteria it stands
// on): the multi-source synthesis pipeline in the JavaScript root. The modules
// under test are the twins of
//   rust/src/relative_meta_logic.rs        -> js/agentic/crate/relative_meta_logic.mjs
//   rust/src/summarization/dedup.rs        -> js/agentic/crate/summarization_dedup.mjs
//   rust/src/summarization/importance.rs   -> js/agentic/crate/summarization_importance.mjs
//   rust/src/summarization/recheck.rs      -> js/agentic/crate/summarization_recheck.mjs
//   rust/src/statement_verification.rs     -> js/agentic/crate/statement_verification.mjs (plan + grounding query)
//   rust/src/translation/formalization.rs  -> js/agentic/crate/translation_formalization.mjs
//
// Rust twins ported verbatim (same inputs, same assertions):
// * rust/tests/source/source_tests/relative_meta_logic/tests.rs (all);
// * rust/tests/source/source_tests/translation/formalization/tests.rs (all);
// * rust/tests/unit/specification/formalization.rs: the formalize_prompt tests
//   (the temperature-selection tests belong to a module not ported here);
// * rust/tests/unit/issue_844_statement_ranking.rs (all);
// * rust/tests/unit/issue_844_statement_merge.rs: the deduplicate / split /
//   recheck tests that do not need `merge_into_context` (world_model) or
//   `gather` (gathering.rs), which are not ported.
// The orchestration-level twin is `council_results_are_formalized_summarized_and_
// cross_checked` in issue-0703-orchestration-analysis.test.mjs.
//
// Assertions tagged `(source-derived)` were derived by hand from the Rust
// source rather than from a Rust run.

import assert from 'node:assert/strict';
import { before, describe, it } from 'node:test';

import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import {
  ASSUMED_TRUE_PRIOR, Aggregator, SourceTier, Stance, TruthValueConstants, assessAssumedTrue, combine, effectiveMass,
  evidenceIsIgnored, negateTruthValue, relativeEvidence, tierWeightPercent, truthValue, truthValueToString,
  assessmentIsProbable,
} from '../../../js/agentic/crate/relative_meta_logic.mjs';
import {
  Polarity, compareStrings, deduplicate, mergedSources, reportJustification, reportStatement, signatureKey, signatureOf,
  sourceCount, sourcedStatementFromSentence, splitMerged,
} from '../../../js/agentic/crate/summarization_dedup.mjs';
import {
  blendImportance, evidenceSummary, isContested, rank, toStatements,
} from '../../../js/agentic/crate/summarization_importance.mjs';
import {
  Verdict, checkedQuery, checkedText, recheck, reportTrace, survivors, verdictIsPresentable, verdictReason, verdictSlug,
  withheld,
} from '../../../js/agentic/crate/summarization_recheck.mjs';
import { groundingQuery } from '../../../js/agentic/crate/statement_verification.mjs';
import {
  candidateCompactSummary, candidateSlot, candidateToLinksNotation, formalizePrompt, formalizePromptCandidates,
  parseTranslationObject,
} from '../../../js/agentic/crate/translation_formalization.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const sentence = (text, source, tier) => sourcedStatementFromSentence(text, source, tier);

/** The same fact, said by `count` distinct independent sources. */
const manySources = (count, text) => Array.from({ length: count }, (_, index) => sentence(text, `source-${index}`, SourceTier.IndependentCorroboration));

describe('relative_meta_logic.rs: truth values, tiers and assessments', () => {
  it('truth_value_clamps_and_rounds', () => {
    assert.equal(truthValue(-3), 0);
    assert.equal(truthValue(7), 1);
    assert.equal(truthValue(Number.NaN), TruthValueConstants.UNKNOWN);
    assert.equal(truthValue(0.123456789), 0.123457);
    assert.equal(truthValueToString(truthValue(0.5)), '0.500000');
  });

  it('truth_value_negate_is_complement', () => {
    assert.equal(negateTruthValue(truthValue(0.2)), truthValue(0.8));
    assert.equal(negateTruthValue(TruthValueConstants.TRUE), TruthValueConstants.FALSE);
  });

  it('aggregators_have_expected_identities', () => {
    const values = [truthValue(0.2), truthValue(0.8)];
    assert.equal(combine(Aggregator.Min, values), truthValue(0.2));
    assert.equal(combine(Aggregator.Max, values), truthValue(0.8));
    assert.equal(combine(Aggregator.Average, values), truthValue(0.5));
    assert.equal(combine(Aggregator.Product, values), truthValue(0.16));
    // 1 - (1-0.2)(1-0.8) = 1 - 0.8*0.2 = 0.84
    assert.equal(combine(Aggregator.ProbabilisticSum, values), truthValue(0.84));
  });

  it('aggregators_handle_the_empty_set', () => {
    assert.equal(combine(Aggregator.Min, []), TruthValueConstants.TRUE);
    assert.equal(combine(Aggregator.Max, []), TruthValueConstants.FALSE);
    assert.equal(combine(Aggregator.Average, []), TruthValueConstants.UNKNOWN);
    assert.equal(combine(Aggregator.Product, []), TruthValueConstants.TRUE);
    assert.equal(combine(Aggregator.ProbabilisticSum, []), TruthValueConstants.FALSE);
  });

  it('unoriginal_sources_are_ignored', () => {
    const repost = relativeEvidence('aggregator.example', SourceTier.Unoriginal, Stance.Supports, 1);
    assert.equal(effectiveMass(repost), 0);
    assert.ok(evidenceIsIgnored(repost));
  });

  it('neutral_evidence_is_ignored', () => {
    assert.ok(evidenceIsIgnored(relativeEvidence('gov.example', SourceTier.OriginalFirstParty, Stance.Neutral, 1)));
  });

  it('no_evidence_keeps_the_assumed_true_prior', () => {
    const assessment = assessAssumedTrue('the sky is blue', []);
    assert.equal(assessment.posterior, truthValue(ASSUMED_TRUE_PRIOR));
    assert.ok(assessmentIsProbable(assessment));
  });

  it('trusted_support_raises_probability', () => {
    const evidence = [relativeEvidence('gov.example', SourceTier.OriginalFirstParty, Stance.Supports, 0.9)];
    assert.ok(assessAssumedTrue('official policy X', evidence).posterior > ASSUMED_TRUE_PRIOR);
  });

  it('contradicting_original_evidence_lowers_probability', () => {
    const evidence = [relativeEvidence('original.journal', SourceTier.OriginalJournalism, Stance.Contradicts, 0.9)];
    assert.ok(assessAssumedTrue('disputed claim', evidence).posterior < ASSUMED_TRUE_PRIOR);
  });

  it('unoriginal_reposts_do_not_move_probability', () => {
    const withReposts = [
      relativeEvidence('mirror.a', SourceTier.Unoriginal, Stance.Supports, 1),
      relativeEvidence('mirror.b', SourceTier.Unoriginal, Stance.Contradicts, 1),
    ];
    const assessment = assessAssumedTrue('viral claim', withReposts);
    assert.equal(assessment.posterior, truthValue(ASSUMED_TRUE_PRIOR));
    assert.equal(assessment.ignored_sources.length, 2);
  });

  it('first_party_outweighs_corroboration_for_the_same_strength', () => {
    const strong = assessAssumedTrue('claim', [relativeEvidence('subject.itself', SourceTier.OriginalFirstParty, Stance.Supports, 0.8)]);
    const weak = assessAssumedTrue('claim', [relativeEvidence('second.hand', SourceTier.IndependentCorroboration, Stance.Supports, 0.8)]);
    assert.ok(strong.posterior > weak.posterior);
  });

  it('independent_support_reinforces_rather_than_averages', () => {
    const one = relativeEvidence('a', SourceTier.IndependentCorroboration, Stance.Supports, 0.5);
    const two = relativeEvidence('b', SourceTier.IndependentCorroboration, Stance.Supports, 0.5);
    assert.ok(assessAssumedTrue('claim', [one, two]).posterior > assessAssumedTrue('claim', [one]).posterior);
  });

  it('posterior_stays_within_bounds', () => {
    const overwhelming = [
      relativeEvidence('a', SourceTier.OriginalFirstParty, Stance.Contradicts, 1),
      relativeEvidence('b', SourceTier.OriginalJournalism, Stance.Contradicts, 1),
    ];
    const { posterior } = assessAssumedTrue('false claim', overwhelming);
    assert.ok(posterior >= 0 && posterior <= 1);
    assert.ok(posterior < 0.1);
  });

  it('tier_weights_are_the_published_percentages', () => {
    assert.deepEqual(Object.values(SourceTier).map(tierWeightPercent), [100, 85, 50, 0]);
  });
});

describe('translation/formalization.rs: formalize_prompt', () => {
  it('relation_prompt_extracts_subject_predicate_object', () => {
    const candidate = formalizePrompt('apple is a fruit', 'en');
    assert.equal(candidateSlot(candidate, 'subject').anchor.id, 'wikidata:Q89');
    assert.equal(candidateSlot(candidate, 'predicate').anchor.id, 'wikidata:P31');
    assert.equal(candidateSlot(candidate, 'object').anchor.id, 'wikidata:Q3314483');
  });

  it('russian_translation_prompt_uses_multilingual_label_table', () => {
    const candidate = formalizePrompt('переведи яблоко на английский', 'ru');
    assert.equal(candidateSlot(candidate, 'predicate').anchor.id, 'wikidata:P5972');
    assert.equal(candidateSlot(candidate, 'object').anchor.id, 'wikidata:Q89');
  });

  it('source_first_translation_formalizes_only_the_source_proposition', () => {
    assert.equal(
      parseTranslationObject('любая формальная система либо неполна, либо противоречива - translate to english'),
      'любая формальная система либо неполна, либо противоречива',
    );
  });

  it('arbitrary_statement_maps_predicate_and_nouns_to_wikidata_ids', () => {
    const candidate = formalizePrompt('apple is a fruit', 'en');
    const subject = candidateSlot(candidate, 'subject');
    const predicate = candidateSlot(candidate, 'predicate');
    const object = candidateSlot(candidate, 'object');
    assert.equal(subject.anchor.kind, 'wikidata_item');
    assert.equal(subject.anchor.id, 'wikidata:Q89');
    assert.equal(predicate.anchor.kind, 'wikidata_property');
    assert.equal(predicate.anchor.id, 'wikidata:P31');
    assert.equal(object.anchor.kind, 'wikidata_item');
    assert.equal(object.anchor.id, 'wikidata:Q3314483');
    const lino = candidateToLinksNotation(candidate);
    assert.ok(lino.includes('subject_q "wikidata:Q89"'), lino);
    assert.ok(lino.includes('predicate_p "wikidata:P31"'), lino);
    assert.ok(lino.includes('object_q "wikidata:Q3314483"'), lino);
    assert.equal(
      lino,
      [
        'formalization_candidate',
        '  source_text "apple is a fruit"',
        '  language "en"',
        '  score "976"',
        '  subject_surface "apple"',
        '  subject_q "wikidata:Q89"',
        '  subject_score "980"',
        '  subject_source "label:wikidata-item"',
        '  predicate_surface "is a"',
        '  predicate_p "wikidata:P31"',
        '  predicate_score "970"',
        '  predicate_source "label:property"',
        '  object_surface "fruit"',
        '  object_q "wikidata:Q3314483"',
        '  object_score "980"',
        '  object_source "label:wikidata-item"',
        '',
      ].join('\n'),
    );
  });

  it('an_ambiguous_copula_also_yields_the_subclass_reading (source-derived)', () => {
    const candidates = formalizePromptCandidates('apple is a fruit', 'en');
    assert.deepEqual(candidates.map((candidate) => [candidate.score, candidateSlot(candidate, 'predicate').anchor.id]), [[976, 'wikidata:P31'], [971, 'wikidata:P279']]);
    assert.equal(candidateCompactSummary(candidates[0]), 'subject=wikidata:Q89 predicate=wikidata:P31 object=wikidata:Q3314483');
  });

  it('action_prompt_maps_translation_verb_to_wikidata_property', () => {
    const candidate = formalizePrompt('translate apple to Russian', 'en');
    const predicate = candidateSlot(candidate, 'predicate');
    assert.equal(predicate.anchor.kind, 'wikidata_property');
    assert.equal(predicate.anchor.id, 'wikidata:P5972');
    assert.ok(candidate.slots.some((slot) => slot.anchor.id === 'wikidata:Q89'), JSON.stringify(candidate));
  });

  it('supported_language_prompts_map_local_surfaces_to_same_language_independent_ids', () => {
    for (const [language, prompt] of [
      ['en', 'translate apple to Russian'],
      ['ru', 'переведи яблоко на английский'],
      ['hi', 'सेब का हिंदी में अनुवाद करो'],
      ['zh', '把 苹果 翻译成中文'],
    ]) {
      const candidate = formalizePrompt(prompt, language);
      assert.equal(candidateSlot(candidate, 'predicate').anchor.id, 'wikidata:P5972', `${language} prompt should map the translation verb to P5972`);
      assert.equal(candidateSlot(candidate, 'object').anchor.id, 'wikidata:Q89', `${language} prompt should map the translated surface to Q89`);
    }
  });

  it('unmodeled_dictionary_terms_fall_back_to_wiktionary_surfaces', () => {
    const candidate = formalizePrompt('what does digress mean?', 'en');
    const term = candidateSlot(candidate, 'subject');
    assert.equal(term.surface, 'digress');
    assert.equal(term.anchor.kind, 'wiktionary_entry');
    assert.equal(term.anchor.id, 'wiktionary:en:digress');
    assert.deepEqual(candidate.unresolved_terms, []);
  });

  it('unanchored_unknown_terms_are_flagged_for_later_translation_gaps', () => {
    const candidate = formalizePrompt('define zzqxqv', 'en');
    const term = candidateSlot(candidate, 'subject');
    assert.equal(term.surface, 'zzqxqv');
    assert.equal(term.anchor.kind, 'raw_text');
    assert.equal(term.anchor.id, 'raw:zzqxqv');
    assert.deepEqual(candidate.unresolved_terms, ['zzqxqv']);
    assert.ok(candidateToLinksNotation(candidate).includes('formalization_unresolved'), candidateToLinksNotation(candidate));
  });

  it('a_prompt_with_no_anchorable_surface_yields_the_empty_candidate (source-derived)', () => {
    const candidate = formalizePrompt('Rust is memory safe. The Moon is cheese.', 'en');
    assert.equal(candidate.score, 0);
    assert.deepEqual(candidate.slots, []);
    assert.equal(candidateToLinksNotation(candidate), 'formalization_candidate\n  source_text "Rust is memory safe. The Moon is cheese."\n  language "en"\n  score "0"\n');
    assert.equal(candidateCompactSummary(candidate), 'empty');
  });
});

describe('summarization/dedup.rs: deduplicate (issue #844)', () => {
  it('n_sources_asserting_one_fact_yield_one_statement_with_a_justification_link', () => {
    const report = deduplicate(manySources(9, 'The parser is fast.'));
    assert.equal(report.statements.length, 1, 'nine sources asserting one fact must yield one statement');
    const node = report.statements[0];
    assert.equal(sourceCount(node), 9);
    assert.equal(node.variants.length, 9);
    const links = reportJustification(report, node.id);
    assert.equal(links.length, 8, 'one link per absorbed sentence');
    for (const link of links) {
      assert.equal(link.representative, node.id);
      assert.equal(link.absorbed, 'The parser is fast.');
      assert.equal(link.justification, signatureKey(node.signature));
      assert.ok(mergedSources(node).includes(link.source));
    }
    for (let index = 0; index < 9; index += 1) assert.ok(mergedSources(node).includes(`source-${index}`), `source-${index} lost`);
    assert.equal(signatureKey(node.signature), 'asserted:fast parser');
  });

  it('wording_differences_merge_but_extra_content_does_not', () => {
    const report = deduplicate([
      sentence('The parser is fast.', 'a', SourceTier.OriginalFirstParty),
      sentence('Parser is fast', 'b', SourceTier.OriginalJournalism),
      sentence('the fast parser', 'c', SourceTier.OriginalJournalism),
      sentence('The parser is fast enough.', 'd', SourceTier.IndependentCorroboration),
    ]);
    assert.equal(report.statements.length, 2, JSON.stringify(report.statements.map((node) => signatureKey(node.signature))));
    assert.equal(sourceCount(report.statements[0]), 3);
    assert.equal(sourceCount(report.statements[1]), 1);
  });

  it('inflected_wordings_stay_separate_because_the_merge_does_not_stem', () => {
    const report = deduplicate([
      sentence('The library ships a solver.', 'a', SourceTier.OriginalFirstParty),
      sentence('The library does not ship a solver.', 'b', SourceTier.OriginalFirstParty),
    ]);
    assert.equal(report.statements.length, 2);
    assert.deepEqual(report.contradictions, []);
    const denied = report.statements.find((node) => node.signature.polarity === Polarity.Denied);
    assert.deepEqual(denied.signature.terms, ['library', 'ship', 'solver']);
  });

  it('a_merge_that_conflates_two_facts_can_be_split', () => {
    const report = deduplicate([
      sentence('Rust calls Python.', 'a', SourceTier.OriginalFirstParty),
      sentence('Python calls Rust.', 'b', SourceTier.OriginalFirstParty),
    ]);
    assert.equal(report.statements.length, 1, 'the conflating merge happened');
    const mergedId = report.statements[0].id;
    assert.equal(reportJustification(report, mergedId).length, 1);

    assert.ok(splitMerged(report, mergedId), 'the merge must be reversible');
    assert.equal(report.statements.length, 2, 'one node per absorbed variant');
    assert.equal(reportStatement(report, mergedId), null, 'merged node is gone');
    assert.deepEqual(reportJustification(report, mergedId), [], 'the merge links are gone with it');
    assert.deepEqual(report.statements.map((node) => node.representative.text), ['Rust calls Python.', 'Python calls Rust.']);
    for (const node of report.statements) assert.equal(node.variants.length, 1, 'nothing is merged any more');
    assert.ok(!splitMerged(report, report.statements[0].id));
    assert.ok(!splitMerged(report, 'statement_deadbeef'));
    assert.deepEqual(report.statements.map((node) => node.id), [`${mergedId}_0`, `${mergedId}_1`]);
  });

  it('contradictions_pair_an_affirmed_fact_with_its_denial', () => {
    const report = deduplicate([
      sentence('The release is reproducible.', 'vendor', SourceTier.OriginalFirstParty),
      sentence('The release is reproducible.', 'review', SourceTier.OriginalJournalism),
      sentence('The release is not reproducible.', 'auditor', SourceTier.OriginalFirstParty),
    ]);
    assert.equal(report.contradictions.length, 1, 'the affirmed/denied twins must be recognized');
    assert.deepEqual(report.contradictions[0].terms, ['release', 'reproducible']);
    const [asserted, denied] = report.statements;
    assert.equal(report.contradictions[0].asserted, asserted.id);
    assert.equal(report.contradictions[0].denied, denied.id);
    assert.deepEqual(report.sources, ['vendor', 'review', 'auditor']);
    const ranked = rank(report);
    assert.equal(evidenceSummary(ranked.find((item) => item.statement.id === asserted.id), report.sources.length), 'asserted by 2 of 3 sources, denied by 1');
    const disputed = toStatements(ranked, report.sources.length).map((item) => item.text);
    assert.ok(disputed.some((text) => text.includes('disputed')), disputed.join(' | '));
  });

  it('a_repeated_sentence_from_one_source_adds_nothing_and_function_word_only_text_never_merges', () => {
    const report = deduplicate([
      sentence('The parser is fast.', 'a', SourceTier.OriginalFirstParty),
      sentence('The parser is fast.', 'a', SourceTier.OriginalFirstParty),
      sentence('It is the.', 'a', SourceTier.OriginalFirstParty),
      sentence('It is the.', 'b', SourceTier.OriginalFirstParty),
      sentence('   ', 'c', SourceTier.OriginalFirstParty),
    ]);
    assert.equal(report.links.length, 0);
    assert.deepEqual(report.sources, ['a', 'b']);
    assert.equal(report.statements[0].variants.length, 1);
    assert.deepEqual(signatureOf('It is the.').terms, []);
    assert.equal(report.statements.length, 3, 'two function-word-only sentences never merge');
  });

  it('signature_order_is_utf8_byte_order (source-derived)', () => {
    // 'z' (U+007A) < 'ａ' (U+FF41) < '😀' (U+1F600): a UTF-16 code unit sort would put the emoji first.
    assert.deepEqual(['😀', 'ａ', 'z'].sort(compareStrings), ['z', 'ａ', '😀']);
    assert.deepEqual(['b', 'a', 'b'].sort(compareStrings), ['a', 'b', 'b']);
  });
});

describe('summarization/importance.rs: rank (issue #844)', () => {
  it('ranking_reflects_observed_frequency_and_source_stance', () => {
    const observations = manySources(8, 'The parser is fast.');
    observations.push(sentence('The manual is long.', 'lonely', SourceTier.IndependentCorroboration));
    const report = deduplicate(observations);
    const ranked = rank(report);

    const widely = ranked.find((item) => item.statement.representative.text.includes('parser'));
    const lonely = ranked.find((item) => item.statement.representative.text.includes('manual'));
    assert.equal(widely.score.prior, lonely.score.prior);
    assert.ok(widely.score.coverage > lonely.score.coverage, `${widely.score.coverage} vs ${lonely.score.coverage}`);
    assert.ok(widely.score.weight > lonely.score.weight);
    assert.equal(ranked[0].statement.representative.text, widely.statement.representative.text);
    assert.equal(evidenceSummary(widely, report.sources.length), 'asserted by 8 of 9 sources');

    const contested = manySources(8, 'The parser is fast.');
    contested.push(sentence('The parser is not fast.', 'denier', SourceTier.OriginalJournalism));
    const contestedReport = deduplicate(contested);
    const demoted = rank(contestedReport).find((item) => item.statement.signature.polarity === 'asserted');
    assert.equal(demoted.score.coverage, widely.score.coverage);
    assert.ok(demoted.score.agreement < 100, 'agreement must fall');
    assert.ok(demoted.score.weight < widely.score.weight, 'a denied fact must rank below the same fact uncontested');
    assert.equal(evidenceSummary(demoted, contestedReport.sources.length), 'asserted by 8 of 9 sources, denied by 1');
    assert.ok(isContested(demoted));
  });

  it('an_unoriginal_mirror_adds_no_probability', () => {
    const original = deduplicate([sentence('The parser is fast.', 'first-party', SourceTier.OriginalFirstParty)]);
    const mirrored = [sentence('The parser is fast.', 'first-party', SourceTier.OriginalFirstParty)];
    for (let index = 0; index < 5; index += 1) mirrored.push(sentence('The parser is fast.', `mirror-${index}`, SourceTier.Unoriginal));
    const alone = rank(original)[0].probability;
    const echoed = rank(deduplicate(mirrored))[0].probability;
    assert.ok(Math.abs(alone - echoed) < Number.EPSILON, `five unoriginal mirrors must not move the posterior: ${alone} vs ${echoed}`);
  });

  it('unoriginal_repetition_cannot_outrank_an_authoritative_source', () => {
    const observations = [sentence('The release is signed.', 'first-party', SourceTier.OriginalFirstParty)];
    for (let index = 0; index < 8; index += 1) observations.push(sentence('The rumour is widespread.', `mirror-${index}`, SourceTier.Unoriginal));
    const ranked = rank(deduplicate(observations));
    const authoritative = ranked.find((item) => item.statement.representative.text.includes('release'));
    const repeated = ranked.find((item) => item.statement.representative.text.includes('rumour'));
    assert.equal(repeated.score.evidence, 0, 'unoriginal assertions carry no ranking evidence');
    assert.equal(repeated.score.authority, 0);
    assert.equal(authoritative.score.authority, 100);
    assert.ok(authoritative.score.weight > repeated.score.weight, `${JSON.stringify(authoritative.score)} vs ${JSON.stringify(repeated.score)}`);
    assert.equal(ranked[0].statement.representative.text, authoritative.statement.representative.text);
  });

  it('blend_is_integer_arithmetic_capped_at_one_hundred (source-derived)', () => {
    assert.deepEqual(blendImportance(70, 100, 100, 100), { prior: 70, coverage: 100, authority: 100, agreement: 100, evidence: 100, weight: 90 });
    assert.deepEqual(blendImportance(90, 81, 50, 50), { prior: 90, coverage: 81, authority: 50, agreement: 50, evidence: 20, weight: 43 });
    assert.deepEqual(blendImportance(100, 0, 0, 0), { prior: 100, coverage: 0, authority: 0, agreement: 0, evidence: 0, weight: 33 });
  });
});

describe('summarization/recheck.rs: the capture-independent preflight (issue #844)', () => {
  it('a_statement_no_trusted_source_asserts_is_withheld_but_kept', () => {
    const report = recheck(rank(deduplicate([
      sentence('The build is reproducible.', 'vendor', SourceTier.OriginalFirstParty),
      sentence('A rumour circulates.', 'echo', SourceTier.Unoriginal),
    ])));
    const held = withheld(report);
    assert.equal(held.length, 1, reportTrace(report));
    assert.equal(checkedText(held[0]), 'A rumour circulates.');
    assert.equal(verdictSlug(held[0].verdict), 'unsupported');
    assert.ok(!verdictIsPresentable(held[0].verdict));
    const kept = survivors(report);
    assert.equal(kept.length, 1);
    assert.equal(checkedText(kept[0]), 'The build is reproducible.');
    assert.equal(verdictSlug(kept[0].verdict), 'confirmed');
    for (const item of report.checked) assert.ok(checkedQuery(item).includes('fact check source'), checkedQuery(item));
  });

  it('verdicts_follow_the_assessment (source-derived)', () => {
    const report = recheck(rank(deduplicate([
      sentence('The release is reproducible.', 'vendor', SourceTier.OriginalFirstParty),
      sentence('The release is not reproducible.', 'auditor', SourceTier.OriginalJournalism),
      sentence('The cache is warm.', 'a', SourceTier.IndependentCorroboration),
      sentence('The cache is warm.', 'b', SourceTier.IndependentCorroboration),
    ])));
    assert.deepEqual(report.checked.map((item) => [checkedText(item), verdictSlug(item.verdict)]), [
      ['The cache is warm.', Verdict.Confirmed],
      ['The release is reproducible.', Verdict.Refuted],
      ['The release is not reproducible.', Verdict.Refuted],
    ]);
    assert.deepEqual(survivors(report).map(checkedText), ['The cache is warm.']);
    // Merged evidence always contradicts at full strength, so a still-probable
    // (contested) verdict needs a weaker contradiction than the merge produces.
    const weakDenial = recheck([{
      statement: { representative: { text: 'The disk is full.' } },
      evidence: [
        relativeEvidence('a', SourceTier.OriginalFirstParty, Stance.Supports, 1),
        relativeEvidence('b', SourceTier.IndependentCorroboration, Stance.Contradicts, 0.2),
      ],
    }]);
    assert.deepEqual(weakDenial.checked.map((item) => [verdictSlug(item.verdict), item.plan.assessment.posterior]), [[Verdict.Contested, 0.9]]);
    assert.ok(verdictIsPresentable(weakDenial.checked[0].verdict));
    assert.equal(verdictReason(Verdict.Unsupported), 'no trusted source asserts it');
    assert.equal(verdictReason(Verdict.Contested), 'sources disagree, still probable');
    assert.equal(report.checked[0].plan.assessment.posterior > 0.5, true);
  });

  it('the_grounding_query_condenses_whitespace_and_quotes_the_statement (source-derived)', () => {
    assert.equal(groundingQuery('The  moon\n is   cheese.'), '"The moon is cheese." fact check source');
    const trace = reportTrace(recheck(rank(deduplicate([sentence('The build is reproducible.', 'vendor', SourceTier.OriginalFirstParty)]))));
    assert.equal(trace, 'verdict=confirmed prior=0.600000 support=1.000000 contradiction=0.000000 posterior=1.000000 ignored=0 sources=1 denied=0');
  });
});
