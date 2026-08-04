# Requirement audit — items #156–#310 (link-assistant/formal-ai)

Extracted 2026-08-04 from local NDJSON dumps. 155 items processed (78 PRs — all merged; 77 issues — all closed). 69 substantive konard comments read word-for-word (all 488 range comments triaged; the other 419 are hive-mind bot templates posted under konard's account). 136 requirements recorded in `req-chunk-156-310.ndjson`.

## Shape of this range

This range covers three phases:

1. **#156–#243 — dogfooding fix wave (2026-05-19 → 2026-05-25).** Users (xlabtg, levi-akkaman, veb86, lion-lef, eg0rmaffin, netkeep80, konard himself) file auto-generated "Unknown prompt / Issue with dialog" reports from the web app; konard turns each into a class-level requirement in a comment, and a konard-account hive-mind PR delivers it. Every such report got a paired fix PR.
2. **#244–#245 — the vision-planning axis.** Issue #244 ("Plan issues to implement our vision fully") and tracking PR #245 drive six audit passes producing epic batches E1–E14 (#246–#259), E15–E20 (#278–#283), E21–E27 (#298–#304), and (outside this range) E28–E32 (#313–#317) and E33–E34 (#326–#327). All epics in range were delivered by merged PRs #260–#277, #285–#297, #305–#310.
3. **#284–#297 — post-epic regression fixes** (name setting, Russian concept lookups, localized rule listings).

## Standing clauses konard repeats (the "boilerplate constitution")

- **Case-study clause**: download all logs/data to `./docs/case-studies/issue-{id}` (later also `pull-request-{id}`, per #234), reconstruct timeline, list every requirement, find root causes, propose solutions, check existing components. Pasted on ≥18 items in this range (#180, #185, #190, #193, #195, #196, #205, #207, #210, #218, #221, #222, #226, #228, #230, #232, #234, #242, #244).
- **Debug-output clause**: if root cause can't be found, add debug/verbose output for the next iteration.
- **Upstream-issue clause**: if another repo is at fault, file a GitHub issue there with reproducible examples, workarounds, and fix suggestions.
- **Single-PR / unlimited-time clause**: "plan and execute everything in this single pull request, you have unlimited time and context… until each and every requirement fully addressed."
- **Deepest-and-widest clause**: all changes correct, consistent, validated, tested, documented, logged; re-list every requirement from the issue AND all comments before checking them off; from #238/#243 extended with "The scope is the entire repository" and "Nothing should be deferred or delayed."
- **Language parity (en/ru/hi/zh)**: the single most-repeated demand — konard pasted a variant on **at least 15 threads** (#175, #177, #196, #198, #200, #201, #202, #204, #214, #215, #219, #227, #229, #231, #233, #240, #243, #292) with escalating frustration ("I asked for that already multiple times, and we still repeating the same mistake"). Materialized as `check:language-parity` (diff-aware CI guard) + `check:intent-coverage` coverage groups around 2026-05-22.
- **Anti-fake / anti-memoization**: "stop faking solutions"; general logic hitting real Wikipedia/Wikidata/Wiktionary APIs; only raw HTTP requests/responses may be cached (max 128 most-frequent words/entities/properties, human-readable `.lino`, ≤1500 lines per file, never base64); never preseed derived dictionaries or logic. Stated hardest on #208, #221, #222, #200.
- **Formalize → reason → deformalize**: everything flows through a semantic meta language of doublet links (distinct meanings ≈ Wikidata Q/P ids); translation = formalize + deformalize. Stated on #180, #207, #208, #221, #230; codified as E3/E6/E22.
- **Tests-as-docs**: exact expected answers (one of several explicit variants) next to prompt variations, not just contains-assertions; CI rule to enforce the style (#234).
- **Tone rules**: never command the user, polite suggestions, natural human phrasing with random variations, "our web app" not "demo", preserve the user's original formatting (#169, #171, #176, #207).
- **PR hygiene**: `Fixes <url>` not `Addresses` (#234); PRs reviewable (<3000 files, #222); resolve conflicts with default branch (many threads); e2e tests before publish locally AND after publish on GitHub Pages (#171).

## Items that look silently dropped or dubiously closed

1. **#171 — in-browser frame-policy detection.** konard explicitly rejected the agent's "we can't preflight iframe policy from the browser" conclusion: *"I don't like the idea that we just give up. Search online, find a way. We may use external APIs for that."* The PR merged with iframe previews simply removed for navigation prompts. No later evidence a real detection mechanism (external API + e2e test) was ever built. **Most clear-cut dropped requirement in this range.**
2. **Upstream issue filings promised but unevidenced** (three separate cases):
   - #164: report missing currency feature to `link-assistant/calculator`;
   - #185: report missing library/features to `link-foundation/relative-meta-logic`;
   - #209: request wasm-to-wasm compilation feature on relative-meta-logic.
   All three parent fixes merged, but no link to a filed upstream issue appears in any thread in this range.
3. **#159 — reusable report-URL decoding script** ("so we don't waste too much tokens on reimplementation") — no delivery evidence.
4. **#172 — wikihow.com fetch/API availability check** — PR #186 delivered how-to discovery, but wikihow integration specifically is unevidenced.
5. **#180 — full raw HTTP request/response display in diagnostics** ("I clearly asked many times… expandable… make it feel real, not something fake") — repeatedly requested before and during this range; no comment in-range confirms it shipped to konard's satisfaction.
6. **#295/#296 doctrine tension**: the fixes for "Что такое антирежим?" and "Что такое ложная тотальность?" are titled *"Seed Russian … concept lookup"* — i.e. seeded/memoized answers, which is exactly the approach konard bans elsewhere ("never memoize", "learn the entire class"). konard did not object in-thread, but these closures are dubious against his own standing doctrine.
7. **Issues closed with empty descriptions and no konard comment** (#160, #167, #168, #182, #184, #187, #192, #212, #213, #216, #217, #237, #262, #272): all were nonetheless matched to fix PRs (#176, #170, #201, #200, #198, #197, #215, #214, #219, #234, #276, #277 respectively) — not dropped, but the requirement was inferred by agents from the title alone.

## Unanswered konard questions

- #171: "Do we actually have a way to test that github.com is not available in iframe?" — answered negatively by the agent; konard's counter-challenge (find a CORS-free way / external API) never got a definitive answer.
- #242: "May be we can use wikidata sources for each separate term?" — phrased as a question; PR #243's report covers dictionary sources but doesn't explicitly answer the per-term-Wikidata idea.
- #245 (fifth pass): "do we really ready to universally solve any problem…?" — answered honestly in the fourth-pass audit ("Not yet, benchmarks 0/5") and then claimed complete in the sixth pass (benchmarks 10/10, E1–E34 merged). The chain of claims is internally consistent but rests on agent self-audit, not independent verification by konard in-range.

## Notable dynamics

- **Escalation pattern**: konard had to declare work "fake" three times on the translation arc (#208 once, #222 twice) before the raw-API-cache architecture was accepted. The translation requirement (formalize→SML→deformalize) was restated at least five times (#180, #207, #208, #218, #221) — the single most-repeated functional requirement in the range.
- **E-epic provenance**: bodies of #246–#259, #278–#283, #298–#304 are AI-drafted plans posted under konard's account, but each traces to explicit konard directives in #244/#245 comments; #298–#304's texts quote konard's #245 comment verbatim as their justification.
- **Numeric constraints stated as durable rules**: ≤3000 files/PR, ≤1500 lines per `.lino` file, ≤128 cached most-frequent words/entities/properties, ≥2 last messages in report URLs, summary ≈20% capped at ~30 statements (configurable), expansion to 200% bounded by NSM semantic primes.
- **Everything merged**: all 78 PRs in the range merged; no rejected threads. "Rejection" in this repo takes the form of konard's follow-up comments calling delivered work fake/partial, followed by more iterations on the same PR.
