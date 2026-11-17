# Issue 1589: calendar arithmetic and currency provenance

This case fixes the six original prompts and the wider requested calendar behavior in JavaScript and native Rust. The shared oracle is [the 37-case fixture](../../data/parity/issue-1589-calendar-currency.json). The JavaScript regression file also exercises fresh-home HTTP responses and five-language duration output.

| Requirement | Implementation and verification |
| --- | --- |
| RU mixed months and years: 14 + 2×12 + 7 = 45 months | Adjacent quantity/unit binding in the duration reader; original prompt and generalized sums in the shared fixture |
| EN/RU complete years: March 2021 to June 2024 = 3 | Completed calendar anniversaries; original prompts, full-month variants, reverse bounds and incomplete anniversaries |
| EN/RU Gregorian day offsets: Dec 25 + 10 days = Jan 4 | Existing Gregorian offset solver with actual derivation events retained in thinking_steps; both original prompts |
| Currency: 1000 RUB to USD, two decimals, rate, source and date | Exact decimal ratios, one rounding operation, captured CBR quote; original prompt, cross rates, positive/negative half-cent boundaries |
| JavaScript and native answers plus API thinking_steps | Same expected answers/evidence in both runtimes; dedicated native CI target and fresh-home HTTP assertions |
| Ambiguous input must not get an invented calendar result | Invalid dates, mixed precision, extra numbers, unrelated prose and unsupported units decline this handler |

## Calendar conventions

A year is twelve calendar months. Sums require explicit adjacent units and an explicit output unit; fractional years are reduced rational numbers. Complete periods compare day-of-month anniversaries rather than average day lengths or end-of-month clamping. Thus Jan 31 to Feb 29 contains zero full months, and Feb 29 to the following Feb 28 contains zero full years. Month/year bounds mean the first day of those months; mixed precision is declined. Reversed bounds use the absolute interval. Recognition vocabulary and localized output live in seed tables; ambiguous or invalid inputs remain unresolved by this handler.

## Captured currency data

The rate is an offline historical snapshot, never advertised as a live quote: 2026-11-17, RUB per unit: USD 82.5143, EUR 91.1948, RUB 1. Source: https://www.cbr.ru/scripts/XML_daily.asp?date_req=17/11/2026. The raw Windows-1251 XML, hash, retrieval date and parsed values are archived in evidence/. The answer includes the rate basis, date, source and the actual computation. Signed decimal arithmetic rounds once to cents, half away from zero.

## Verification

Before the change, the initial regression draft failed 25 of 33 tests (the baseline and complete failure output are retained). After adding broader cases, all 39 JavaScript checks pass, including the original six prompts, API thinking_steps, full-month API responses and five-language duration output. Existing calendar, intent, trace and debug-session tests also pass. Native execution is required from CI; no local Rust build was run, in accordance with repository and resource limits.

Reproduce:

```sh
node --max-old-space-size=192 --test --test-concurrency=1 rust/tests/web/issue-1589-calendar-currency.test.mjs
node --max-old-space-size=192 --test --test-concurrency=1 rust/tests/web/server-debug-session.test.mjs
node --max-old-space-size=192 --test --test-concurrency=1 rust/tests/web/default-branch-build-policy.test.mjs
node experiments/formal_ai_subagent/local-gates.mjs
# Adequately provisioned CI only:
cargo test --manifest-path rust/Cargo.toml --locked --test calendar_currency -- --test-threads=1
```

The complete local gate inventory was run. Failures and subsequent focused corrections are retained. The dependency audit reports GHSA-fc5q-39xc-vjpf in existing desktop/editor picomatch dependencies. At investigation time the official advisory has no first patched version and the registry latest is 4.0.7. An isolated install of upstream proposed fix commit 1bf0d04563cb9ce568cf36efc5f6b7187d9911d5 still fails npm audit. No audit suppression, unsupported version claim or dependency downgrade is included. Current-head CI evidence and remaining acceptance limits must be assessed before marking this PR ready.

## Development boundary and reasoning assessment

Actual development actions were executed by the repository's JavaScript Formal AI driver and its real tools. The bootstrap adapter and supplied procedures came from Codex and receive zero autonomous code-authorship credit. The source-repair request returned SourceRepairOperatorUnavailable without reading or modifying source. The expanded recipes are archived in supplied-recipes.json.gz rather than kept as competing implementations.

Six reasoning units were frozen before evaluating outcomes: quantity binding, complete anniversaries, ambiguous input, exact rounding, provenance, and cross-runtime/API verification. One anniversary-semantics reasoning unit was attempted through Formal AI; its complete response was a calculation_error/unparseable route. Accepted independent reasoning: 0/6; attempted units: 1/6. Supplied algorithms and interpretations receive zero independent reasoning credit. This does not meet the project's aspirational five-percent independent-reasoning target.

The evidence directory preserves failures, full responses, actual tool receipts and runtime source identities. Compressed provenance and driver logs contain real executions; they do not establish that Formal AI independently authored the supplied implementation.

The generated-file inventory exhausted 384 MiB and 1 GiB heaps by retaining repository file bodies. Its reader now inspects only the first twelve lines, one file at a time, with unchanged generator and exception rules. Header boundary, UTF-8, EOF, stale-exception and actual-repository checks pass; the repository check runs with a 128 MiB heap.

Final local verification: 39/39 feature checks pass; the complete gate run passes or skips 137/138 gates, with only check-javascript-dependencies failing. Final logs, the official advisory response and the isolated proposed-patch audit are included. Intermediate witness-review data describes an abandoned test-module draft; the final native test is the standalone calendar_currency target.

The first published-head CI round and its complete log investigation are recorded in [ci-round1.md](ci-round1.md). The final governed witness refresh changes only four Cargo fixture hashes after complete object comparison. Duration response language is selected from the actual matching seeded request prefix; Spanish also works without an accented word. This keeps compiled worker inputs current without a local Rust build.
