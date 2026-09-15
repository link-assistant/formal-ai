# Plan 13 — Merged-PR carry-over and issue coverage (#1138)

The maintainer's rule for this batch, quoted from [`README.md`](README.md):

> all not yet fully done that from all previous merged pull request must be
> planned here to actually fully done it here.

This plan is the ledger that makes that rule auditable. It sweeps every merged
pull request in the repository's history for statements of partial delivery,
declared boundaries, deferrals and honest gaps; quotes each one; assigns it to a
bottleneck B1–B12 from issue #1138 (or to `other` with a label); says whether a
later merged pull request already finished it; and names the plan leaf in this
pull request that must absorb it. It then does the same for the open issues, so
the final `Closes #N` list is derived from evidence rather than optimism.

Nothing here is paraphrased. Every remainder is the merged pull request's own
words, trimmed only at a sentence boundary and marked with `…` when trimmed.

## Method

### Commands used

```bash
# Every pull request the repository has ever had, with merge state.
gh api --paginate "repos/link-assistant/formal-ai/pulls?state=all&per_page=100" \
  --jq '.[] | [.number, (.merged_at // "null"), .state] | @tsv'      # 517 rows

# Every merged pull request with its body and the issues it closes.
gh pr list --repo link-assistant/formal-ai --state merged --limit 1000 \
  --json number,title,mergedAt,body,closingIssuesReferences           # 500 rows

# Bodies split one file per pull request, then swept case-insensitively.
grep -n -i -E 'out of scope|honest boundary|honestly|not yet|remains open|remain open|deferred|defers|follow-up|follow up|tracked in|partial|does not yet|do not yet|not delivered|not achieved|narrow|bounded first slice|future work|left for|continuation|pending|unchecked|blocked|still open|still not|remains a gap|remaining|no attempt|beyond this PR|next PR|later PR|not covered|does not cover|limitation' bodies/*.md

# Second, higher-signal sweep to catch remainders the first pattern missed.
grep -n -i -E 'not wired|cannot yet|is not implemented|no implementation|remains a gap|remains open|still open|still not|honest boundary|out of scope|not achieved|not delivered|left for a|its own issue|its own change|separate change|a later slice|next slice|to be done|TODO' bodies/*.md

# Closing comments of every closed issue whose title carries an E-number.
gh issue list --repo link-assistant/formal-ai --state closed --limit 800 \
  --json number,title,closedAt                                        # 557 rows
jq -r '.[] | select(.title | test("\\bE[0-9]+\\b")) | .number'        # 93 issues
gh api "repos/link-assistant/formal-ai/issues/<N>/comments?per_page=100"

# Open issues and open pull requests.
gh issue list --repo link-assistant/formal-ai --state open --limit 200 \
  --json number,title,body,labels                                     # 61 rows
gh pr list --repo link-assistant/formal-ai --state open \
  --json number,title,createdAt,updatedAt,mergeable,body,changedFiles # 6 rows
```

### Counts

