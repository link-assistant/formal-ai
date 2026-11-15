# Issue #1487: semantic identity loses operand order and logical connectives

Issue: <https://github.com/link-assistant/formal-ai/issues/1487>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1497>

## 1. Collected data

| Path | Content |
| --- | --- |
| `raw-data/issue-1487.json`, `issue-1487-comments.json` | The issue as read on 2026-11-15. The only comment is a Hive Mind attempt notice. |
| `raw-data/probe-before-fix-main-57b7a3b69.txt` | `experiments/issue-1487/issue-probe.mjs` run on `main` at `57b7a3b69` |
| `raw-data/probe-after-fix.txt` | The same probe on the branch of this pull request |
| `../../../rust/tests/fixtures/proposition-structure-parity.json` | The shared JS/native fixture, written by `node experiments/issue-1487/parity-fixture.mjs --write` |

## 2. Reproduction

`node experiments/issue-1487/issue-probe.mjs <checkout>` runs the minimal
probe from the issue, then asks for a Python `count_vowels` with and without
`twice`. It needs no network and no native build.

Before the fix (`main` at `57b7a3b69`):

```text
restates("2 is less than 1." , "1 is less than 2."): true
same identity (true or false / true and false): true
negated theorem: proof_request [ 'proof_outcome:proven' ]
Return the number of vowels in the text. ->     return sum(1 for character in text if character in 'aeiouAEIOU') []
Return twice the number of vowels in the text. ->     return sum(1 for character in text if character in 'aeiouAEIOU') []
```

After the fix:

```text
restates("2 is less than 1." , "1 is less than 2."): false
same identity (true or false / true and false): false
negated theorem: proof_request [ 'proof_binding:negated', 'proof_outcome:disproven' ]
Return the number of vowels in the text. ->     return sum(1 for character in text if character in 'aeiouAEIOU') []
Return twice the number of vowels in the text. -> (no program) [
  'synthesis:obligation:unresolved_structure=repetition=repetition-twice',
  'trace:synthesis:obligation:unresolved_structure=repetition=repetition-twice'
]
```

The last line on `main` shows the coding consequence the issue predicts. The
twice requirement got the plain program. The examples were derived from the
same reading, so the program passed them and the answer looked verified.

## 3. Root causes

1. **Identity was a set of terms.** `statementIdentity` (JS) and
   `statement_identity` (native) keyed a statement on its polarity, subject and
   sorted content words. Comparison words, operators, `and`/`or`, quantifier
   order and `twice` are function words or symbols, so they never reached the
   key.
2. **Similarity certified equivalence.** `restates` in
   `dependency_summarization` fell back to term-set inclusion. When the terms of
   one statement were a subset of another's, the statement was dropped as a
   restatement. `2 is less than 1.` therefore "restated" `1 is less than 2.`.
   The statement-merge signature in `summarization_dedup` had the same flaw,
   so `not all X` and `all X not` were merged or reported as a contradiction.
3. **Proof dispatch matched keywords.** The worker matched a hardcoded Euclid
   keyword list in the request. `not` is not a keyword, so the negated request
   received the positive proof with `proof_outcome:proven`.
4. **The intent-formalization cache keyed on the normalized prompt.**
   Normalization drops operators, so `a < b` and `a <= b` shared one cached
   formalization.
5. **Synthesis dropped unread words silently.** A program was composed from the
   coding meanings a requirement evidences. Structural words that no meaning
   claimed (`twice`, `not`, `or`) were ignored. Examples derived from that same
   reading could not catch the loss.

## 4. Fix

- `proposition_structure` (JS `js/agentic/crate/proposition_structure.mjs`,
  native `rust/src/formalization/proposition_structure.rs`) reads a
  language-free surface structure: ordered relations (`less(1,2)`;
  `greater` is normalized to `less` with the operands swapped), symbolic
  operators, the and/or shape, quantifiers and their order against negation,
  repetition (`twice`, `дважды`), and an `unresolved` marker for structure it
  cannot read (`x + y * z`). The words come from the seeded grammar and
  lexicon of every supported language, not from a list of these sentences.
  Statement identity appends this structure.
