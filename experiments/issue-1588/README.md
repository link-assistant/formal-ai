# Request spans and duration facts (#1588)

Pull request: https://github.com/link-assistant/formal-ai/pull/1596

A document containing a duration and a day word opened the conversion handler
because connector vocabulary counted as intent, and the nearest unit could
borrow a number across intervening words. Calendar creation also accepted a
generic day word and scanned backwards through unrelated words for its date.

The fix selects a single conversion request sentence, binds the quantity to its
immediately following unit, and requires a scheduling request span with a concrete calendar anchor. Connector
lexemes retain their original meaning identities; separate seed fields describe
request starts and shorthand connectors. Explicit date-offset questions bind the
day quantity separately from the date and use Gregorian day arithmetic. SI
results use exact terminating decimals or reduced fractions and seed-defined
Russian numeral forms. Existing formula and captured Wikidata conversions remain.

## Reproduce and verify

Run these small checks separately, without a Rust build:

```sh
node --test --test-concurrency=1 rust/tests/web/issue-1588-intent-spans.test.mjs
node experiments/issue-1588/probe.mjs
node experiments/formal_ai_subagent/local-gates.mjs
```

The probe calls the complete browser worker solve path. The socket regression
posts the original four inputs and two explicit conversion questions through
`/v1/chat/completions` in a fresh temporary home. The same JSON fixtures
are included by `rust/tests/unit/issue_1588_intent_spans.rs`. Native execution is
reserved for hosted CI under the repository build policy; local results do not
claim native execution. The focused PR type check covers changed native source.

| Requirement | Implementation and verification |
| --- | --- |
| Original duration documents never convert or schedule | All original inputs and each Russian line alone are shared document fixtures; both runtime tests reject conversion/event intents. |
| Explicit requests still convert | English/Russian year-to-day questions plus Hindi, Chinese, Spanish and polite English requests use the same span recognizer and fixtures. |
| Quantity stays with its unit | Adjacent binding, a separate-sentence control, and a direct intervening-word rejection check. |
| Russian agreement and exact values | `2 года = 730,5 дня`, `5 лет`, `21 день`, `22 дня`, plus exact `8 hours = 1/3 days` fixtures. |
| Correct date offset | Original Russian request returns `4 января 2027 года`; English, leap day, non-leap year and backwards controls freeze full answers and result-date evidence. |
| Route evidence and runtime twins | `before.jsonl` records the original 20 probes on the base revision. `after.jsonl` records 57 final probes, including later multilingual and calendar intent controls. Both retain selected-rule/handler evidence; successful answers also retain their bound spans or date derivation. |
| Integration and release preparation | JS suite is registered in the fast PR inventory; native shared-fixture suite is registered in the unit test root; embedded seed mirrors and changelog fragment are updated. |

The original incorrect answers were `2 лет = 1461/2 день`,
`2 hours = 1/12 day`, an invented December event, and
`10 дней = 40/1461 года`. The first three now decline those task handlers.
Existing unknown/calculator fallback responses are preserved; this change does
not claim general document extraction capability. The fourth returns
`Дата: 4 января 2027 года. 2026-12-25 +10 дней = 2027-01-04.`

The related issue https://github.com/link-assistant/formal-ai/issues/1589 has no
parent/child requirement relationship with #1588. This PR covers the shared
date-offset case; its other operations are outside this issue. The issue has no
required sub-issues. All issue comments and all three PR feedback endpoints were
read during implementation.

## Development evidence

`provenance.json` freezes the six substantive reasoning units and records the
runtime driver hash and base source commit. The independent reasoning attempt in
`reasoning-attempt.log.gz` read a source file but did not diagnose the bug, so its
accepted credit is zero: 1/6 attempted, 0/6 accepted, and zero autonomous authored
leaves. Codex supplied the procedures and edits; Formal AI executed those shell
tasks. These are not autonomous synthesis or reasoning successes. The standing
5% reasoning target was missed; it is a reporting target, not a release gate.

`tool-sessions.txt.gz` preserves actual driver tool requests, failures and outcomes.
`supplied-procedures.txt.gz` preserves the external edit procedures they reference.
Git retains the resulting diff. Codex prepared the external procedure files and also performed read-only
workspace, metadata and execution-state inspections directly. `validation.txt.gz` records independent check outputs, including the first
full local gate run and its failures, followed by the repaired run. A temporary
connector-meaning split reduced Chinese summarization compression; the controlled
`summary-probe.mjs` experiment identified the semantic identity change, and
retaining those identities restored the existing compression check without
relaxing its assertion. Missing local checker dependencies were installed outside
the tracked source tree. Rust compilation stayed in hosted CI.

Logs and supplied procedures are gzip-compressed to preserve their original
bytes, including whitespace, without adding generated log formatting to source
checks. Read one with `gzip -dc experiments/issue-1588/validation.txt.gz`.

[Current-head CI investigation](ci-round-1.md) records the complete first
published run, the exact failure causes and the batched repairs.

[Calendar intent completion and the second CI round](ci-round-2.md) record
six newly reproduced document misroutes, the shared request-span repair, and
the generated TypeScript repair reported by Layered CI.
