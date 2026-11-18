# Isolated native synthesis checkpoint

PR: https://github.com/link-assistant/formal-ai/pull/1581

Published input `fda677ef9f25692b7b3823556f1349839a71f7a3` completes all 23 workflows: 22 succeed and CI/CD Pipeline fails. The complete failed workflow, both failed job logs, both JavaScript shard logs, API payloads and digest receipts are retained under `experiments/issue-1580/evidence/ci/native-php-fda677ef/`.

The [actual native job](https://github.com/link-assistant/formal-ai/actions/runs/44309937639/job/134821269126) compiles and runs the focused targets. Literal lines 1624–1685 show calendar/currency **1/1**, executor **8/8**, forced functions **1/1**, and synthesis **11/12**. The two new production executor cases actually execute: remainder-sale source operands produce 36 and 72, and the membership-source count produces three. The synthesis target also passes courtesy/question ordering, filtered count, algebra and renumbered word problems. A failure stops the later workflow commands; this checkpoint does not claim a new native HTTP pass.

The sole synthesis failure is `shared_cross_runtime_synthesis_fixture_matches_rust_solver`, at `synthesis.rs:88`. Native log lines 1670–1685 preserve its complete diagnostic:

```
actual: The Halley research station, opened in 1956, is used to study the Antarctic ice shelf. It provides year-round measurements of ozone and sea temperature.
stale expected: The Halley research station opened in 1956 is used to study the Antarctic ice shelf.
```

This is a stale dormant test expectation. Commit `ea67811fd82f38f12ca91e2d6c042d7e0afc8801` made free-text summaries rank complete sentences so governed clauses remain intact. Both existing handler specifications, `rust/tests/unit/web-engine-core/issue_1174_text_transform.rs` and `rust/tests/web/issue-1174-text-transform-parity.test.mjs`, already pin the two complete sentences. Both production implementations use that whole-sentence path and keep two of five substantive sentences under the existing divisor-three budget. No production summarizer, budget, shared prompt or forbidden fragment is changed.

Source commit `e3b50624c1c7f694373cb406fbb2293f49cd05a9` corrects the exact native answer to the already documented contract and identifies the case ID in future assertion failures. Exact equality remains mandatory. All **14 existing text-transform tests**, **four source-payload routing tests**, **six affected gates**, edition-2024 Rust formatting and whitespace checks pass. Complete local logs and hashes are retained in `experiments/issue-1580/evidence/checks/native-summary-expectation/`; all five logs were literally reviewed and hash verified. No local Rust build runs. Hosted execution of the correction remains pending.

The bounded twelve-case browser audit confirms seven of eleven native documented answers exactly, including Halley and all later host-independent cases. It retains four differences rather than changing assertions: algebra and word problems use explanatory browser renderings; membership counting and generated Python remain explicitly unverified at the browser executor boundary. The formalization case is host dependent and is not exact-compared. The audit exits one for those four observations. Its complete responses, expected strings and digests remain preserved; it is not a passing parity claim or independent reasoning credit.

Both Layered CI groups succeed: group 1 reports **1,190 passes / four skips**, group 2 **1,533 passes / 13 skips**, with zero failures and cancellations. Their complete payload hashes are verified; only named result ranges are literally reviewed. The complete 215-line Pipeline Status job is read: native-type-check is its only failed prerequisite, with no cancellation. Native log lines 1570–1686 were read literally; the full native and full workflow payloads are retained without claiming complete literal review.

Fresh feedback at 2026-11-18T13:20:18Z contains one issue comment, nine PR conversation comments, no inline comments, no reviews and no required sub-issues. The entire issue and all comment bodies are read. No acceptance requirement is removed. The prior publication retains all 94 command payloads with hashes; its 895-line API source patch remains retained without a literal-review claim.

Every broader unfinished criterion remains in the work plan, including full template applicability and independent Formal AI reasoning **0/20**. Supplied code, experiments and conclusions receive zero independent credit. Keep PR #1581 draft and unmerged.