- `statement_relation` gives a checked verdict: `equivalent`, `entailed`
  (conjunction elimination), `distinct` or `unresolved`, with a reason.
  `restates` and summarization accept only `equivalent` or `entailed`. Shared
  terms can still retrieve a candidate, but they cannot certify it.
- `StatementSignature` and `signatureOf` carry the structure and the negation
  scope. A contradiction needs the same terms and the same structure with the
  opposite polarity.
- `theorem_claim_binding` binds a proof request to the seeded proof-library
  proposition. A negated claim is refuted when the library marks the theorem
  `negation refuted`. A negation whose scope is not read gets a partial plan.
  Every answer carries `proof_binding:*` evidence.
- The intent-formalization cache answers only the same request, up to
  whitespace.
- `unread_structure` computes the structure left in a coding requirement after
  quoted literals, identifiers, hyphenated compounds and the surfaces of the
  discovered meanings are blanked. The browser withholds every composed
  program that leaves such structure, because the browser has no independent
  oracle. Native composition withholds the program only when its examples were
  derived from the requirement. Examples stated in the request, or documented
  by a source, stay an independent oracle, and they reject a wrong draft on
  their own. The structure stays as an open `structure:` need.

## 5. Acceptance mapping

| Acceptance criterion | Evidence |
| --- | --- |
| Ordered comparisons and the and/or pair stay distinct and are not deduplicated | `issue-1487-proposition-identity.test.mjs` (identity, restatement, dependency summarization, statement merging); native `proposition_structure_parity::ordered_and_connective_pairs_keep_distinct_signatures` |
| A negated theorem request never receives positive success | JS `never answers a negated theorem request with the positive proof`; native `proof_request::negated_theorem_request_is_refuted_never_proven`, `unread_negation_scope_returns_partial_plan` |
| Checked paraphrases still unify | JS `still unifies a checked paraphrase and a repeated mention`; fixture relations `2 is greater than 1.` and `1 < 2` against `1 is less than 2.` are `equivalent` |
| Tests cover `a-b`/`b-a`, `<`/`<=`, and/or, negation scope, repeated mention against `twice` | Fixture `structures`, `relations`, `signatures` and `unread` sections; both JS tests |
| Coding acceptance: issue text → obligation graph → candidate → independent test | `issue-1487-coding-acceptance.test.mjs`; native `coding_discovery::composition::unread_requirement_structure_withholds_the_derived_example_draft`, `stated_examples_reject_the_draft_that_drops_twice` |
| Unrecognized grammar keeps unresolved obligations | JS coding acceptance: `Frobnicate the quux so that it glorps.` is one `underivable` node with one gap, and no proven evidence; an unread coding requirement keeps `unresolved_structure` |
| JS/TS and native agree on shared fixtures, with native verification in CI | `rust/tests/fixtures/proposition-structure-parity.json`, read by both JS tests and by native `proposition_structure_parity`. The TS twins are regenerated and checked by `translate-es.mjs --check`. Native tests run in the main-branch CI, because pull-request CI does not compile Rust (`CONTRIBUTING.md`). |

## 6. Known limitations

These are outside the supported subset and are left unresolved, not guessed:

- A plain conjunction in a coding requirement (`vowels and digits`) is not
  reported as unread, because the reader gives a conjunction no shape.
- Quoted literal data is blanked even when it carries meaning
  (`vowels means "a or e"`).
- `-1` after `or` is read as a binary minus.
- Verb-role order (`Alice pays Bob` against `Bob pays Alice`) still merges in
  statement deduplication. Comparison-word paraphrases (`greater` against
  `less`) do not merge in deduplication, although the checked relation
  unifies them.
- `x is less than y` reads `y` as the Spanish conjunction. Hindi words with a
  virama are split by the tokenizer. `not less than` is consumed as a
  comparison phrase.
- The browser has no independent oracle for synthesis
  (`synthesis:verification:unverified_browser_boundary`). The JS guard reads
  the whole prompt, and the native guard reads `requirement_sentences` only.
  JS `discoveredCodingStructures` lacks the native fuzzy matching.
