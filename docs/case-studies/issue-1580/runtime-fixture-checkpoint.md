# Runtime fixture and registry checkpoint

Draft work for [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
and [issue #1580](https://github.com/link-assistant/formal-ai/issues/1580).
The published input is `7780c70715df515a8ffcb085bff0902b4489bb94`, including
main `058e9bef1214f2f1bc25971243cb787703ed64f1`.

## Reproductions and changes

All 23 workflows on that branch head completed: 22 succeeded and
[Layered CI 44180825006](https://github.com/link-assistant/formal-ai/actions/runs/44180825006)
failed. Original log lines 9038–9062 identify the inline OpenCode test under
`rust/src`. Moving its unchanged assertions into `rust/tests/source` makes
the existing five-case placement suite pass. Its fixture include now resolves
to the canonical repository data. The maintained mirror writer removes the
unused production copy: 481 remaining demands are byte-equal. The first
post-move check failed on that unused copy; its complete diagnostic is retained.
The fixture itself and its 15 envelope cases are unchanged. All 51 JavaScript
source-provenance cases pass; the relocated native case has not executed.

[Main Layered CI 44174503458](https://github.com/link-assistant/formal-ai/actions/runs/44174503458)
records the stale Moon count at lines 15868–15880 and generated registry at
18550–18592. Two fresh workers with local seed transport isolate the count
change: removing both English causal lexemes restores 196 unknown terms;
the current vocabulary has 195, differing only in the term `because`.
All other counts remain identical: 25 sentences, 67 statements, three fully
formal sentences, 423 terms and 61/62 retained facts. Both native and
JavaScript expectations now reflect the causal vocabulary; the JavaScript
file passes all six cases in five languages. The native twin remains unexecuted.

The official native-program generator reproduces and repairs the stale
registry. A complete object comparison preserves all 157 programs and every
executable field. Sixteen source witness hashes change, and the two offsets
of `memory_sync::LEARNED_CHUNK_KIND` move by 127 characters. Each new hash is
checked against the actual source, and the moved span is checked against the
same constant. The existing generated-program suite passes. An initial
comparison incorrectly assumed only hash changes; its failed assertion is
preserved separately from the corrected, complete comparison.

Main pipeline log 44174504347 lines 25731–25739 and 43465–43474 show Claude's
existing Read/Glob/Grep/WebFetch/WebSearch allowlist missing from the expected
argv. The three affected native expectations now retain that exact prefix,
caller flags and argument ordering. Capture normalization and the separate
quiet-stderr assertion remain unchanged. Rust formatting passes; actual native
argv and quiet startup behavior remain unverified.

Main Layered log lines 31241–31257 expose another stale test expectation:
tool-binding failures moved raw schemas and arguments into a lazy private
diagnostic field. The test now checks the two exact public failure events,
absence of private fields in public metadata, and retained schema, arguments
and actionable reason in the private factory. Production logging is unchanged;
all 25 cases in the complete focused regression file pass without skips.

## Supplied-passage routing controls

The branch already marks recursive candidates as `metaSubTask` and skips
unknown research for those candidates. Removing that guard only inside a
disposable worker reproduces 26 mocked transport calls; the production guard
makes zero calls with the identical passage answer. A standalone unknown
question still makes 16 mocked calls, preserving ordinary research behavior.
No probe contacts a provider. The existing factual QA file passes 15 cases.
The parent procedure still reports unresolved status for this correct answer;
this evidence does not complete recursive procedure acceptance.

Four main-only ledger entries retain the narrow existing JavaScript replays
for the count, registry, passage and private diagnostic failures. The first
complete local run passes 86 gates with 52 explicit exclusions and zero failures.
The subsequent ledger test catches all four replays missing from the fast PR
inventory. Adding those four existing files grows that inventory from 235 to
239 files. The corrected ledger/profile files pass all 4/15 tests without skips.
The final shared-data run passes 86 gates with 52 explicit exclusions and
zero failures; no native build or runtime is claimed.

## Evidence and unfinished work

Complete focused logs, controlled reports and hashes are under
`experiments/issue-1580/evidence/checks/{runtime-fixture-refresh,moon-lexeme-probe,passage-routing-probe,native-program-registry-refresh,repaired-registry-and-external-test}/`.
Branch and fresh main inventories/logs are under
`experiments/issue-1580/evidence/ci/{checkpoint-7780c707-final,main-058e9bef-refresh}/`.
CLI and Layered logs identical to the earlier main snapshot are referenced by
hash rather than duplicated. Targeted payload review is not full-log review.

All 17 main push workflows now finish: 13 successes and four failures. The
benchmark corpus has a 900-second termination (44174504103 lines 2829–2831)
and a missing shard proof at 3335–3336; its hot-case cause remains unknown.
The full native pipeline retains failures and five 1,080-second terminations.
Main also reports the picomatch advisory; this branch already carries its
separately tested backport, so the default-branch failure alone is not proof
of a new branch defect. No new security fix is attributed to this checkpoint.

Feedback remains one issue comment, nine PR conversation comments, no inline
comments and no reviews. All earlier native, benchmark, warning, dependency
migration, template semantic review, historical evidence and live identity
criteria remain unfinished. Local native execution remains forbidden. The
merged policy permits only the existing focused clippy job and its isolated
calendar/currency CI regression on PRs; other native suites and builds stay on
main. Independent Formal AI reasoning acceptance
remains 0/20; supplied scripts, patches and conclusions earn zero credit.
The PR remains draft and unmerged.

Main advances to `015a683197c89e8f74375b097e8dd06a57dcea8f` during this
verification. The merge retains the calendar/currency implementation, fixtures
and isolated native CI regression, while preserving our always-reporting PR
context and compile-input conditions. Both parents’ generated-header tests
remain; the manifest contains the union of their 240 JavaScript test files.

The reviewed source-reader refresh changes exactly four Cargo manifest hashes
from our parent. All 221 governed cases, 964 unsupported cases, native source
manifest entries and other executable values remain identical. A full comparison
against the incoming parent also checks all changed source values and hashes
against actual files. All 15 combined-source focused checks pass, including the
new calendar/currency suite, 13 clock-preservation cases, six Moon summary cases,
client defaults/permissions and generated projections. Rust formatting passes
without native execution. Complete logs and comparisons are in
`experiments/issue-1580/evidence/checks/main-merge-015a6831/`.
The full combined-source gate run and hosted checks remain pending.