| Measure | Value |
| --- | --- |
| Pull requests in the repository, all states | 517 |
| Merged pull requests scanned (the complete set) | 500 |
| Closed-unmerged pull requests (not scanned; they delivered nothing) | 12 |
| Open pull requests | 6 (#1139 is this one) |
| Bodies matching the broad remainder pattern | 441 lines across 204 pull requests |
| Bodies matching the higher-signal pattern | 43 lines across 33 pull requests |
| Hits discarded as false positives | 344 |
| Substantive remainder statements recorded below | 97, across 71 pull requests |
| Of those: already completed by a later merged pull request | 14 |
| Of those: **still open, and therefore this pull request's obligation** | 83 |
| Closed issues carrying an E-number, comments swept | 93 |
| E-numbered issues whose comments state a remainder | 4 (#671, #702, #703, #709) |
| Open issues classified for coverage | 61 (60 plus #1138 itself) |

The 344 discarded hits fall into five classes, listed so the discard is
auditable rather than asserted:

1. **"follow-up" naming a product feature**, not a deferral — the
   `follow_up_probability` setting, `courtesy_response` follow-ups, coreference
   follow-ups, `research_result_followup`. 181 hits, chiefly #60, #176, #199,
   #345, #369–#375, #387, #413, #430, #565, #569, #573, #584, #585, #599.
2. **"Review follow-up" / "CI follow-up" section headers** recording work that
   landed in the same pull request. 84 hits, chiefly #104, #154, #222, #470,
   #525, #806, #853, #854, #965.
3. **"partial" / "narrow" used descriptively** of a fix that is complete —
   "partial grants", "partial rerun", "narrowly scoped lint", "partial
   preferences". 41 hits.
4. **"remaining" counting tests that passed** — "the remaining 110-test
   multilingual group then passed", "all remaining integration targets passed".
   26 hits.
5. **Upstream constraints in another project's code**, correctly filed there and
   outside this repository's reach — `agent-commander` approve-each, Playwright
   #33031, `meta-language` query IR, crates.io 429. 12 hits, kept out of the
   table but named in "Risks and open questions".

### How a remainder was classified

A hit became a table row when the sentence asserts that something the pull
request was asked to do was **not done in that pull request**. Sentences that
assert an upstream project's limitation, a policy decision reserved for the
maintainer, or a test-count observation were discarded. Where a later merged
pull request demonstrably finished the item, the row says "completed by #N" and
the item is dropped from the grouped obligations; where no later pull request
names the same subject, the row says "still open" and the item appears in the
grouped section and in a plan leaf.

## Carry-over table

Ordered by pull request number. `id` is the reference used by the grouped
section below.

| id | PR | merged | closes | quoted remainder | bottleneck | status | planned in |
| --- | --- | --- | --- | --- | --- | --- | --- |
| C1 | #9 | 2026-05-12 | #8 | "Docker and `tsc` are not installed in the prepared runtime, so Docker-backed isolation was not used and TypeScript answers report that limitation instead of claiming execution." | B6 | still open | 06 (docker execution pipeline, #930/#937) |
| C2 | #13 | 2026-05-14 | #12 | "remaining `#[ignore = \"MVP-target: …\"]` tests still document the road ahead without blocking CI" | other — ignored-test debt | still open | new leaf: enumerate every `#[ignore]` still in the tree, un-ignore or delete with a reason (plan 11 ledger) |
| C3 | #15 | 2026-05-15 | #14 | "Hello-world programs for non-JavaScript languages now honestly report that the browser sandbox cannot invoke the toolchain instead of faking execution status." | B6 | still open | 06 (browser runtime honesty; plan 02 §7 records the same gap) |
| C4 | #99 | 2026-05-17 | #68 | "Keeps `link-calculator` delegation first, then uses a narrow formal-ai fallback for calculator-missing linear equations." | B8 | still open — #968 later recorded ten equation limitations | 08 (equation corpus re-measured) |
| C5 | #208 | 2026-05-21 | #207 | "`cargo test` (406 unit tests passing, 69 ignored as tracked future work)" | other — ignored-test debt | still open | same leaf as C2 |
| C6 | #245 | 2026-05-29 | #244 | "**With E1-E34 all merged, no vision-planning epic remains open for issue #244.**" | B11 | still open — contradicted by #710's audit ("of 183 closed issues ≤ #350, only ~15% show clear delivery evidence") | 11 (the record of truth must not carry a completeness claim a later audit refuted) |
| C7 | #319 | 2026-05-28 | #313 | "the same benchmark reports `passed=3 failed=2`; the remaining failures are the programming fixtures (`humaneval_0_has_close_elements`, `mbpp_2_similar_elements`)" | B2 | completed by #888 (first 20 of each suite at 20/20) | — |
| C8 | #320 | 2026-05-28 | #314 | "reports `passed=3 failed=2 total=5`; the remaining failures are the existing programming fixtures" | B2 | completed by #888 | — |
| C9 | #331 | 2026-05-29 | #330 | "### Deferred (honest scope note) … reasoning as link-substitution rules over `doublets-rs` convertible to Rust/JS/WASM, isolated JS evaluation, in-browser Rust→WASM compilation, a browser Linux VM (`rust-web-box`), and dockerized execution with snapshots/replay (`box`/`start`)." | B6 + B9 | still open — re-filed as #936 (closed), #937 (open), #670 (open) | 06 (docker execution, snapshot/replay) + 09 (links substitution rules as the executable system of record) |
| C10 | #366 | 2026-05-30 | #355 | "The diagnostics golden fixture remains deferred as described in #355 until #360 exposes full reasoning-step diagnostics." | other — diagnostics fixture | completed by #372 (#360) | — |
| C11 | #396 | 2026-06-10 | #395 | "The remaining missing upstream feature is **reported**: … cannot yet emit code from a links network built via `insert_*`" | other — `meta-language` upstream | still open upstream | 09 (records the dependency; the upstream fix is not this pull request's to make) |
| C12 | #399 | 2026-06-10 | #398 | "### Deferred to follow-up issues — These review items are intentionally out of scope for this round … **Rule 6:** no hardcoded domain-data string literals in `src/`." | B9 | still open — 1,286 allowlisted prose literals today | 09 (allowlist ratchet strictly downward) |
| C13 | #399 | 2026-06-10 | #398 | "**Grounding floor (partial, R282).** … Grounding every remaining meaning to an external source — and sourcing per-language surfaces from those records instead of hand-typed …" | B1 + B4 | still open | 01 L5–L7 (extractors and provenance), 04 (concepts stored, not sentences) |
| C14 | #399 | 2026-06-10 | #398 | "**#3** reverse-dictionary / part-of-speech-as-meaning (the genuine residue: `verb`, `noun_phrase` used as POS tags but not yet defined meanings)." | B4 | still open | 04 (every unresolved concept emits a need) |
| C15 | #399 | 2026-06-10 | #398 | "**#5** provable 1:1 meaning↔type via relative-meta-logic." | B4 | still open | 04 + 12 (relative-meta-logic as a selection method, not prose) |
| C16 | #399 | 2026-06-10 | #398 | "**#6** minimal seed + on-demand expansion." | B1 + B2 | still open — the seed grew instead | 01 (on-demand lookup) + 02 (idiom catalog becomes a deletable bootstrap) |
| C17 | #450 | 2026-06-13 | #449 | "the spec suite was doubled first (close to 100% coverage, locking the public API), **then** the remaining paper mechanisms were ported" | other — ordering note | completed in the same pull request | — |
| C18 | #452 | 2026-06-13 | #451 | "`docs/case-studies/issue-451/symbolic-ai-best-practices.md` — the best-practice audit (R299), now **16 applied / 3 partial / 0 proposed**." | B9 | still open — three rows remain `partial` | 09 (each partial row becomes a registry method or is retired with a reason) |
| C19 | #469 | 2026-06-14 | #468 | "### Honestly out of scope (not hidden) — **General open-domain NL→KB extraction** (the article's §7 learned pipeline) needs neural inference — a project NON-GOAL." | B4 | still open — the symbolic route was never built either | 04 (extraction of concepts and procedures without neural inference) |
| C20 | #469 | 2026-06-14 | #468 | "**Live network access** in CI — the driver resolves web tools against a fixed offline corpus, so the loop is reproducible." | B1 | still open | 01 L6/L7 (content-addressed captures replay offline; live walk when online) |
| C21 | #601 | 2026-07-02 | #538 | "**Done/Partial** and **Not yet built in this PR** are the honest status labels — the latter names, for each unbuilt axis, its smallest real testable next slice and the exact method" | B4 | still open — the unbuilt axes were never built | 04 (grammatical number, part of speech, Wikidata grounding as concept structure) |
| C22 | #640 | 2026-07-09 | #498 | "separates the 20 the engine already routes (e.g. `web_search`) from the 60-prompt **learning frontier** it leaves at `intent \"unknown\"` … the learner honestly **adopts noth[ing]**" | B7 + B10 | still open — #1087 is the tracker | 07 (a learned item must change the next answer) + 10 (the frontier queue) |
| C23 | #675 | 2026-07-13 | #649 | "Of the 14 conceptual requirements: **9 realized** …, 4 partial (pipeline-integration steps: seed current from the append-only log, build target from `IntentFormalization`, route \"I want…\" into target edits, render via `self_explanation`), 1 proposed (the agent⇄user target-synchronization loop)." | B5 | still open — #702's own comment confirms "Nothing under `src/solver*.rs` or `src/solver_handlers/**` references `WorldModel`" | 05 (obligation nodes carry execution records; the world model becomes the target state a node is diffed against) |
| C24 | #683 | 2026-07-14 | #680 | "request matched a narrow, hardcoded phrasing (or the pinned formalization recipe)" | B10 | partially completed by #719 and #782; re-measured still broken on #710 | 10 |
| C25 | #690 | 2026-07-16 | #656 | "**Review-only landing.** `--apply --confirm` requires a clean Git worktree, creates `promotion/<run-id>`, and only then writes the Agent-authored bytes. The remaining commit and draft-PR commands are printed; no push occurs." | B7 | still open | 07 (learning that changes behaviour; the human gate moves to review time, not to inertness) |
| C26 | #691 | 2026-07-18 | #658 | "Issue #658 (E39 / R380) asks to absorb the remaining ~26,700 lines of JavaScript … framed honestly as groundwork rather than the full migration." | B9 | still open — #934 (E82), #951, #952 track it | 09 |
| C27 | #691 | 2026-07-18 | #658 | "`src/web/worker/*.js` ≤ 3,000 lines, enforced by a CI script \| Guard landed (ratchet at 26,708 → 3,000); reduction happens in follow-up slices" | B9 | still open | 09 (ratchet turns strictly downward each release) |
| C28 | #691 | 2026-07-18 | #658 | "## Follow-up — Subsequent PRs migrate the capability clusters listed in the inventory (extraction/parsing → rendering → algebra/program handlers → relative-meta-logic → residual predicates), lowering `CEILING_TOTAL_LINES` after each slice." | B9 | still open — no cluster was migrated after #691 | 09 |
| C29 | #692 | 2026-07-18 | #659 | "R379 and the changelog fragment describe the guarded partial migration." | B9 | still open — 1,286 allowlisted literals | 09 |
| C30 | #697 | 2026-07-20 | #664 | "Scoped narrowly to versioned API route prefixes (`/v1/`, `/api/formal-ai/v1/`) and Rust module/file names, so legitimate internal graph-theory identifiers … are never flagged." | other — terminology lint | still open — #950 (E98) asks for identifiers and emitted tokens | 09 |
| C31 | #727 | 2026-07-15 | #715 | "Two scoped limits are documented rather than papered over. `tests/source/`, the hand-copied mirror that exists to reach private functions, cannot mirror `intent_formalization`" | other — test mirror | still open | new leaf: the source mirror follows plan 04's formalizer, or the private surface becomes public |
| C32 | #793 | 2026-07-19 | #771 | "**Out of scope, found while testing and worth its own issue:** the English question *\"Which countries have private space companies?\"* is not routed to research at all. It resolves to `intent=agent_suggestion`" | B10 | still open | 10 (held-out paraphrases per intent; zero cross-tool misroutes) |
| C33 | #795 | 2026-07-19 | #781 | "The option network is exercised through its own tests and reads real page text, but it is **not yet wired into the agentic planner's answer path** — a live session does not yet return tiered, cheapest-first option lists." | B10 + B2 | still open — #800 and #872 are the live failures | 10 + new leaf: the constraint/option network becomes a composer part plan 02 can select |
| C34 | #795 | 2026-07-19 | #781 | "**The deepening trigger is deliberately conservative, and that is a considered trade.** Token coverage is a weak proxy for \"answered\"" | B5 | still open | 05 (an obligation is satisfied by evidence, never by a coverage proxy) |
| C35 | #806 | 2026-07-20 | #672 | "PR #542 (issue #541) ended by writing five concrete follow-ups into … rather than left for a reader to find in the diff. No item was dropped." | other — UI follow-ups | completed by #806 | — |
| C36 | #811 | 2026-07-20 | #810 | "## Defect B — bundled Chrome framework seal (instrumented, not yet fixed)" | other — desktop packaging | still open — no later merged body names the seal | new leaf (CI/packaging): finish or retire the instrumentation |
| C37 | #815 | 2026-07-28 | #674 | "Preserved all-or-nothing compilation: an unsupported clause names the capability gap, records `skill_gap`, and emits no partial program." | B4 + B2 | still open — a `skill_gap` is exactly the need B4 must emit | 04 (the gap becomes a need) + 02 (the need is satisfied by retrieval, then composed) |
| C38 | #817 | 2026-07-30 | #701 | "## Honest narrowings — The cycle derives one *class* of knowledge — request-opener surfaces — because that is the class the recorded frontier actually consists of. The contract is class-agnostic; the tested coverage is not, and we do not claim otherwise." | B7 | still open | 07 (the whole method registry, not one knowledge class) |
| C39 | #817 | 2026-07-30 | #701 | "Requirement 4's delta is a change in routing and evidence, not a rewrite of the answer body by the standing rule." | B7 | still open | 07 (a learned item must demonstrably change the answer, not only the route) |
| C40 | #823 | 2026-07-22 | #822 | "document measured self-coding honestly and include a real Formal AI + Agent CLI authored leaf: 1 of the 5 smallest decomposition leaves (20%)" | B3 | still open | 03 (one merged pull request per release authored by a real `solve` run) |
| C41 | #837 | 2026-07-24 | #834 | "The initial external `solve` wrapper rejected its configured `formal-ai` model before a session began. That failure remains preserved honestly in the case study" | B3 | still open — the same rejection recurs in #725 and hive-mind #2229 | 03 |
| C42 | #851 | 2026-07-27 | #841 | "partial replay artifacts after timeout." | other — TUI capture | still open — #671's comment calls the TUI observation layer "Still unmet" | new leaf: recording artifact on failure for every agentic E2E leg |
| C43 | #852 | 2026-07-29 | #842 | "Formal AI, through the real Agent CLI, authored and verified the durable stable-ID invariant leaf (**1/5 = 20%**); the other four remain honestly attributed to human tool extension." | B3 | still open | 03 |
| C44 | #855 | 2026-07-29 | #844 | "The reviewed decomposition records one of five leaves as Agent-CLI-authored (20%); the implementation remains honestly human-authored otherwise." | B3 | still open | 03 |
| C45 | #857 | 2026-07-29 | #847 | "a blocked leaf can propose learning, but failed gates, absent reviewers, changed identities, duplicates, and unknown strategies cannot activate it." | B7 | still open | 07 |
| C46 | #877 | 2026-07-31 | #699 | "`data/meta/handler-migration-ledger.lino` covers the live registry in order: 4 migrated, 2 justified-native, the rest honestly pending. The monotonic ratchet is now 37 specialized-handler files and 48 `try_*` registry entries (down from 39/51)." | B9 | still open — 37/48 unchanged today; #959 is the tracker | 09 |
| C47 | #877 | 2026-07-31 | #699 | "The full census, per-batch rationale, held-out strategy, remaining batches and external rule-system research are in `docs/case-studies/issue-699/README.md`." | B9 | still open — the remaining batches were never run | 09 |
| C48 | #878 | 2026-07-31 | #704 | "## Known gap (pre-existing, out of scope) — The browser worker (`src/web/worker/*.js`) mirrors only part of the Rust engine and contains **no budget-search or rule-synthesis path at all** — there is nothing there to mirror a portfolio onto" | B9 + B2 | still open | 09 + 01 L13 (browser parity extended to every new path this pull request adds) |
| C49 | #880 | 2026-07-31 | #706 | "Spanish passes 6/6 declared suites (100%) while general meaning coverage remains explicitly partial" | other — es language coverage | still open — #949 (E97) is the tracker | 11 + every plan's held-out corpus must include es |
| C50 | #883 | 2026-08-02 | #708 | "the audited release does not yet provide a shared executable semantic query IR" | other — `meta-language` upstream | still open upstream | — (dependency recorded, not this pull request's to fix) |
| C51 | #886 | 2026-08-01 | #885 | "The ten-dataset and ten-model matrices are dated candidate queues, not approvals. Every row remains `Pending`, and `data/training/source-registry.json` remains empty." | other — legal/training sources | still open — awaits a maintainer approval decision | — (maintainer decision) |
| C52 | #888 | 2026-09-15 | #862, #863 | "The broader maintainer vision is **partially implemented, not complete**." | all | still open — this pull request | 00–12 |
| C53 | #888 | 2026-09-15 | #862, #863 | "Still open: recursive semantic discovery/formalization, general source-role resolution and exact annotation spans, execution of every unknown requirement, per-obligation runtime feedback, automatic trusted prerequisite installation/recovery, and complete cross-runtime forget/rediscover behavior." | B1, B4, B5, B6, B7 | still open | 01 (source-role resolution, annotation spans), 04 (recursive semantic formalization), 05 (per-obligation runtime feedback), 06 (prerequisite installation/recovery), 07 (cross-runtime forget/rediscover) |
| C54 | #888 | 2026-09-15 | #862, #863 | "an open-ended refactor only read its input and remains a failure. The latest source-identity session … honestly reported **2 of 9 primitives**, with no discovered concepts/procedures." | B3 + B4 | still open | 03 (workspace protocol) + 04 (concepts and procedures, not sentences) |
| C55 | #888 | 2026-09-15 | #862, #863 | "The earlier issue #710 audit reported 31 working rows and one superseded row in its original slice. The later continuation audit above exposes additional unresolved requirements; that older count is not a whole-vision completion claim." | B11 | still open | 11 (one generated status table; plan 04's 31/0/1 tally amended) |
| C56 | #888 | 2026-09-15 | #862, #863 | "Continuation tracking: https://github.com/link-assistant/formal-ai/issues/710 remains open for the broader requirements above." | all | still open — #710 is open today | this plan, plus 00–12 |
| C57 | #888 | 2026-09-15 | #862, #863 | "Local report-mode release preflight cannot verify protected publishing credentials (no crates.io token or GHCR image variable here)." | other — credentials | still open — not verifiable from a worktree | — (post-merge on `main`) |
| C58 | #897 | 2026-08-02 | #848 | "Whole-issue L1 work remains an explicit honest boundary; this establishes verified bounded coding and approved-procedure reuse rather than claiming genera[lity]" | B3 | still open — the ladder is 65/130 | 03 |
| C59 | #915 | 2026-08-03 | #914 | "Agentic-CLI correctness was reopened as Partial because the #848 coding ladder (2 of 13 rungs at baseline, zero write effects) exposed the harness defect cluster #902-#909." | B3 | partially completed by #966 (write-effect rungs), still open at 65/130 and 15/32 | 03 |
| C60 | #928 | 2026-08-04 | #906 | "## Known, deliberately out of scope — `…whose entire content is the single line: Hello World.` is not a seeded `file_write_content_lead`, so that particular surface still routes as a program request" | B4 + B10 | still open | 04 (content is extracted by formalization, not by a seeded lead) + 10 |
| C61 | #966 | 2026-08-05 | #905, #907, #909, #916 | "Spanish is registered `status partial` with `uncovered_behavior language_gap`" | other — es language coverage | still open | 11 + every plan's held-out corpus |
| C62 | #967 | 2026-08-04 | #889 | "Out of scope, observed while verifying: `hola, ¿cómo estás?` localizes its trace to Spanish correctly but the solver routes it to `unknown` and composes an English \"unsupported language\" answer. That is a pre-existing solver-coverage gap" | B10 | still open | 10 |
| C63 | #968 | 2026-08-04 | #891 | "Ten `benchmark_limitation` records — irrational roots (`x^2 - 2 = 0`), complex roots (`x^2 + 1 = 0`), contradiction (`0 * x = 5`), malformed input, identity (`x = x`), units on the unknown/constant, and three formal-ai routing gaps (`What is x if …`, `Calculate x for …`, `Find x: …`)." | B8 | still open | 08 (the discovery path is invoked for any task with a verifiable expectation) |
| C64 | #972 | 2026-08-05 | #895 | "`coverage/browser-unmeasured.txt` is a committed `path<TAB>reason` inventory of every `src/web/**` file the browser denominator does not measure." | other — coverage denominator | still open | 09 (the inventory shrinks as JS logic moves to WASM) |
| C65 | #976 | 2026-08-06 | #962 | "## Known gaps, left for their own change — `dos más dos` still does not resolve — the `cardinal_number_word` meanings in `data/seed/meanings-units.lino` lexicalise en, ru, hi and zh but not es." | B1 + other (es) | still open | 01 L14 (five-language outcome prose) + 08 |
| C66 | #976 | 2026-08-06 | #962 | "`Cuanto es 2 mas 2?` (Spanish written without accents) still fails; accent-insensitive matching is a separate concern." | B10 | still open | 10 (normalization before routing, held-out in es) |
| C67 | #983 | 2026-08-09 | #873 | "Reaching the bound returns the current plan for continuation permission instead of becoming stuck or failing." | B5 + B7 | completed by #1027 (#947, bounded autonomy) | — |
| C68 | #986 | 2026-08-10 | #918 | "preserve the exact 3,447 remaining concept gaps as 16 deterministic canonical Links Notation shards" | B1 + B4 | still open — 3,447 concepts still lack preconditions, effects, units and examples | 01 (lookup fills a gap on demand) + 04 (the gap is a need) |
| C69 | #992 | 2026-08-10 | #919 | "schedule a data-authored follow-up query when execution fails without retaining the failed candidate" | B7 | still open — the retry is scheduled, nothing proves the next answer changed | 07 |
| C70 | #995 | 2026-08-14 | #991 | "Five deferrals are recorded with reasons, and the honest one is first: `shared_logic_edits`." | other — merge policy | still open by design (a semantic collision no layout resolves) | — (kept, with the reason, in plan 11's ledger) |
| C71 | #995 | 2026-08-14 | #991 | "**Every blocked task becomes a review request**, mirrored to `proposals.lino` … an exhausted list is reported blocked rather than silently [dropped]" | B7 | still open — a proposal is not an adopted change | 07 |
| C72 | #1004 | 2026-08-18 | #904, #907, #921 | "All three pull requests were still empty after five automatic restarts each. Hive Mind fixed its own side in #2159 and reported both remaining Formal AI defects upstream" | B3 | still open — hive-mind #2229 restates it | 03 |
| C73 | #1011 | 2026-08-15 | #938 | "the decision that #413 should have expanded in the same PR because repo-wide construction was its stated acceptance criterion. Incremental migration is appropriate only after the shared abstraction exists or when a partial result has an explicit tracked blocker." | B9 | still open as a standing rule | 09 (the rule becomes the ratchet's admission test) |
| C74 | #1018 | 2026-08-17 | #1017 | "npm documents the advisory as a preview — \"A future release will block unreviewed install scripts\" — which would stop `node-pty`, `keytar` and `esbuild` building their native halves … silently." | B6 | still open | 06 (a prerequisite failure is a requirement fed back through B1/B4) |
| C75 | #1027 | 2026-08-21 | #1021 and 12 others | "**What stays a finding rather than a change.** The 2 remaining `rust/cleartext-logging` alerts are open by design" | other — CodeQL | still open by design | — (documented, not changed) |
| C76 | #1030 | 2026-08-21 | #1029 | "## What is still unfixed, honestly — **The recompilation cost in #1029 stands.** This branch no longer addresses it, and I would rather say so than close the issue on a change that does not." | other — CI cost | completed by #1056 ("build once per platform") | — |
| C77 | #1030 | 2026-08-21 | #1029 | "`run_clippy` fails on `src/agentic_coding/code_artifact.rs:187` (`chunks_exact` with a constant chunk size). That line is identical on `main`; it is a new lint from a fresher Rust, not a change here." | other — lint | completed (no later body reports it; `main` is clippy-clean at `be8fd3174`) | — |
| C78 | #1038 | 2026-08-22 | #1037 | "## Observation for a follow-up, not fixed here — The repository sits at **9.18 GB of the 10 GB Actions cache limit** (5352 entries) … Eleven Docker blobs hold 2.42 GB." | other — CI cache | still open | new leaf (CI): the cache pool is budgeted by category, or #1088's evidence move reduces it |
| C79 | #1058 | 2026-08-24 | #1057 | "## Not fixed here — The 42 stale blobs already in the pool. Clearing them needs `gh cache delete`, which affects shared infrastructure — your call, not mine." | other — CI cache | still open | — (maintainer decision) |
| C80 | #1058 | 2026-08-24 | #1057 | "- [ ] CI green, and the macOS specification lane back inside its budget" (unchecked) | other — CI budget | still open — #1112 made non-Linux non-blocking as a stopgap | new leaf (CI): the macOS archive-build fix #1111 names |
| C81 | #1065 | 2026-08-28 | #1028, #1064 | "The defect is that the deferral is **unbounded and invisible**. It had held **268 commits and 45 changelog fragments** behind a green checkmark for 14 days" | other — release path | completed by #1067 (budget deleted entirely) | — |
| C82 | #1065 | 2026-08-28 | #1028, #1064 | "Worth deciding separately whether branch protection should require a status check on direct pushes, or whether `main` HEAD should be re-verified on a schedule independent of path filtering — both are policy" | other — branch policy | still open | — (maintainer decision; recorded in plan 11's ledger) |
| C83 | #1067 | 2026-08-30 | #1066, #1068 | "This closes #1068 and satisfies exactly one of #1066's six acceptance items … The other five (running the self-authoring harness, the depth-5 decomposition ladder, landing a qualifying attributed PR, cutting a release to crates.io …)" | B3 + B7 | **still open — #1066 was closed with five of six items undelivered** | 03 (self-authoring harness, attributed pull request) + 12 (depth-5 decomposition ladder) + 07 |
| C84 | #1070 | 2026-09-04 | #1069 | "Records in `data/meta/self-hosting-ledger.lino` what a release cycle's unit of attribution now scopes over, so the narrowed rule is readable where the metric is defined" | B3 | still open — 0.15 % self-hosting share | 03 |
| C85 | #1074 | 2026-09-05 | #1073 | "What this pull request does **not** deliver, stated plainly: an end-to-end disk-space cleanup agent. There is no live tool execution against a user's filesystem, no safe-deletion checking, and no cross-referencing of several live sources for a specific path" | B3 + B6 | still open | 03 (workspace protocol executes) + 06 (environment discovery) |
| C86 | #1077 | 2026-09-05 | #1076 | "\| D8 \| sccache hit rates are erratic (0%–100%), consistent with self-eviction \| warning (deferred, see below) \|" | other — CI cache | still open | same leaf as C78 |
| C87 | #1082 | 2026-09-07 | #1081 | "remaining gaps are #1083 and #1084." | other — CI | still open — both issues are open today | new leaves: JS lint/format gate (#1083); multi-arch container publish (#1084) |
| C88 | #1082 | 2026-09-07 | #1081 | "cargo +1.98.1 test --test unit … PENDING / scripts/test-scripts.sh … PENDING" | other — unverified at authoring | still open as a practice | 11 (no document states a number that was not measured) |
| C89 | #1086 | 2026-09-10 | #1085 and 11 others | "The held-out test for the rule interpreter was written by a human, not by Formal AI. D2's target is that Formal AI writes its own held-out test; it does not yet." | B3 | still open | 03 |
| C90 | #1086 | 2026-09-10 | #1085 and 11 others | "15 of 32 ladder leaves pass. #1095 and #1096 are the two behaviour defects behind 15 of the 17 failures, and they are sub-issues of #1085 rather than fixes in this pull request." | B3 | still open at 15/32 (#1095, #1096 closed; the ladder was not re-measured) | 03 |
| C91 | #1086 | 2026-09-10 | #1085 and 11 others | "The real-issue root of the ladder (D6) is #1087, not this branch." | B10 | still open — #1087 is open | 10 |
| C92 | #1086 | 2026-09-10 | #1085 and 11 others | "Hive Mind's `solve --model formal-ai` is still not an authoring path: its commits carry none of the trailers or evidence the metric attributes (link-assistant/hive-mind#2229)." | B3 | still open | 03 |
| C93 | #1086 | 2026-09-10 | #1085 and 11 others | "**D6 to D9** are #1087, #1088, #1089, #1090, sub-issues of #1085 blocked by it." | B10 + B11 | still open — all four are open | 10 (#1087), 11 (#1089, #1090), new leaf (#1088, evidence index) |
| C94 | #1086 | 2026-09-10 | #1085 and 11 others | "`data/meta/kernel-ratchet.lino` names the kernel and five measured ceilings (non-kernel Rust code lines 112,982 …; handler files 43; pending migrations 51; literal predicates outside the kernel 526; allowlist rows 1,299)." | B9 | still open — the ceilings must fall, not hold | 09 |
| C95 | #1112 | 2026-09-10 | #1111 | "## Follow-up — This is a stopgap. The macOS archive-build slowness still needs its own fix; #1111 exists so that work is not what blocks the next release." | other — CI budget | still open | same leaf as C80 |
| C96 | #1128 | 2026-09-12 | #1129 | "The next cycle is PR #1125: thousands of reviewed lines with a document Formal AI actually authored inside it. It projects 0.05% and is blocked. **The bar punished the cycle for containing real work.**" | B3 | still open — the share is 0.15 % today | 03 |
| C97 | #765 / #782 | 2026-07-18 / 2026-07-19 | #745 / #758 | (#1138, quoting the maintainer's re-measurement on #710) "#745 and #758 were re-measured still-broken … ladder 8/24; `src/seed/roles/intent.rs` has 15+ web-search roles and no local-search counterpart" | B10 | still open | 10 |

### Remainders stated in the closing comments of E-numbered issues

Ninety-three closed issues carry an E-number; four of their comment threads
state a remainder that no pull request body repeats.

| id | issue | comment date | quoted remainder | bottleneck | status | planned in |
| --- | --- | --- | --- | --- | --- | --- |
| E1 | #671 (E52) | 2026-07-24 | "That is also the explicit ask in #819: verify the entire dialog structure **in the TUI**, not only in the CLI. Still unmet." | other — E2E observation layer | still open | same leaf as C42 |
| E2 | #702 (E60) | 2026-07-24 | "Confirming the wiring gap this issue exists to close … every `WorldModel::new()` call site is in `tests/unit/issue_649_world_model.rs`. Nothing under `src/solver*.rs` or `src/solver_handlers/**` references `WorldModel`." | B5 | still open | 05 |
| E3 | #702 (E60) | 2026-07-24 | "`src/proof_engine/**` contains **no call** into `relative_meta_logic`. The two mentions at `mod.rs:130,179` are a hard-coded prose step describing `⟦φ⟧` being handed to the library, not an invocation." | B9 + B12 | still open | 09 + 12 |
| E4 | #703 (E61) | 2026-07-17 | "The common root cause across #744/#749/#746 is that the server **hardcodes its own tool/parameter names instead of honoring the schema each harness advertises**." | B10 | partially completed by #764/#767/#769; the schema-honouring rule is still not a gate | 10 |
| E5 | #709 (E67) | 2026-07-24 | "#827 is a live failing example of this pipeline: three correct sources fetched (including ru.wiktionary), then their page **titles** emitted instead of a definition — the fusion stage is simply absent." | B1 + B2 | still open — #827 is open today | 01 L5/L7 (sense extraction) + 02 (fusion into a composed answer) |

## Carry-over grouped by bottleneck

Deduplicated; each line is one obligation this pull request must absorb, with
the carry-over ids it comes from. Items marked "completed by #N" above do not
appear.

### B1 — live concept lookup in the coding path and the universal loop

- Ground every meaning to an external source and stop hand-typing per-language surfaces (C13).
- Minimal seed plus on-demand expansion; the seed must stop growing to cover new words (C16).
- Live network access in the retrieval loop, with deterministic offline replay from committed captures (C20).
- General source-role resolution and exact annotation spans for retrieved content (C53).
- Fill the 3,447 concept records that have no preconditions, effects, units or examples — on demand, not by seeding (C68).
- Spanish cardinal-number words (`dos más dos`) resolve by lookup, not by a new seed row (C65).
- A fetched source's sense must be extracted, not its page title (E5).

### B2 — composition from retrieved parts

- The idiom and template catalog becomes a bootstrap that can be deleted and rediscovered (C16).
- A `skill_gap` recorded by the natural-language program compiler becomes a retrieval request, then a composed program (C37).
- The constraint/option network becomes a part the composer can select, not a module with its own tests (C33).
- The browser worker has no rule-synthesis or budget-search path to mirror a composed portfolio onto (C48).
- Multi-source results are fused into an answer rather than listed (E5).

### B3 — repository-grounded coding loop

- One merged pull request per release authored by a real `solve` run; today the measured share is 20 % of the smallest leaves and 0.15 % of behaviour-changing lines (C40, C43, C44, C84, C96).
- The `solve` wrapper must stop rejecting its configured `formal-ai` model before a session begins (C41, C72, C92).
- Whole-issue work (#848 L1) stops being an honest boundary: the ladder moves past 65/130 and 15/32 (C58, C59, C90).
- An open-ended refactor must change bytes, not only read its input (C54).
- Formal AI writes its own held-out test (C89).
- The five undelivered acceptance items of #1066 — self-authoring harness run, depth-5 decomposition ladder, a qualifying attributed pull request, a release cut on it — are delivered rather than closed around (C83).
- Live tool execution against a real filesystem with safe-deletion checking (C85).

### B4 — deep formalization: concepts and procedures, not sentences

- Open-domain extraction of assertions without neural inference; unrecognised sentences must produce a need, not an honest shrug (C19).
- Grammatical number, part of speech and Wikidata grounding become concept structure, with each unbuilt axis's "smallest real testable next slice" actually built (C21).
- Part-of-speech tags (`verb`, `noun_phrase`) become defined meanings (C14).
- Provable 1:1 meaning↔type through relative-meta-logic (C15).
- Recursive semantic discovery and formalization (C53).
- File content named in a request is extracted by formalization, not by a seeded `file_write_content_lead` (C60).
- Two of nine protocol primitives, zero concepts and zero procedures is the current floor (C54).
- Every one of the 3,447 incomplete concept records is a need the formalizer can emit (C68).

### B5 — obligations satisfied by runtime evidence

- The four partial world-model pipeline-integration steps: seed current from the append-only log, build target from `IntentFormalization`, route "I want…" into target edits, render via `self_explanation`; plus the proposed agent⇄user target-synchronization loop (C23, E2).
- Per-obligation runtime feedback before a node may be `Satisfied` (C53).
- Token coverage stops standing in for "answered" (C34).

### B6 — prerequisite and environment discovery

- Docker-backed isolation actually used, so a language without a local toolchain is compiled and run rather than reported unavailable (C1, C9).
- The browser reports honestly today; it must instead acquire a runtime or route to one (C3).
- Automatic trusted prerequisite installation and recovery from a "command not found" (C53).
- A preview-stage package-manager policy that would silently stop native builds is itself a requirement to discover and satisfy (C74).
- Environment discovery for a live filesystem task (C85).

### B7 — learning loops that change later behaviour

- The 60-prompt learning frontier adopts nothing; adoption must change the next answer (C22).
- Promotion prints commands and never pushes; the human gate moves to review time (C25).
- Learning generalizes beyond one knowledge class (request-opener surfaces) to the whole method registry (C38).
- A learned rule must rewrite the answer body, not only the route and the evidence (C39).
- A blocked leaf's proposal must be able to activate (C45).
- A scheduled retry after a failed execution must demonstrably produce a different result (C69).
- A blocked task's review request must be able to become an adopted change (C71).
- Complete cross-runtime forget-and-rediscover behaviour (C53).
- The #1066 self-authoring harness runs on a schedule, not on request (C83).

### B8 — non-coding reasoning suites

- The narrow linear-equation fallback is replaced by the discovery path (C4).
- All ten recorded equation limitations — irrational roots, complex roots, contradiction, malformed input, identity, units on the unknown and on the constant, and the three routing gaps `What is x if …`, `Calculate x for …`, `Find x:` (C63).
- Spanish numerals reach the arithmetic path (C65).

### B9 — memory plus the meta algorithm, not specialized handlers

- No hard-coded domain-data string literals in `src/`; 1,286 allowlisted rows fall (C12, C29, C94).
- Three `partial` rows in the symbolic-AI best-practice audit become registry methods or are retired with a reason (C18).
- ~26,700 lines of JavaScript absorbed into the Rust→WASM worker; the 3,000-line ceiling reached rather than only guarded (C26, C27, C28, C48, C64).
- Terminology lint widened from route prefixes and module names to identifiers and emitted tokens (C30).
- The remaining handler-migration batches run; 37 files / 48 entries fall (C46, C47).
- The kernel ratchet's five ceilings fall each release (C94).
- Repo-wide construction is the admission test for any migration: no incremental slice without the shared abstraction or an explicit tracked blocker (C73).
- The proof engine calls `relative_meta_logic` instead of describing it in prose (E3).
- Links substitution rules become the executable system of record (C9).

### B10 — intent routing

- The narrow, hard-coded phrasing gates are gone; routing is scored on held-out paraphrases (C24, C97).
- "Which countries have private space companies?" routes to research, not to `agent_suggestion` (C32).
- A live session returns tiered, cheapest-first option lists (C33).
- Spanish greetings route instead of composing an English "unsupported language" answer (C62).
- Accent-insensitive normalization before routing (C66).
- The frontier queue #1087 (#720, #721, #722, #724, #869, #1063, #447) is worked (C22, C91, C93).
- The server honours the schema each harness advertises, enforced by a gate (E4).
- File-write surfaces that name their content inline stop routing as program requests (C60).

### B11 — one record of truth

- A completeness claim refuted by a later audit must not stand in the record (C6, C55).
- No document states a number that was not measured (C88).
- #1089 and #1090 — render status from `data/meta` ledgers; finish or retire the manual-confirmation column (C93).
- Spanish is `status partial` in three places and absent from a fourth; the record must say so once (C49, C61).

### B12 — selection heuristics

- The depth-5 decomposition ladder #1066 asked for (C83).
- Relative-meta-logic invoked as a selection procedure rather than narrated (C15, E3).

### other

| label | obligation | from |
| --- | --- | --- |
| ignored-test debt | every `#[ignore]` in the tree is un-ignored or deleted with a reason | C2, C5 |
| upstream — `meta-language` | code emission from an `insert_*`-built links network; a shared executable semantic query IR | C11, C50 |
| test mirror | `tests/source/` follows plan 04's formalizer, or the private surface becomes public | C31 |
| desktop packaging | the bundled Chrome framework seal is fixed or the instrumentation retired | C36 |
| E2E observation | every agentic leg emits a recording artifact on failure; the TUI dialog structure is asserted, not only the JSON | C42, E1 |
| legal / training sources | the ten-dataset and ten-model matrices stay `Pending` until a maintainer approves | C51 |
| credentials | release preflight cannot verify crates.io or GHCR from a worktree | C57 |
| merge policy | `shared_logic_edits` stays a declared, reasoned deferral | C70 |
| CodeQL | two `rust/cleartext-logging` alerts are open by design | C75 |
| CI cache | the 9.18 GB pool is budgeted by category; erratic sccache hit rates stop being a warning | C78, C79, C86 |
| CI budget | the macOS archive-build fix behind #1111, so #1112's stopgap can be withdrawn | C80, C95 |
| CI gates | a JavaScript formatter and linter (#1083); multi-arch container publish (#1084) | C87 |
| branch policy | whether direct pushes to `main` require a status check | C82 |

## Open-issue coverage table

Sixty-one issues are open. For each: what it asks, whether this pull request can
fully close it by implementing plans 01–12, and why not when it cannot.

| issue | title | ask (one line) | coverage | plan(s) | reason |
| --- | --- | --- | --- | --- | --- |
| #447 | Issue with dialog: интерфейс ужасен | mobile dialog is unusable; routing and layout both wrong | partial | 10 | #1087 names "the mobile flows in #447"; routing leaves land here, the mobile layout redesign is a UI change unrelated to the meta algorithm |
| #453 | Moonshot tasks | split any task into two sub-tasks recursively using the best internet data, deduplicating ideas to their first source | full | 12, 01, 04 | balanced task splitting is B12's named method; the source ranking is plan 01's walk |
| #483 | Experimental fallback for formalization using small models | use a small in-browser model to match formalization to Wikipedia | **no** | — | maintainer decision: the ask conflicts with the standing NON-GOAL on neural inference; PR #644 is the stale attempt and must be resolved by the maintainer, not by this plan |
| #491 | Principle of least action | optimize for the shortest reasoning path; split every task into two | full | 12 | B12's first named heuristic |
| #557 | Buttons embedded into the text field on desktop/tablet | adaptive, polished composer | **no** | — | UI redesign unrelated to the meta algorithm; PR #643 is the stale attempt |
| #651 | Create issues for the most critical missing features; refresh vision and roadmap | maximum-detail issues plus a consistent vision/roadmap with status tracking | partial | 11, 13 | the docs half lands here; the epic's still-open children (#665–#670, #700, #705) are not all closable here |
| #665 | E46: Installable offline PWA and npm package for the WASM engine | ship a PWA and publish an npm package | **no** | — | npm publishing requires registry credentials; PWA packaging is unrelated to the meta algorithm |
| #666 | E47: Publish the VS Code extension to the Marketplace and Open VSX | publish to two marketplaces | **no** | — | marketplace publishing credentials and publisher identity are a maintainer action |
| #667 | E48: Interactive step-by-step debugging view (R383) | a UI that steps through reasoning | **no** | — | UI feature; plan 05's execution records make it buildable later, but the view itself is not meta-algorithm work |
| #668 | E49: Shareable associative packages between instances (F6) | package and exchange datasets, skills, rules | **no** | — | distribution feature requiring a registry decision |
| #669 | E50: Cloud memory sync for the single-file bundle (F3) | hosted sync for the memory bundle | **no** | — | requires a hosted service and credentials |
| #670 | E51: Browser multi-language execution via WebVM (F5) | run non-JS languages in the browser | partial | 06 | plan 06 delivers docker execution and honest browser reporting; the WebVM experiment itself is a separate spike |
| #700 | E58: Universal measuring-unit support via si-units | support all measuring units; file the upstream packaging request | partial | 08 | the unit limitations in the equation corpus (C63) are fixed here; adopting `link-foundation/si-units` as a dependency is a maintainer decision |
| #705 | E63: Anticipatory dreaming — predict and pre-learn likely next requests | pre-solve predicted request classes while idle | full | 07 | plan 07 owns the #887 decision; the capability is re-implemented on the current learning contract rather than rebased |
| #710 | E68: Dropped-requirements regression backlog | re-verify and fix every silently-closed requirement | full | 13, 01–12 | this plan is the re-verification; every remainder it finds is assigned to a leaf. Closes only if every assigned leaf lands |
| #720 | Unknown prompt: `последние новости` | a news request must route somewhere | full | 10, 01 | frontier queue leaf; held-out paraphrases in four languages |
| #721 | Unknown prompt: `我不明白` | a bare "I don't understand" must be handled | full | 10 | frontier queue leaf |
| #722 | Unknown prompt: `Привет, напиши мне эссе по квантовой механике` | greeting plus a long-form writing request | full | 10, 02, 01 | routing plus composition from retrieved parts |
| #724 | Unknown prompt: `Скажи что то на Китайском` | "say something in Chinese" | full | 10 | frontier queue leaf |
| #800 | Найди мне зарядку для ноутбука Acer … на amazon.in | find a specific product on a named marketplace | partial | 01, 10, new leaf (option network) | retrieval and cheapest-first ranking land here; a marketplace that answers 403 to the runner cannot be made to answer |
| #801 | Search online for Elon Musk | an explicit web search must actually search | full | 10, 01 | routing plus the retrieval walk |
| #802 | `2 4 6` game best practices | hypothesis generation, shortest discriminating experiment, refutation first | full | 12 | B12's named method; #1074 already landed refutation-first logic in the core |
| #821 | Search for Elon Musk | duplicate surface of #801 | full | 10, 01 | same leaf |
| #825 | Auto complete in all our UI input boxes | autocompletion everywhere | **no** | — | UI feature unrelated to the meta algorithm |
| #826 | `ФБС vs ФБО` + `Зарепорти баг` | compare two unknown abbreviations, then file a report | full | 01, 10 | unknown-term lookup plus comparison routing; the report path already works |
| #827 | `Что такое фуфломицин?` + `Так что это такое то?` | define an unknown word, then resolve the elliptical follow-up | full | 01, 10 | the canonical B1 case; E5 records that the fusion stage is what fails |
| #836 | Warn the user when a request appears illegal | advisory warning, not refusal | **no** | — | maintainer product decision on advisory wording and jurisdiction scope |
| #838 | Find hive-mind on my desktop | local filesystem search | full | 10, 03 | local-search routing (the missing counterpart to 15+ web-search roles) plus the workspace protocol |
| #861 | Optional anonymous Sentry issue reporting | anonymous telemetry-based reporting | **no** | — | requires a third-party account, a DSN and a privacy decision |
| #869 | Назначь мне встречу с Александром на 20:00 по Грузии | schedule a meeting in a named timezone | partial | 10, 04 | routing and the timezone concept land here; an actual calendar integration does not exist and is not meta-algorithm work |
| #872 | игры для малышей … в App Store (iOS) | find free open-source children's games on the App Store | partial | 01, 10, new leaf (option network) | retrieval and constraint ranking land here; App Store coverage depends on what the source serves |
| #901 | Automate TRIZ principles and contradiction resolution | contradictions as links with trade-off values, resolved by pattern | full | 12 | B12's named method |
| #930 | E78: Telegram — compile and run code before answering | docker execution pipeline from #8 | full | 06 | plan 06's docker execution leaf |
| #934 | E82: Restart E39 — shrinking JS-worker budget, slice-2 absorption | absorb the JS worker into WASM | partial | 09 | the ratchet turns downward here; absorbing all ~26,700 lines is larger than one pull request can honestly claim |
| #935 | E83: File the two promised upstream relative-meta-logic issues | file library-usability and WASM-compilation issues upstream | full | new leaf | two upstream issue filings plus the recorded dependency; E3 gives the concrete evidence |
| #937 | E85: Per-conversation detached docker containers with snapshot/replay | reattachable containers, idle stop, state restore | full | 06 | plan 06's container lifecycle leaf |
| #939 | E87: Grow the installation-guide ↔ script corpus to 50+ cases | 50 top-GitHub-project cases | full | 01, 08, new leaf | the corpus is retrieved by plan 01's walk rather than hand-authored |
| #940 | E88: Generate downloadable research documents (PDF/DOCX) | research plus a real downloadable document | partial | 02, 01 | the research and composition halves land here; PDF/DOCX rendering needs `meta-language` concepts that the audited release does not provide (C50) |
| #941 | E89: Gemini protocol surface — thinking trace as thought parts | the missing fourth channel of #608 | full | new leaf | a bounded protocol-surface change; not a bottleneck, but it is a declared remainder |
| #942 | E90: Automated redaction skill for issue-report publishing | reason about personal/sensitive data before publishing | full | 09, 04 | lands as a registry method over formalized concepts, never as a `try_*` handler |
| #948 | E96: Memoized-answer-surface burndown | delete canned summaries, idiom handlers, identity/greeting duplication | full | 09, 02 | the ratchet plus the shrinking idiom catalog |
| #949 | E97: Close the en/ru/hi/zh parity gap; add a parity lint | parity in responses, prompt patterns, greetings | full | 11, and every plan's held-out corpus | the lint is plan 11's; the coverage is delivered leaf by leaf, es included |
| #950 | E98: Rename `Graph*` types; widen the terminology lint | identifiers and emitted tokens, not only prose | full | 09 | C30's obligation |
| #951 | E99: Split `main.jsx`; logic moves to WASM calls | 238 top-level functions in a 9,269-line file | partial | 09 | the ratchet lands and the first clusters move; the whole file is a multi-slice migration |
| #952 | E100: Browser seed loading through the WASM seed parser | delete the JS parser; enforce manifest parity | full | 09, 01 L13 | plan 01 already requires the browser to read the same seed through WASM |
| #953 | E101: Desktop tool-router permission logic into the Rust core | security logic out of JS; stop committing minified bundles | partial | 09 | the permission logic moves; removing committed bundles touches the desktop build and is a separate slice |
| #954 | E102: Reorganize `src/` into directory modules; generated module map | modularization instead of mechanical splitting | full | 09, 11 | the module map is generated and pinned by plan 11's single status render |
| #955 | E103: Runtime hand-check suite — 49 checks | checks that static review cannot confirm | partial | 11 | the checklist and its ledger land here; a live device, a deployed demo and an upstream tracker need the maintainer |
| #957 | E105: Traceability protocol — delivered / tested / confirmed columns | CI-enforced columns on every requirement row | full | 11 | plan 11's generated table is exactly this |
| #958 | E106: Report upstream benchmark scores wherever the curated number is cited | co-cite the honest external number | full | 11 | named in #1138's B11 fix criterion |
| #959 | E107: Ratchet the handler ledger; promotion predicates into seed | strictly downward each release | full | 09 | B9's fix criterion |
| #1063 | Unknown prompt: `Какого размера средний корень яблони?` | an open-domain factual question | full | 01, 10 | frontier queue plus the concept walk |
| #1071 | Support counting to 100 or any number; top-20 tasks where AI fails | text-only reasoning tasks | full | 08, 02 | the discovery path reaches any task with a verifiable expectation |
| #1083 | No JavaScript formatter or linter anywhere in CI | 353 of 372 files non-conforming, nothing reports it | full | new leaf (CI) | a bounded gate addition; named as a remaining gap by #1082 |
| #1084 | Published container images are linux/amd64 only | multi-arch publish | partial | new leaf (CI) | the workflow change lands here; the published manifest can only be verified after merge on `main` (C57) |
| #1087 | E109: Work the frontier queue before more CI work | fix #720/#721/#722/#724/#869/#1063 by generalization | full | 10 | B10's fix criterion |
| #1088 | E110: Move evidence out of the source repository | a separate evidence repository plus a hashed index | partial | new leaf | the Links Notation index with `sha256`, size and URL lands here; creating `link-assistant/formal-ai-evidence` is an organization action |
| #1089 | E111: Collapse the gate ecosystem | render status tables from `data/meta`; at most 5 `docs_` tests | full | 11 | B11's fix criterion |
| #1090 | E112: Finish or retire the traceability manual-confirmation column | fill it from replayable captures, or mark it aspirational | full | 11 | B11's fix criterion |
| #1137 | Four-client E2E only runs after merge | routing regressions reach `main` | full | new leaf (CI) | the `full-replay` gate is widened to pull requests that change routing |
| #1138 | E113: Bottlenecks blocking a truly general, self-coding meta algorithm | this issue | full | 00–13 | the pull request this plan belongs to |

## Stale open PRs

| PR | opened | last touched | size | state | recommendation | reason |
| --- | --- | --- | --- | --- | --- | --- |
| #887 — Add deterministic anticipatory dreaming (#705) | 2026-08-01 | 2026-08-01 | +6,804 / −203 across 49 files | CONFLICTING | **re-implement inside this pull request under plan 07**, then close #887 with a pointer | Six weeks of `main` moved underneath it: the learning contract it targets was replaced by #817 (#701, adoption gate), #922/#1005 (promoted-method learning) and #995 (blocked-task proposals). A 49-file rebase across those three rewrites is larger and less reviewable than re-deriving the capability on today's contract, and re-implementation is the only route that can satisfy B7's criterion — a dreamt item must demonstrably change the next answer, which #887 predates. Keep its `.lino` corpora and test fixtures as input |
| #652 — Plan the critical missing features as tracked sub-issues; refresh vision and roadmap (#651) | 2026-07-12 | 2026-07-12 | +6,238 / −324 across 27 files | CONFLICTING | **close with reason; its live output is already merged** | Its purpose was to file E35–E55 as sub-issues and refresh the vision and roadmap. The sub-issues exist and most are closed; the doc refresh it proposed is superseded by plan 11, which audits against measurements this branch will take rather than against July's. Closing it removes a 27-file conflicting diff that would otherwise have to be reconciled with plan 11's rewrites of the same files. #651 stays open as the parent epic |
| #644 — Add experimental formalization model fallback (#483) | 2026-07-08 | 2026-07-09 | +4,164 / −56 across 45 files | CONFLICTING | **close pending a maintainer decision on #483** | The branch adds a neural fallback, which the standing NON-GOAL on neural inference forbids and which #469 explicitly declined for the same reason ("needs neural inference — a project NON-GOAL"). Neither rebasing nor re-implementing is defensible until the maintainer rules on whether #483 overrides the NON-GOAL. Recommending a close rather than a merge is the honest position; the requirement stays filed on #483 |

Two further open pull requests are outside the three named in the task but are
listed for completeness, because leaving them unmentioned would repeat exactly
the silent-omission failure this plan exists to catch:

| PR | opened | state | recommendation | reason |
| --- | --- | --- | --- | --- |
| #646 — Fix sidebar splitter affordance and scrolling clarity (#447) | 2026-07-10 | CONFLICTING | close; re-file the splitter fix on #447 | #447 is in the #1087 frontier queue for its routing half; the splitter is a separate UI change and the diff conflicts with every later `main.jsx` edit |
| #643 — Add polished multi-framework UI skins and color themes (#557) | 2026-07-08 | CONFLICTING | close; #557 stays open | 174 files of UI theming, unrelated to the meta algorithm, conflicting since July |

## Issues this PR will close

Thirty-seven issues, derived from the "full" rows of the coverage table. Each is
closed only when the plan leaf named in that table is ticked and its gate is
green; a leaf that cannot be finished removes its issue from this list and
records why, rather than closing it on prose.

```
Closes #453
Closes #491
Closes #705
Closes #710
Closes #720
Closes #721
Closes #722
Closes #724
Closes #801
Closes #802
Closes #821
Closes #826
Closes #827
Closes #838
Closes #901
Closes #930
Closes #935
Closes #937
Closes #939
Closes #941
Closes #942
Closes #948
Closes #949
Closes #950
Closes #952
Closes #954
Closes #957
Closes #958
Closes #959
Closes #1063
Closes #1071
Closes #1083
Closes #1087
Closes #1089
Closes #1090
Closes #1137
Closes #1138
```

### Issues this PR will NOT close, with one-line reasons

Twenty-four issues. Thirteen are partially advanced and stay open with a
narrowed remainder; eleven are outside what this pull request can honestly
deliver.

| issue | why it stays open |
| --- | --- |
| #447 | routing leaves land; the mobile layout redesign does not |
| #483 | maintainer decision needed — the ask conflicts with the NON-GOAL on neural inference |
| #557 | UI redesign unrelated to the meta algorithm |
| #651 | parent epic; stays open while any of its children does |
| #665 | npm publishing needs registry credentials |
| #666 | marketplace publishing needs publisher credentials |
| #667 | a debugging UI, not meta-algorithm work |
| #668 | package distribution needs a registry decision |
| #669 | cloud sync needs a hosted service and credentials |
| #670 | docker execution lands; the WebVM spike does not |
| #700 | unit limitations are fixed; adopting `si-units` is a dependency decision |
| #800 | retrieval and ranking land; a source that answers 403 cannot be made to answer |
| #825 | UI feature unrelated to the meta algorithm |
| #836 | advisory wording and jurisdiction scope are a maintainer product decision |
| #861 | needs a third-party account, a DSN and a privacy decision |
| #869 | routing and timezone reasoning land; a calendar integration does not exist |
| #872 | retrieval and ranking land; App Store coverage depends on what the source serves |
| #934 | the ratchet turns downward; absorbing all ~26,700 JS lines is multi-slice |
| #940 | research lands; PDF/DOCX rendering is blocked on `meta-language` upstream |
| #951 | the first clusters move; the whole 9,269-line file is multi-slice |
| #953 | permission logic moves; removing committed minified bundles is a separate slice |
| #955 | the checklist lands; live-device and deployed-demo checks need the maintainer |
| #1084 | the workflow change lands; the published manifest is only verifiable after merge |
| #1088 | the hashed index lands; creating the evidence repository is an organization action |

## Risks and open questions

1. **Thirty-seven `Closes` lines are a promise, not a measurement.** The honest
   failure mode of this repository, documented by #710's audit and by C6, is a
   pull request that closes issues on prose. The mitigation is mechanical: each
   row of the coverage table names a leaf, each leaf names a gate, and an issue
   leaves the `Closes` list the moment its leaf is struck through. Plan 14 owns
   the ordered leaf list; this plan owns the mapping from leaf to issue.
2. **Eighty-three still-open remainders against twelve plans is a scope that can
   only be met by generalization.** If any remainder is met by adding a handler,
   a seeded phrase or a per-prompt branch, it has been met in the way B9
   forbids and will reappear. Every grouped obligation above must land as
   registry data or as a kernel method, and plan 09's ratchet is the detector.
3. **The `other` group has no owning bottleneck and is the likeliest place for
   silent loss.** Fourteen labelled obligations — ignored-test debt, the Chrome
   framework seal, the E2E observation layer, the CI cache pool, the macOS
   archive build, the JS lint gate, multi-arch publishing — belong to no plan
   01–12. They are listed here with explicit new-leaf assignments so that
   "not a bottleneck" never becomes "not tracked".
4. **Three remainders are upstream and cannot be closed here** (C11, C50, E4's
   residue): `meta-language` code emission from an `insert_*`-built network, a
   shared executable semantic query IR, and the harness-schema contracts the
   agent CLIs advertise. #935 asks for two of these to be filed; filing is the
   deliverable, not the fix.
5. **Five remainders are maintainer decisions** (C51 training-source approvals,
   C79 shared cache deletion, C82 branch protection on direct pushes, #483's
   neural fallback, #836's advisory policy). Each is named rather than quietly
   resolved in a plan's favour. Open question: should this pull request stage
   the branch-protection change for review, or only record the decision?
6. **#1066 is the precedent this plan exists to prevent.** It was closed by
   #1067 with five of six acceptance items undelivered, in a body that said so
   plainly. Re-opening it is one option; absorbing its five items into plan 03
   and plan 12 is the option taken here. Open question for the maintainer:
   should #1066 be re-opened so the record shows it was never finished, or is
   C83's entry in this ledger sufficient?
7. **The carry-over sweep is text-based and can only find remainders a body
   states.** A pull request that narrowed scope silently — which #710's audit
   found to be "the dominant failure mode" for closed issues above #350 —
   leaves no phrase for `grep` to match. The 500 bodies were swept; the 557
   closed issues were not swept in full, only the 93 carrying an E-number.
   Open question: is a full sweep of all 557 closed-issue bodies and comment
   threads worth the cost, or does plan 11's requirement-by-requirement audit
   of `REQUIREMENTS.md` and `docs/requirements-traceability.md` cover the same
   ground more reliably?
8. **PR #887's 6,804 added lines are being recommended for deletion rather than
   rebase.** If the maintainer prefers the rebase, plan 07's leaf order changes
   and the anticipatory-dreaming corpora land before the adoption contract
   rather than after it. This is the single largest reversible decision in this
   plan.
