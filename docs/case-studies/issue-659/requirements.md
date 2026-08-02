# Issue 659 requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| I659-01 | Scan Rust production sources for user-facing natural-language literals. | Scanner fixtures cover sentences, raw/escaped strings, and context-sensitive phrases. |
| I659-02 | Detect sentences with spaces and terminal punctuation. | Inline `rust-script --test` sentence fixtures. |
| I659-03 | Detect multi-word phrases in `format!`, `push_str`, and return positions without treating every internal token as prose. | One regression fixture covers all three contexts and negative code/token cases. |
| I659-04 | Exclude comments, character literals, trace/event slugs, and code snippets. | Inline lexer and heuristic fixtures. |
| I659-05 | Seed a sorted allowlist whose rows carry relative file paths. | Check mode parses the committed inventory; round-trip/order tests pin its format. |
| I659-06 | Reject new debt and stale migrated rows, so the inventory can only shrink. | Diff fixtures cover an unknown literal, an admitted literal, and a stale row. |
| I659-07 | Fail on the issue's exact `Sorry, I can't do that.` fixture. | Minimum temporary-source regression fixture. |
| I659-08 | Wire the gate into CI and the contributor local-check list. | Workflow/document traceability test plus the CI lint step. |
| I659-09 | Migrate at least one real entry into seeded meanings in this PR. | `engine_responses` reads the existing multilingual seed; eight fallback rows disappear. |
| I659-10 | Preserve the complete issue, PR feedback, and verification evidence in a case study. | `docs/case-studies/issue-659/` with raw GitHub captures and requirement mapping. |
| I659-11 | Use Formal AI auto-learning over the gate's observed failure/debt network and keep adoption human-review gated. | Persisted Links network, derived report tests, and an explicit promotion gate. |
| I659-12 | Execute issue 659's learning task through Formal AI as the model behind a real external Agent CLI. | Live server/Agent-CLI transcript and CI E2E step. |
| I659-13 | Make the self-hosted artifact reproducible rather than hand-authored. | Committed report must equal the general learning-report renderer byte for byte. |
| I659-14 | Prove generalized routing with different natural-language wording and no issue-specific planner branch. | Descriptor-table routing tests plus differently worded in-process and live requests. |
| I659-15 | Exercise the whole task, not isolated helpers only. | Whole-task test asserts routing, write, verify, final answer, report identity, gate, and source observations. |

The original issue defines I659-01 through I659-09. I659-10 through I659-15
come from the maintainer's 2026-07-18 PR feedback and the standing self-hosting
rules in `CONTRIBUTING.md`.
