# Completed CI round and warning repairs

Published source `d1d11e6039845ead74f1cb229266acb9ea42823d` completed all
34 observed workflows: 30 succeeded, one skipped and three failed. The full
34 workflow streams and all 31 annotations from 173 checks are retained with
original and compressed byte hashes. A complete-stream diagnostic scan found
the worker compiler error, stale worker receipt, source/checkout failures,
missing interface companion, nine deprecated artifact-download calls, one
Hive timing warning, the deliberate BuildKit outage annotation, two SQLite
experimental imports and one experimental timer mock. Selected diagnostics
and their actual source were reviewed; the entire 68,174-line Pipeline stream
was not literally read. The passing test name containing “warn category” is
a scan false positive, not a runtime warning.

The source and native companion repairs are recorded in
[the bootstrap checkpoint](bootstrap-ci-repair.md). The standalone wrapper is
now receipt-bound together with the main binary. All ten family groups reuse
verified finite executables in the regression, with zero Cargo invocations.
This fixture result does not replace the next hosted native execution.

The Hive workflow test combined four complete language sessions into one
696.84-second case and crossed the unchanged 675-second warning boundary.
Extract exactly its existing session/assertion body into one helper and expose
Kotlin, Scala, Rust and Python as separate real tests. Both complete bodies
have the same normalized SHA256. Four measured-weight shards retain every
listed test exactly once, two concurrent jobs, single native test threads,
the 900-second command budget and the 20-minute job clock. New language weights
are explicitly labeled estimates of 169 seconds, derived from the historical
672.933-second aggregate; they are not new hosted duration observations.
All 62 runner/admission regressions pass; rustfmt passes without a Rust build.

The first BuildKit annotation repair incorrectly assumed that a command guard
could cross workflow-step boundaries. Its 11 local controls passed, but the next
hosted run retained the failure annotation. The same-process correction and
its new negative control are described below; that hosted observation supersedes
the earlier local inference. The production setup action remains unchanged.

