# Issue #1639: CI/CD audit of formal-ai against the eight pipeline templates

Collected 2026-11-25. This extends the previous round,
`dev/log/issues/1637/pulls/1638/analysis/template-comparison.md` (gaps G1-G8).
It does not repeat that work. The reference guide is hive-mind
`docs/CI-CD-BEST-PRACTICES.md` at `1825819` (2026-11-21), cited as §N.

Audited runs (raw annotation JSON is in `../ci-logs/annotations/<run>-<job>.json`;
the job lists are in `jobs-<run>.tsv`):

| Run | Workflow | Jobs | Warnings | Failures | Notices |
|---|---|---|---|---|---|
| 45636781871 | CI/CD Pipeline (`release.yml`, push 513e4d954) | 62 | 7 | 23 | 24 |
| 45636781444 | Benchmark Corpus Gate | 9 | 0 | 7 | 1 |
| 45648404039 | dependencies-latest | 1 | 0 | 4 | 3 |

## 1. What changed upstream since the last round

The template heads are the same as last round. Each was confirmed on 2026-11-25
against `gh api repos/link-foundation/<t>/commits/main`:

| js | rust | python | php | csharp | java | cpp | go |
|---|---|---|---|---|---|---|---|
| `8e72538` | `1d29f55` | `7a05d05` | `e12a25d` | `10edf2e` | `dd803a2` | `c1c60fb` | `785292e` |

All the issues filed last round are still OPEN with no comments:

- js#317 (cache v5 is a major behind)
- rust#270 (unused rust-script dependencies)
- rust#271 (MSRV leg overridden by `rust-toolchain.toml`)
- rust#272 (detect-code-changes push range)

java#29 (no pipeline-status job) and csharp#116 (no deadline on the preflight
probe) are also still open.

The best-practices guide is unchanged since `6f19ff0`/`1825819`
(2026-11-20/21). Those commits added the §1 "Commit and ready-for-review checks"
tier and the §10 per-event concurrency key. hive-mind ships a patch for each
template under `examples/ci-cd-pull-request-tiers/{js,rust,python,php,csharp,java,cpp,go}.patch`,
and none of the templates has applied its patch yet.

### Status of last round's formal-ai gaps

| Gap | Status now |
|---|---|
| G1 floating toolchain; PR clippy without `--tests` | **Open.** There is no `rust-toolchain.toml`. `dtolnay/rust-toolchain@stable` is used 70 times. `scripts/ci-clippy-profile.sh:14` is still `--lib --bins`. |
| G2 setup-node/setup-python v6 | **Fixed.** Both are on v7, and the dependency-currency gate is green. |
| G3 rust-script unused dependencies | Dependencies removed. `CARGO_BUILD_WARNINGS=deny` is still not set. |
| G4 dead `docker-build`/`docker-slim` jobs | **Fixed.** Both now use the main-only prefix plus `needs.build-artifacts.result == 'success'`. |
| G5 rustdoc main-only; Pages deploy not gated | **Open.** The deploy now fails with `pages/deployments` HTTP 500 (it was 404 last round). |
| G6 MSRV untested | Open (P2). |
| G7 SHA pinning | Open (P3). |
| G8 full tests main-only | **Open, and the cause of this round's main failures** (§3). |

## 2. Practice matrix (this round's focus areas)

Legend: Y = present, N = missing, P = partial.

