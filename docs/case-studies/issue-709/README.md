# Issue #709: Ranked Search Statements With Provenance

Issue [#709](https://github.com/link-assistant/formal-ai/issues/709) closes the
gap between search retrieval and an answer. Before this change, the Rust path
called `execute_source_research(..., 0)` and rendered one link per provider row;
it fetched no result pages, formalized no claims, and could neither merge a
Russian statement with its English equivalent nor expose a contradiction. The
browser had a separate RRF and entity-level URL deduplicator, but it likewise
presented opaque result snippets. Telegram merely escaped that Markdown. This
was the root cause behind the title-only result reported in related issue #827.

Issue #844 supplied the missing statement-level merge, evidence-weighted
ranking, source tiers, contradiction context, and exact-capture composition.
Issue #709 composes those pieces at the production search boundary and gives
the browser worker the equivalent data-driven cross-language operation.

## The operation

`execute_search_fusion` performs one inspectable sequence:

1. Search through the existing provider/RRF boundary and capture up to three
   exact result pages with `CachedSourceClient`.
2. Strip non-visible HTML bodies and split every hit and page into statements.
   Language is detected after that split, so one mixed capture can produce
   correct English and Russian receipts. `formalize_prompt` records complete
   subject/predicate/object Wikidata anchors; incomplete grounding deliberately
   falls back to conservative lexical terms.
3. Deformalize a completely grounded foreign statement from the same meaning
   lexicon into the query language. The source card keeps the original quote.
4. Feed role-qualified meaning links into #844's `merge_into_formal_context`.
   `OriginalFirstParty`, `IndependentCorroboration`, and `Unoriginal` use the
   relative-meta-logic weights. Reposts are traced but add no evidence, and
   byte-identical page captures are deterministically assigned to the highest
   tier (then earliest retrieval rank) while every mirror is demoted.
5. Select no more than three meanings. A contradiction is one meaning with two
   polarities, so both ranked sides remain visible with posteriors and
   `conflict:source_disagreement` evidence.
6. Project normalized source cards: URL, title, quote, read-more URL, language,
   tier, provider ranks, and exact-capture SHA-256.

CLI and HTTP call this Rust operation directly. Telegram converts the same
Markdown links, code spans, and quoted fragments to its safe HTML subset. The
browser sends its captured excerpts through an 18-line transport bridge to
`web_search_fusion_core`, compiled into the existing Rust→WASM worker. That
core consumes the same seed meaning anchors, function words, negation cues,
source-tier slugs, and Agent-authored semantic-role order. It detects the
language of each excerpt statement from those anchors and can therefore merge
mixed-language captures before projecting natural Hindi S–O–V or the declared
fallback order. No second JavaScript solver owns the projection.

![A Russian-only source deformalized into an English ranked statement](../../screenshots/issue-709-search-fusion.png)

## Reproduction and tests

The pre-fix reproduction is captured by the initial failing build in
`test-logs/red.txt`: the public fusion symbols did not exist. Later red logs pin
the fragment-language, hidden-HTML/mirror, Hindi-order, and duplicate-learning
boundaries before their fixes. Run the focused acceptance suite with:

```console
cargo test --test unit issue_709
cd tests/e2e
npx playwright test --config=playwright.local.config.js tests/issue-709.spec.js
```

The Rust fixture has three sources: an original English claim, independent
Russian corroboration, and a repost. Its second execution is offline and must
produce byte-identical presentation, trace, and learning proposal without any
new transport request. Separate fixtures pin a foreign-only decisive fact and
an original-versus-independent contradiction. The browser tests pin the same
two visible outcomes; the screenshot above is produced from the foreign-only
test when `ISSUE_709_SCREENSHOT=1`.

`examples/issue_709_formalization_probe.rs` is the minimum diagnostic used to
verify that English and Russian variants converge on `Q89/P31/Q3314483` before
the fusion code was written.

## Determinism, live access, and learning

No neural inference is used. Complete semantic keys are sorted, merged nodes
are content-addressed, rankings have stable tie-breaks, and the presented
meaning bound is constant. Exact search/page captures are content-addressed;
cache-hit state is omitted from the replay trace. Production traffic remains
behind `FORMAL_AI_LIVE_API`, while tests provide captured transports.

The generated `learning_proposal` contains every capture receipt and every
formalization, merge, and rank record. Accepted executions enter an append-only
`SearchFusionLearningFrontier`; a data-authored policy requires two distinct
execution fingerprints before it can infer the stable seven-stage recipe.
Renaming the same run cannot satisfy that threshold. The candidate remains
inert until a nonempty held-out suite has passed with zero failures and a named
reviewer approves it. Only then can a content-addressed Links Notation ledger
restore the recipe and execute an unseen equivalent task.

There is no silently pre-approved production recipe in this change. The
committed auto-learning report says `awaiting_human_review`. It was derived in
three rounds by Formal AI behind the real external Agent CLI from the persisted
associative network of four failures and four amendments, and its bytes match
the in-process report recipe. The deterministic held-out fixture uses tomato
taxonomy after apple-taxonomy and parser-speed training runs.

## Traceability and authorship

[`requirements.md`](requirements.md) maps R709-01…R709-09 to named tests.
`raw-data/` preserves the issue, its cross-link comment, the deeper-implementation
review, and all three GitHub PR comment channels as read during implementation.
The reviewed decomposition and the first real external Agent CLI transcript
live under `self-hosting-authorship/`; six additional bounded sessions and their
server/CLI transcripts live under `agent-cli-evidence/`.

Attribution stays leaf-specific: six implementation/test leaves are
Codex-authored, while Formal AI authored seven of thirteen reviewed leaves
(54%): the original provenance invariant, learning contract, source policy,
held-out fixture, associative observation network, language grammar, and derived
learning report. The decomposition records every session ID and evidence path;
it does not attribute the Rust implementation to Formal AI.