The two SQLite consumers now explicitly select Node 24.22.3 instead of Node 22
or the runner's ambient executable. Its upstream SQLite module no longer calls
the experimental-warning emitter; the API is still a release candidate, not
fully stable. See the [official API history](https://nodejs.org/docs/latest-v24.x/api/sqlite.html)
and [versioned implementation](https://github.com/nodejs/node/blob/v24.22.3/lib/sqlite.js).
The regression exercises a real SQLite round trip and emits an independent
sentinel to prove that process warnings remain enabled. Local Node 26 execution
does not qualify the selected hosted Node 24 binary; that awaits CI. The link
test uses stable method mocks, observes the exact remaining 15 ms budget,
asserts cancellation and verifies timer cleanup. All 16 link tests pass.

The crates.io warning is an honest unavailable credential observation, not an
invalid-token verdict: a public crate lookup cannot verify a publish-scoped
token. No credential is sent to a cookie-only endpoint and no publication is
performed to manufacture verification. Registry uncertainty remains explicit.

The final maintained entry passes 335 cases. Production checkout tests pass
21/21; the two runtime checks, the touched size/readability/filename/scheduling
and workflow gates, and whitespace checks pass. Earlier incorrect gate names,
the mistaken test job key, source-selector corrections and failed before-fix
controls are retained separately and receive no verification credit. The full
local inventory's earlier 87 passes, 53 hosted-only skips and one genuine stale
worker failure remain its actual outcome, not 88 passes. Hosted worker
regeneration, final-head native checks, complete template/practice acceptance,
whole-PR review and the independent-authorship floor remain unfinished.
Supplied procedures and conclusions receive zero autonomous credit.

Evidence: `experiments/issue-1580/evidence/checks/terminal-warning-repairs/`.
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.

## Final seed warnings and same-head hosted regressions

The current file-size gate still reported five warnings: the translation meaning and agentic response seeds, their package mirrors, and the generated staged-release workflow. The new focused regression fails on all five original owners. Warehouse occupations, source-backed composition templates and quantity-comparison templates now have registered owners below the unchanged limits. The maintained registry and package-mirror writers generate every consumer; tests preserve every original meaning field through the maintained concise-lexeme conversion, reconstruct the complete response source bytes, and use actual lexicon/response lookup for every moved record. The final three regression checks pass.

The staged-release generator emits artifact-download inputs as YAML flow mappings. The generated workflow decreases from 1,412 to 1,360 lines. Full parsed YAML is exactly equal before and after, including every job, condition, input and shell command; all 52 operations and 104 cross-stage bindings remain present. The retained reproduction script performs this comparison before writing the maintained projection.

Hosted Layered CI on `de79e6ec` failed at log lines 4454–4484 (VS Code sparse-checkout order), 16075–16118 (shell description expectations and enum population), and 22540–22561 (the known stale worker). The full 25,377-line hosted log is preserved separately. The checkout test now compares every directory without imposing an irrelevant order, and its real sparse Git exercise passes. The description test retains main's selected-command narration behavior. Native, JavaScript and TypeScript projection now leave an unmade description enum choice absent while preserving explicit values/defaults. Focused results: 23 projection tests, nine native-release workflow tests, 13 JavaScript client tests and three headroom tests pass; the native client test remains an explicit hosted-only exclusion. The touched native source passes rustfmt. Worker compilation has an independently verified hosted receipt; no local Rust compilation was used.

These local results do not certify the still-running full hosted round or the remaining issue-wide applicability/authorship criteria.

The current native job also rejected the Qwen provider setting in `client-integrations.lino` at line 524: C-style escaped double quotes are not valid canonical Links Notation. A single-quoted notation value carries the unchanged JSON payload. The new two-case regression reproduces the canonical-parser failure before the fix, then checks canonical parsing, complete provider JSON, the correlation header and package-mirror equality after it. The existing native all-data parse regression will verify the same source in hosted CI.

The same-head native interface shard then exposed three stale argument expectations (hosted log lines 822–979). Main’s managed Claude configuration explicitly adds `--permission-mode default` before caller arguments. Four complete expected vectors now include those two arguments, covering interactive/non-interactive execution, argument/piped prompts and repeated caller flags. Production permission behavior is preserved; native execution remains a hosted check. Both changed test files pass rustfmt without compilation.


## Complete-log warnings from de79e6ec

The terminal CI/CD run 44962289753 passed all four Hive contract shards.
Its two native failures were the canonical Qwen seed quotation and stale
Claude argv expectations described above. The complete Layered CI logs also
revealed warnings outside failed checks, so the batch includes these repairs:

- RML's pinned source exporter imports the canonical notation parser and its
  source Lean exporter; esbuild is a browser-build development dependency.
  The install now explicitly ignores lifecycle scripts. A real offline npm
  fixture runs the actual workflow install and launcher, demonstrating that
  the old install executes its sentinel and the new install leaves it absent.
  The maintained real `rml export lean` check remains enabled in hosted CI.
- A stop-command in a separate shell step did not affect annotations emitted
  by the subsequent setup-buildx Node action. The negative-control preload now
  changes only its exact deliberate outage command to a notice inside that
  same process. Its exit code and every unrelated error remain unchanged;
  fallback failure, daemon identity and image digest are independently fatal.
  Twelve focused controls pass; hosted annotations still require verification.
- Observed JavaScript shard durations included a 316.415-second provenance
  file, with 309.297 seconds in its primary test. Reweighting alone could not
  fit that indivisible file under the unchanged 270-second warning point.
  Installed-source proof, archived-source proof and negative controls now run
  as three independent suites using the same fixture builder. A source-bound
  regression preserves every original setup and assertion block and the other
  tests byte-for-byte. New suites inherit the full measured parent cost as
  explicitly labelled conservative scheduling estimates. Their individual
  durations are Unknown until hosted execution. The replay asserts headroom
  for unchanged measured suites and keeps all split scenarios in separate
  first worker waves; it does not claim measured headroom for new suites.

All original execution budgets, warning ratios, independent trust pins,
provenance refusal cases and eight JavaScript shards remain in force.
These supplied repairs earn zero autonomous-authorship credit. The broader
template practice and authorship acceptance criteria remain open.

The complete `de79e6ec` round is terminal: 31 successful workflows, one skipped
workflow and two failures. All 34 original streams and all 174 terminal check
records/annotations are archived under `evidence/ci/de79e6ec-terminal-round/`.
Both actual native failures and all three JavaScript failure classes are
repaired locally. The complete local inventory executed 88 JavaScript gates
and explicitly excluded 53 hosted-only gates; the two draft failures were
repaired and rerun under unchanged policies. Rust formatting uses edition 2024
and checks only the changed files. No local Rust build was performed.

Fresh hosted execution still must verify the native expectations, source parsing,
BuildKit annotation boundary and the duration of each new provenance scenario.
The unavailable crates.io credential observation, remaining template practice
acceptance, complete PR review and genuine authorship floor remain unfinished.