| Area | formal-ai | js | rust | python | php | csharp | java | cpp | go | Best-practice rule | Verdict | Action |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Formatter on PRs | **Was N; fixed in this branch's working tree.** `native-type-check.yml:132-139` adds `rustfmt` plus `cargo fmt --all -- --check`. Before that, fmt ran only in main's `lint` lane 1 (`data/meta/ci-gates/check-formatting.lino`). | Y `release.yml:284` | Y `release.yml:488` | Y `release.yml:183` | Y (templates.yml) | Y `release.yml:164` | Y spotless `release.yml:119` | Y `release.yml:318` | Y gofmt `release.yml:139` | §3, §5 (lint = format + linter, fast-fail) | **False negative, twice in a row** (fe376028e in #1637, 15ae736ab in #1638) | F1 |
| Linter covers test targets on PRs | **P.** PR clippy is `--lib --bins` (`scripts/ci-clippy-profile.sh:14`). `--tests` and `--examples` run only on main (`:16-17`). | Y | Y `clippy --all-targets` + `RUSTFLAGS=-Dwarnings` (`release.yml:39,494`) | Y | Y | Y | Y | Y | Y | §1 commit tier, §4 | Gap (G1, still open) | F4 |
| Full tests on PRs | **N.** `test`, `doc-tests`, E2E, parity and corpus are main-only (`release.yml:446`). The PR lane runs a curated subset in `native-type-check.yml:166-265`. | Y | Y | Y | Y | Y | Y | Y | Y | §1 ready tier (`ready_for_review`) | **False negative.** All 9 test shards broke on main from green PR #1638 (§3). | F5 |
| PR trigger types | `[opened, synchronize, reopened]` (`release.yml:11-13`) | same (`release.yml:8`) | same (`:8`) | same (`:8`) | same (`:8`) | same (`:8`) | same (`:7`) | same (`:18`) | same (`:7`) | §1: `[synchronize, ready_for_review]` | Everyone diverges. hive-mind ships template patches. | F5; upstream draft U1 |
| **Release gated on lint + tests** | **N.** `auto-release` needs only `build` (which is `cargo package --no-verify`), `secrets-scan` and `release-preflight` (`release.yml:717-729`). This changed in 4c6d5653b (2026-11-21), and `docs/ci-cd/release-handoff.md` documents it. | Y (`needs` includes lint, test) | Y `release.yml:866` | Y `:561` | Y `:312` | Y `:408` | Y (via `build: needs [lint, test]`) | Y `:738` | Y `:242` | §9 "Validated releases only - all checks must pass before publishing" | **False-positive release.** Runs 45118482226 and 45275490842 published to crates.io and created GitHub releases while all four lint lanes were red. Lint lane 1 finished at 22:15, and `publish-crate` started at 22:29. | F2 |
| crates.io credential preflight | **Reports unknown.** `scripts/preflight-credentials.sh:163-195` records `unknown` every time, giving 2 warnings on every main push. | — | **Y (verified).** The `PUT /api/v1/crates/new` probe with no archive expects 400 `invalid tarball length` (`scripts/crates-publish-preflight.mjs`, wired in `scripts/preflight-credentials.sh:91-113`). | — | — | — | — | — | — | §16 "probe with a write", "report unknown, never a guess" | A warning with no action (W1). The template is already correct. | F3 |
| Test shard plan fresh | **N.** `data/meta/test-durations.lino` was recorded 2026-10-08 (run 37802763478; last edited 2026-10-12). Unrecorded tests weigh `default-seconds 0.092`. The plan predicts 846 s per shard, which is 78% of 1080 s, so it is past the 70% warn line before any test runs. | — (no sharding) | — | — | — | — | — | — | — | §5 "Size jobs and start the longest work first" | **Guaranteed warning (W2), and exit 124** on shards 3/6/1 in 3 consecutive main runs | F6 |
| Failure-log upload finds files | **N.** The generalization E2E fails before anything is in `$EVIDENCE_DIR`, and `trap cleanup` deletes `$WORKDIR`, which holds the diagnostics (W3). | Y | Y (`if-no-files-found: error`) | Y | Y | Y (security/docs: `error`) | P (`release.yml:219`, success path only) | Y (`error`) | Y | §17 "Write logs where the upload step looks" | A failure with no evidence (W3) | F7 |
| Action majors / Node 20 | **Y.** Every external action is on its latest major (checkout v7, upload v7, download v8.0.2, setup-node/python v7, cache v6, github-script v9, …). Contract test: `no_workflow_pins_an_action_on_the_deprecated_node_20_runtime`. | **N** (js#317, cache v5) | Y | Y | Y | Y | Y | Y | Y | §14, §15 | OK. No Node 20 warnings in any of the 3 runs. | — |
| `timeout-minutes` | **Y.** 0 jobs are missing one in 61 workflows. The 12 jobs without one in `release.yml` are reusable-workflow callers (`uses:`), which cannot take a timeout, and every callee job has one. | Y | Y (`javascript-first` jobs are `uses:` callers) | Y | Y | Y | Y | Y | Y | §5 | OK | — |
| Concurrency | **Y.** Job-level groups with per-run keys for non-`synchronize` events. Writers share `formal-ai-repository-writes` with `queue: max`. Workflows without a top-level group either have job-level groups or are `workflow_call`. | Y | Y | Y | Y | Y | Y | Y | Y | §10 | OK, matches the §10 per-event key | — |
| Changelog / version checks on PRs | Y (`changelog`, `version-check` at `release.yml:129,252`) | Y | Y | Y | Y | Y | Y | Y | Y | §6, §9 | OK | — |
| Toolchain pinned | N (G1) | P | Y | Y | Y | Y | Y | Y | Y | §1 / reproducibility | Open | F8 |
| Pages deploy gated by variable | N (G5). The deploy fails with HTTP 500 after 3 attempts (external). | Y | Y | Y | Y | Y | Y | Y | Y | §17 "Tell a broken pipeline from a changed world" | Open | F9 |

## 3. Warning annotations (run 45636781871; runs 45636781444 and 45648404039 have none)

| # | Job (id) | Annotation (log line) | Root cause | Fix (exact location) |
|---|---|---|---|---|
| W1a | Release Preflight (139284265280) | `UNKNOWN: crates.io publish token -- token present; crates.io offers no token-accepting read endpoint …` (line 28) | `scripts/preflight-credentials.sh:189` records `unknown` without trying anything. The comment at `:170-181` says no route except publish checks the publish scope. That is true, and the publish route itself can be probed without publishing (rust template `scripts/crates-publish-preflight.mjs`). | F3: replace the lookup in `check_crates_io` (`scripts/preflight-credentials.sh:182-195`) with the PUT probe. |
| W1b | same | `2 verified, 1 unknown -- proceeding, but the unknown probes above did not confirm anything` (line 29) | Emitted at `scripts/preflight-credentials.sh:362` because of W1a | Goes away with F3 |
| W2a-d | Test full shard 7/8 (139322454392, line 679), 3/8 (139322454545, line 638), shard 1 "full" (139322454725, line 464), 6/8 (139322454794, line 619) | `Test suite is approaching its timeout: still running after 756s (70% of its 1080s execution budget)` | Emitted by `scripts/run-with-budget-warning.sh:152`. The plan itself predicts 846 s for every shard (`shard N/8: 846s predicted`, job log). That is 78% of `TEST_BUDGET_SECONDS: 1080` (`release.yml:567`), so the 70% line (`:575`) is crossed by design. About 1,350 of about 5,690 tests are recorded. The rest weigh 0.092 s, so new slow tests land arbitrarily. Shards 3/6/1 hit the 1080 s budget (exit 124) in runs 45636781871, 45549993600 and 45490077033. | F6: raise `shards: 8` to 10 (`release.yml:455-462`) and re-record durations. Also make the planner fail when a predicted shard exceeds the warn ratio. |
| W3 | E2E agent CLI (computer-use-generalization) (139322455628, line 36) | `No files were found with the provided path: /tmp/formal-ai-serve-*.log /tmp/agent-out-*.log /tmp/formal-ai-issue-*-evidence /tmp/formal-ai-agent-evidence-*` | The step at `agent-cli-e2e.yml:341-352` sets `EVIDENCE_DIR=/tmp/formal-ai-issue-707-generalization-evidence`, which the glob matches. `run_issue_707_generalization.sh:78` creates it empty, and the failure at `:115` (`local reference rejected for notes_count_and_pack`) exits through `fail()` (`:55-62`) before anything is written. The real diagnostics (`$WORKDIR/<case>.local.json`, `.local.err`) are deleted by `trap cleanup EXIT` (`:43-53`). The upload (`agent-cli-e2e.yml:511-521`) then finds an empty directory. | F7: `fail()` copies `$WORKDIR` into `$EVIDENCE_DIR/failure/` before it exits. |

The notices are benign:

- sccache stats; "0% - 0 hits" on jobs that use prebuilt binaries
- closure-driven cache bucket notices (wordnet/wiktionary/wikidata)
- dependencies-latest notices

The failure annotations are not warnings, but they classify as follows:

- **Fmt (lane 1).** A real regression (15ae736ab) that the PR lane could not see. Fixed in 903009ca2; F1 prevents it from recurring.
- **dependencies-latest (45648404039).** Fmt plus `clippy::large_enum_variant` on `ResearchDiscoveryContext::Source`. Both are real, and both are fixed in 903009ca2.
- **Test shards 2/4/7/8 and spec 2/3 (exit 1).** Real regressions from PR #1638. The files involved:
  - `lino_parity.rs:73`
  - `minimal_core_and_seed_metadata.rs:693`
  - `full_range_coding_artifacts.rs:324`
  - `issue_1588_intent_spans.rs:28`
  - `issue_1158_pull_request_steps.rs:24`

  The working tree fixes some of them. The PR lane never runs them, so this is a false negative (F5).
- **Test shards 1/3/6 (exit 124).** W2 escalated to failures (F6).
- **E2E `notes_count_and_pack`.** A real regression that only main can see (F5).
- **Auto Release / verify-package.** The selected binary fails "Verify greetings and issue reporting". It stopped the release only by luck, because lint and tests do not gate it (F2).
- **Deploy Pages.** `POST pages/deployments` HTTP 500 after 3 attempts. External (F9).
- **Corpus shards 1/2/6.** The 25-minute job cap and exit 143. Shards 3/4 were cancelled. "no benchmark prompt reaches the unknown opener" is the aggregate of those shards.
- **Pipeline Status.** The correct aggregate.

## 4. Concrete fixes for formal-ai, ordered by impact

The markers in the "Who" column mean:

- (A) already in this branch's working tree
- (P) proposed only, because the file is outside this audit's edit scope (`release.yml`, `rust/**`, `.github/actions/**`, `data/meta/main-only-regressions.lino`, `experiments/formal_ai_subagent/evidence/**`)
- (E) editable by the parent

| Fix | Impact | Who |
|---|---|---|
| F1 PR rustfmt | P0, false negative twice in a row | (A) + (P) regression row |
| F2 Gate release on lint (and tests) | P0, false-positive releases already published | (P) |
| F3 crates.io publish probe | P1, 2 warnings on every main push | (E) |
| F4 PR clippy `--tests` | P1, false negative | (E) |
| F5 Ready tier for the heavy suites | P1, false negative (G8) | (P) |
| F6 Re-shard and re-record test durations | P1, 4 warnings plus 3 exit-124 per main push | (P)/(E) |
| F7 Generalization E2E evidence | P2, W3 | (E) |
| F8 Pin the toolchain | P2, G1 | (E) |
| F9 Pages deploy gate | P2, external HTTP 500 | (P) |

### F1. Run rustfmt on PRs (P0), already applied in the working tree

`.github/workflows/native-type-check.yml:129-139` adds `components: clippy, rustfmt`
and a `cargo fmt --manifest-path rust/Cargo.toml --all -- --check` step before
clippy. Rustfmt compiles nothing, so the step takes seconds. To finish the fix:

- Add a contract row to `data/meta/main-only-regressions.lino` (P) naming
  15ae736ab and fe376028e, with this PR's check as the guard.
- Add a contract test that the PR lane's fmt command equals the one in
  `data/meta/ci-gates/check-formatting.lino`, so the two cannot drift.

### F2. Do not publish a release from a red commit (P0)

Evidence:

- Run 45275490842 failed all 4 lint lanes, yet `Auto Release / auto publish-crate` and `auto create-release` succeeded. Lint lane 1 failed at 22:15:49Z, and publish started at 22:29:10Z.
- Run 45118482226 shows the same pattern.

Every template, and §9, gate publication on lint and tests. The publish
started 13 minutes after lint finished, so gating on `lint` costs no latency.
Minimum proposed diff (`.github/workflows/release.yml:717-729`):

```diff
     needs:
       [
+        lint,
         build,
         secrets-scan,
         release-preflight,
       ]
     if: |
       !cancelled() &&
       github.event_name == 'push' &&
       github.ref == 'refs/heads/main' &&
       needs.build.result == 'success' &&
+      needs.lint.result == 'success' &&
       needs.release-preflight.result == 'success' &&
       needs.secrets-scan.result == 'success'
```

The full §9 version also adds `test` (and `doc-tests`) with `== 'success'`.
That reverts the intent of 4c6d5653b ("Allow staged publication when
diagnostic CI fails"), so it is a maintainer decision. While three of
the last three main runs have red test shards, it would block every release.
The same change is needed in `manual-release` (`release.yml:743+`) and in the
mirrored evidence copies:

- `experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/original-release-workflow.yml`
- `experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/candidate-release-caller.yml`

The contract tests also need updating:

- `rust/tests/unit/ci-cd/workflow_release.rs`
- `rust/tests/unit/ci-cd/one_build_per_platform.rs`
- `rust/tests/web/default-branch-build-policy.test.mjs`

Finally, update the "Production admission and diagnostics" paragraph in
`docs/ci-cd/release-handoff.md`.

### F3. Verify the crates.io token instead of reporting unknown (P1)

Port the rust template's probe. `PUT /api/v1/crates/new` with a valid
length-prefixed metadata block and no archive is authenticated and
scope-checked before the archive length is read. HTTP 400 with the single
error `invalid tarball length` proves the token and crate scope without
publishing, and 401/403 is a definite denial. Proposed replacement for
`scripts/preflight-credentials.sh:182-195`:

```bash
  local body status probe meta len
  body="$(mktemp)"
  probe="$(mktemp)"
  meta="{\"name\":\"${CRATE_NAME}\",\"vers\":\"0.0.0-preflight\"}"
  len=${#meta}
  # u32 little-endian metadata length, the metadata, and no archive length or bytes.
  printf "$(printf '\\%03o\\%03o\\%03o\\%03o' $((len & 255)) $(((len >> 8) & 255)) \
    $(((len >> 16) & 255)) $(((len >> 24) & 255)))%s" "$meta" >"$probe"
  curl_headers=("Authorization: ${token}" "Content-Type: application/octet-stream")
  status="$(http_status "$body" -X PUT --data-binary "@${probe}" "${CRATES_IO_API}/crates/new")"
  trace "crates.io publish probe (no archive) -> HTTP ${status}"
  if [ "$status" = 400 ] && grep -q '"detail":"invalid tarball length"' "$body" \
     && [ "$(grep -o '"detail"' "$body" | wc -l)" -eq 1 ]; then
    record verified "$check" "publish token and crate scope accepted; archive omitted (ownership is checked by the real publish)"
  elif [ "$status" = 401 ] || [ "$status" = 403 ]; then
    record failed "$check" "crates.io rejected the publish token or its crate scope (HTTP ${status})"
  else
    record unknown "$check" "publish probe answered HTTP ${status} without the exact missing-archive confirmation"
  fi
  rm -f "$body" "$probe"
```

Do not echo `$body` to the log, because remote bodies may echo the header.
Rewrite the comment at `:170-181`: it is correct that `GET /me` is cookie-only,
but its conclusion is not. Add a fixture test that serves 400
`invalid tarball length`, 403, and 500. Optionally, adopt trusted publishing
(`rust-lang/crates-io-auth-action`, as in rust template `release.yml:941,1139`).

### F4. PR clippy covers test targets (P1, G1 remainder)

`scripts/ci-clippy-profile.sh:14`:

```diff
-  exec cargo clippy --manifest-path rust/Cargo.toml --locked --lib --bins --all-features -- -D warnings "${style_flags[@]}" -D clippy::correctness -D clippy::suspicious -D clippy::perf
+  exec cargo clippy --manifest-path rust/Cargo.toml --locked --lib --bins --tests --all-features -- -D warnings "${style_flags[@]}" -D clippy::correctness -D clippy::suspicious -D clippy::perf
```

Clippy runs in check mode, so `--tests` adds type-checking only, with no
codegen. Measure it once against the 20-minute cap of
`native-type-check.yml` `clippy`. If it does not fit, run it as a separate
parallel job.

### F5. A §1 ready tier that runs the heavy suites on PRs (P1, G8)

The main-only `test` matrix is where #1638's regressions surfaced, and only
after merge. Proposed change to `.github/workflows/release.yml`:

```diff
   pull_request:
     branches-ignore: ['e2e/**']
-    types: [opened, synchronize, reopened]
+    types: [opened, synchronize, reopened, ready_for_review]
```

Then, in `build-artifacts` (`:846`), `test` (`:446`), `doc-tests`, `lint`,
`test-agent-cli-e2e`, `test-e2e-local` and `server-parity`, widen the main-only
prefix:

```diff
-    if: ${{ (github.event_name != 'pull_request' && github.event_name != 'pull_request_target' && (github.ref == 'refs/heads/main' || ...)) && (...) }}
+    if: ${{ ((github.event_name == 'pull_request' && github.event.action == 'ready_for_review') || (github.event_name != 'pull_request' && github.event_name != 'pull_request_target' && (github.ref == 'refs/heads/main' || ...))) && (...) }}
```

Keeping `opened`/`reopened` (unlike §1) preserves the current focused-check
behaviour that the existing contract tests assert. The current concurrency keys
already give non-`synchronize` events a per-run group (`release.yml:79,135,…`),
so a ready run is not cancelled by the next push. That matches §10.

### F6. Re-shard the full suite and refresh the durations (P1, W2)

At the planner's own 846 s prediction, the total load is 846 × 8 = 6,768 s:

- 10 shards gives 677 s each (63% of 1080 s)
- 11 shards gives 615 s each (57%)

Proposed change, `release.yml:455-462`:

```diff
-          - { os: ubuntu-24.04, test-suite: full, shards: 8 }
-          - { os: ubuntu-24.04, test-suite: full, shard: 2, shards: 8 }
-          ...
-          - { os: ubuntu-24.04, test-suite: full, shard: 8, shards: 8 }
+          - { os: ubuntu-24.04, test-suite: full, shards: 10 }
+          - { os: ubuntu-24.04, test-suite: full, shard: 2, shards: 10 }
+          ...
+          - { os: ubuntu-24.04, test-suite: full, shard: 10, shards: 10 }
```

Then:

- Re-record `data/meta/test-durations.lino` from one main run with
  `vars.FORMAL_AI_RECORD_TEST_TIMES=true` (`release.yml:574,590`).
- Make `scripts/plan-test-shards.mjs --check` fail when a predicted shard
  exceeds `TEST_WARN_RATIO_PERCENT` of `TEST_BUDGET_SECONDS`. The over-budget
  plan then fails at plan time with its cause, instead of as a 70% warning
  and a late exit 124.
- Add an age check for `recorded` (for example, 30 days).

### F7. Keep the generalization E2E evidence where the upload looks (P2, W3)

In `experiments/agent_cli_e2e/run_issue_707_generalization.sh:55-62`:

```diff
 fail() {
   echo "::error title=issue #707 held-out computer-use generalization::$1" >&2
   echo "!! $1" >&2
   if [[ -n "$SERVER_PID" ]]; then
     tail -120 "$WORKDIR/${CURRENT_PHASE:-record}/server.log" >&2 2>/dev/null || true
   fi
+  # Issue #1639 (§17): cleanup deletes $WORKDIR, and the failure upload reads
+  # only $EVIDENCE_DIR, so keep the per-case reference output it is about.
+  mkdir -p "$EVIDENCE_DIR/failure" && cp -R "$WORKDIR"/. "$EVIDENCE_DIR/failure/" 2>/dev/null || true
   exit 1
 }
```

Keep `if-no-files-found` at its default (`warn`) at `agent-cli-e2e.yml:511-521`.
Once the script always writes evidence, a "No files were found" warning means
a regression in evidence capture, not a normal outcome.

### F8. Pin the toolchain (P2, G1)

Add `rust-toolchain.toml` with `channel = "1.100.0"` and
`components = ["rustfmt", "clippy"]`. Keep `dtolnay/rust-toolchain@stable`, or
switch to `@master` with `toolchain: 1.100.0` so the action and the file agree.
Avoid the rust template's MSRV trap (rust#271): an MSRV leg must override the
file with `RUSTUP_TOOLCHAIN`.

### F9. Make Pages deploy opt-in (P2, G5)

In `release.yml:982-987`, add
`vars.DEPLOY_GITHUB_PAGES != 'false' &&` to the `deploy-pages` condition. When
the deploy is skipped, emit a `::notice::`. The HTTP 500 from
`POST pages/deployments` is external. With the variable, a maintainer can turn
it off without a red main.

## 5. Upstream reports

| Draft | Template(s) | Filed? | Why |
|---|---|---|---|
| `../upstream-reports/all-templates-pr-ready-tier.md` | all 8 | **Not filed** | The current mains are verified (all 8 use `[opened, synchronize, reopened]` with no ready tier). However, this is a divergence from a best-practice rule, not a defect that produces a false positive or negative in the template itself. hive-mind already ships the patch for each template (`examples/ci-cd-pull-request-tiers/*.patch`), so filing 8 issues would duplicate upstream's own rollout. No duplicate exists: searches for `ready_for_review` and `tier` returned nothing in any of the 8 repositories. |

No other defect found this round also exists on a template's current main:

- Fmt on PRs: all 8 templates run their formatter.
- Release gating: all 8 gate on lint and tests; java does so through `build: needs [lint, test]`.
- crates.io preflight: the rust template already verifies the token.
- Failure uploads: the templates set `if-no-files-found: error`.
- Sharding: no template shards its tests.
- Concurrency and timeouts: every template job has a timeout or is a reusable-workflow caller.
