TASK (tag TEXT-CAPABILITY): deliver R1188-U18 to U21 (`docs/requirements/issue-1188-user-requirements.md`, section "Text understanding"). These are four capabilities over real text, each with a benchmark, a ratchet and a CI gate. They must be generalized: one formalization pipeline serves all four.

Shared foundation: formalize text → statements in the links network → (translate | extract | summarize) → deformalize. JS first (`js/worker`, `js/agentic`), then the Rust twin. Use existing code wherever it exists:
- the summarization pipeline (R197–R205);
- `formalize` / `deformalize`;
- the meta translation (R526);
- the formalization probe set (issue #1186);
- the web capture cache (`js/source-cache`).

1. **U20, requirement extraction from issues:**
   - **Corpus:** for each `docs/requirements/issue-NNNN-*.md` shard whose issue body is cached or fetchable (`gh issue view N --json body` → cache under `data/benchmarks/issue-requirements/`), the reviewed rows are the gold list.
   - **Route:** "List the requirements of this issue: <text>" returns one requirement per stated obligation. That covers imperative and modal sentences, list items, and "must/should/need" clauses in all five languages, with the vocabulary in seed.
   - **Measure:** recall and precision against the gold rows, matched by shared content words after normalization. Record the measures in `data/meta/text-capability-ratchet.lino`; the ratchet goes up only.
2. **U21, dependency summarization:**
   - Build the statement graph: statement B depends on A when B refers to A's subject or concept, or elaborates it.
   - Keep the roots and the most-depended-on statements; drop statements whose formal content is already present (deduplication by formal identity, not by string); deformalize concisely.
   - **Corpus:** cached Wikipedia intros and the issue bodies.
   - **Measure:** compression ratio, retained key facts (the gold is each page's first-sentence subject plus its linked defining facts) and duplicates removed. Extend the issue #893 ratchet or add rows to the new ratchet.
3. **U19, round-trip translation of sentences and texts:**
   - Translate source → meta → target → meta → source for all 20 ordered pairs of en, ru, hi, zh and es.
   - Where several target surfaces exist, choose the one whose round trip survives best (meaning identity first, surface second).
   - **Corpus:** a few hundred sentences across the five languages, built from the seed's own example sentences and cached page sentences.
   - **Measure:** per-pair survival rate, ratcheted.
4. **U18, page formalization:**
   - Fetch-and-cache a set of Wikipedia pages in five languages, under `data/benchmarks/web-formalization/`. Store them small: the intro sections only, with their source URL and revision id.
   - Formalize every sentence.
   - **Measure:** sentence coverage (the share that yields at least one statement with no unknown), fact survival through deformalize, and unknown tokens. Ratchet these.

For every item:
- A JS test in `rust/tests/web/` pins exact outputs for a few cases. A Rust test mirrors it, checked by rustfmt and CI only.
- A gate under `data/meta/ci-gates/` runs the benchmark with node within a few minutes.
- Update the R1188-U18 to U21 status cells honestly. Mark a row implemented only when its benchmark passes its ratchet in CI and the measure is stated in the row.

Rules:
- No cargo and no rust-script locally.
- Generalize: no per-page or per-issue special cases; vocabulary in seed.
- Use Formal AI for the small edits; ledger rows T440–T469.
- Claim your files in `claims.md`.
- Do not commit.
- Stop and report after each item, so LEAD can commit.
