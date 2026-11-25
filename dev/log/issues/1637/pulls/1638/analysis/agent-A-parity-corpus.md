# Agent A: corpus gate, server parity, native capture, verify-package, Pages

Runs analysed: Benchmark Corpus Gate 45549993299, CI/CD 45549993600, Layered CI 45549993081 (main, eee485204).
No cargo builds were run. JS checks were run with CI-downloaded binaries (`/tmp/agentA-1/rel/formal-ai` 0.356.12).

## Common cause: why PR CI never catches these

Every failing job is gated to `main` push only:

| Job | Gate |
|---|---|
| corpus-build / corpus-shard / corpus-gate | `.github/workflows/benchmark-corpus-gate.yml` `if:` excludes `pull_request`, requires `refs/heads/main`; on PRs only `pr-check-profile` runs, so the PR check is green **vacuously** |
| server-parity | `.github/workflows/layered-ci.yml:775-796` (main only) and the CI/CD equivalent in `release.yml` |
| native-response-observations | `.github/workflows/release.yml:888-894` (main + build-artifacts success) |
| auto verify-package | `release.yml:715-724`, `release-staged.yml:208` |
| Deploy Pages | `release.yml:972-977` |

`data/benchmarks/fast-pull-request-javascript-tests.json` (scope: "Full native and product build qualification runs only on main") contains no test that exercises the real repo module graph for capture-case binding, no server-parity smoke over real binaries, and the Pages test only mocks the API. These are **false negatives** on PRs: the PR check reports success because it never ran.

---

## 1. Benchmark Corpus Gate (run 45549993299): shards 1, 4, 6 and the aggregate fail

**Symptom.** Shards 1, 4 and 6 receive the runner "shutdown signal" (exit 143) 6-7 min into the test step. For example, shard 6's step started at 05:03:46, logged `corpus shard 6/6: 233 of 1399 prompts`, and was `Terminated` at 05:10:08. Shards 2 and 3 were cancelled at the 25 min `timeout-minutes` and have no uploaded logs (BlobNotFound). Shard 5 passed in 684s, which hit the 70% budget warning. `corpus-gate` fails through `scripts/check-shard-results.sh` because the shards are non-success.

**History on main.**
- 45001542967 (6a49eff07): last green; every shard took 10-15 min.
- 45020745809 (755e8781a): shard 1 exceeded its 900s `TEST_BUDGET_SECONDS`.
- 45038778669 through 45463430386: `corpus-build` failed (exit 101), so no shard ran, which hid the slowdown.
- 45490076540 (1c109025c): shard 2 got the shutdown signal; the other shards were cancelled at 25 min.
- 45549993299 (eee485204): the current failure.

**Root cause, partly identified.**
- A runtime regression pushed the shards over the 900s budget and 25 min timeout. The window is 6a49eff07..1c109025c: PRs #1626, #1614, #1623, #1625, #1622, #1627, #1633, #1581, #1635.
- Separately, the runner VM was killed mid-test, which is CI-only.
- A local run of shard 1 with the CI release and gate binaries went past 970s with RSS steady at 120-185 MB. There was no OOM and no hang, so the VM kill is **not reproduced** and remains unidentified. Disk or memory pressure on the runner is the leading suspect.
- Note: a local `lean` on PATH causes spurious slow prompts 840-856, which do not occur in CI.

**Observability gap.** `rust/tests/integration/issue_1138_no_silent_unknown.rs:131-166` loops `FormalAiEngine.answer(prompt)` without logging each prompt. A killed shard therefore cannot show which prompt it was on or how long it had taken. The loop was introduced in e8d169099 (shard split) and 70df98cf6.

**Classification.** Real regression (runtime, plus an infra kill). PR CI gives a false negative because the PR result is vacuous.

**Minimal fix.**
```diff
--- a/rust/tests/integration/issue_1138_no_silent_unknown.rs
+++ b/rust/tests/integration/issue_1138_no_silent_unknown.rs
@@ (inside the per-prompt loop)
+        let started = std::time::Instant::now();
         let answer = engine.answer(prompt);
+        let elapsed = started.elapsed();
+        if elapsed.as_secs_f64() > 2.0 || index % 50 == 0 {
+            eprintln!("corpus shard {shard}: prompt {index} {:.2}s {prompt:?}", elapsed.as_secs_f64());
+        }
```
```diff
--- a/.github/workflows/benchmark-corpus-gate.yml
+++ b/.github/workflows/benchmark-corpus-gate.yml
@@ corpus-shard: before the test step
+      - name: Resource monitor
+        run: (while true; do date -u +%T; free -m | sed -n 2p; df -h / | tail -1; sleep 30; done) > "$RUNNER_TEMP/resources.log" 2>&1 &
@@ after the test step
+      - if: always()
+        run: cat "$RUNNER_TEMP/resources.log" || true
```
Also either run one rotating shard on `pull_request` that touch `rust/src/**` or `data/seed/**`, or make the PR job state explicitly that it is a skip rather than a success. Then bisect the window above using the per-prompt timings.

---

## 2. Server parity (CI/CD 45549993600, Layered CI 45549993081)

**Reproduced locally.** `timeout 600 node scripts/check-server-parity.mjs --rust-binary /tmp/agentA-1/rel/formal-ai` gives 134 identical cases, 21 protocol divergences and 6-7 content divergences, ending with `::error::7 content divergence(s) above the ceiling 0`. The ceiling is the `answer-divergence-ceiling`, read at `scripts/check-server-parity.mjs:97` and enforced at :578-587.

**History.**
- Last green on main: 44255971948 (19235c49a, 2026-11-18).
- First red: 44321788230 (30ff2d8a8, merge of PR #1585, branch issue-1584). Main has been red since. Because the job is main-only, no PR saw it.

### Divergence classes

**(A) `output_projection` key order. Affects 20 protocol cases.**
- `sameSkeleton` (`scripts/check-server-parity.mjs:327-368`) compares keys in order. That is deliberate, because order is part of protocol identity.
- Rust: `rust/src/protocol/output.rs:34` declares `pub output_projection: Option<Value>`. `serde_json` is built without `preserve_order` (`rust/Cargo.toml:72`), so the `Value` map is a BTreeMap and keys come out alphabetical: `content, document, links_notation, notice, report, text`. Nested `document` is `language, parts, source`; nested `report` is `compliant, lexical_density, output_standard, unchecked, violations`.
- JS: `js/server/cnl-output.mjs:41-59` `projectServerAnswer` returns `rendered` in insertion order: `text, notice, content, document, links_notation, report`.
- Introduced by 409d72d08 ("feat: add experimental controlled output policy and settings"), with a6862bf74 and fb73b49a2 (all PR #1585), plus 00d426c1c and 556a9b6a6.
- Fix: make JS emit the canonical (sorted) key order that Rust's `Value` produces.
```diff
--- a/js/server/cnl-output.mjs
+++ b/js/server/cnl-output.mjs
@@
+// Rust stores output_projection as serde_json::Value (BTreeMap, no preserve_order),
+// so its keys serialise sorted; emit the same canonical order for protocol parity.
+function canonicalKeys(value) {
+  if (Array.isArray(value)) return value.map(canonicalKeys);
+  if (value === null || typeof value !== 'object') return value;
+  return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonicalKeys(value[key])]));
+}
+
 /** The protocol may preserve a body already composed under request constraints. */
@@
-  return {...symbolic, answer: rendered.content, output_projection: rendered};
+  return {...symbolic, answer: rendered.content, output_projection: canonicalKeys(rendered)};
```
Also apply this anywhere else JS attaches `output_projection` to stream or final chunks; compare with `rust/src/server/chat_stream.rs:109-110`.

**(B) `report.violations`. Affects 2 cases: agent_tool_choice_without_tools and agent_contextual_clarification.** Rust returns a non-empty list of strings; JS returns `[]`. The controlled-output validator differs between runtimes. The cause is the same feature (409d72d08 / PR #1585). The fix belongs in JS `validateOutput`, which should apply the same rules as the Rust validator in `rust/src/cnl_output/`.

**(C) `thinking_steps[].source_origin`. Affects 4 cases: chat_memory_recall, chat_memory_recall_missing, chat_memory_link_count and responses_memory_recall_amended.**
- Rust always emits the key (`rust/src/thinking.rs:56`, populated from `rust/src/event_log.rs`).
- JS emits it only when the event carries it (`js/server/thinking.mjs:272`).
- The JS memory path, `js/server/memory-answer.mjs:64` `thinkingStepsFromEvents(log.events, body)`, builds events without `source_origin`.
- Introduced by a8643b4a7 ("Record actual validation origins and replay conversation contracts in native CI", 2026-11-19). It is followed by d5b64675e.
- Fix: stamp `source_origin` on the memory-answer events in JS the same way `rust/src/event_log.rs` does for the memory path.

**(D) Content divergences.**
- `$.evidence_links.27` in responses_basic, responses_openai_path and responses_input_items: `method_registry_d4cae59cc2bb6e3a` (Rust) vs `method_registry_e3c01a5eb8ebbb63` (JS). An earlier run had a different pair, 5b7a1dab25e65337 vs 2cc42deb6d8158be. The id is a content hash of the method registry, and the hashed registry inputs differ between runtimes, so every registry change moves both hashes independently. The fix is to hash the same canonical serialisation, or the same registry source, on both sides. I did not trace the hash function byte by byte.
- `sse content differs` in chat_stream, chat_stream_usage and responses_stream. These first appeared in the first red run (30ff2d8a8), consistent with the final chunk carrying `output_projection` (chat_stream.rs:109-110) in different key order or content.
- `bundle: body text differs` appeared only in my local run. It probably comes from the uncommitted working-tree JS changes, not from CI.

**Classification.** Real regressions: cross-runtime drift introduced by PR #1585 and a8643b4a7. The PR result is a false negative because the job is main-only. The parity checker is correct and should not be loosened.

---

## 3. native-response-observations: "no unique registration ... 2 !== 1"

**Root cause.**
- `scripts/lib/native-test-module-graph.mjs:311-320` `bindOriginalSourceCase` asserts `matches.length === 1`.
- 634eb760a ("refactor: resolve reported source size warnings without dropping coverage", merged via e3046ab36) added `#[path = "specification/behavior_rules.rs"]` and `#[path = "specification/chat_surface.rs"]` to `rust/tests/unit/reasoning_contracts.rs:5-9`. That file also includes `reasoning_paths_procedures.rs` (7b10f54c1, 54986d9a4) and `web_requests` (33aca392f).
- Those sources are now compiled into both the `unit` and `reasoning_contracts` targets (`rust/Cargo.toml:187-188`). As a result, five of the 28 cases in `data/meta/native-response-capture-cases.json` (from 269eb5af1, PR #1188) bind twice: behavior_rules_count_followup_answers_reported_russian_prompt, three procedural_elaboration_followup_* cases, and research_comparison_table_followup_uses_prior_search_topics.
- Reproduced locally in about 5s with the real module graph.

**Classification.** Real regression (a latent ambiguity). The PR result is a false negative: the job is main-only, and `rust/tests/web/native-test-module-graph.test.mjs` used fixtures only.

**Fix.** Already on this branch at HEAD as **dec7e0d17**, which accepts `@<target>` qualifiers. The five ids now name `@unit`, and the graph test binds every committed case against the live graph. My proposed equivalent was a `preferredTarget` argument passed from `scripts/capture-native-responses.mjs:24`. Remaining step: make sure `rust/tests/web/native-test-module-graph.test.mjs` is listed in `data/benchmarks/fast-pull-request-javascript-tests.json`. Before this work it was not; only `native-producer-source-pin.test.mjs` was.

---

## 4. auto verify-package: AddrInUse and missing server capture

**Root cause.**
- faa55b564 ("feat(issue-1632): run Claude Code in auto mode by default", PR #1636) set permission-mode `auto` in `data/seed/client-integrations.lino` (around lines 477-478 and 509).
- `experiments/issue_1609_client_reports/stream-report.mjs:41-43` still asserts `event.permissionMode === 'default'`, so it fails with `'auto' !== 'default'` (job log lines 4031-4048).
- Its `finally` block SIGTERMs `formal-ai with`. That process has no signal handler, so `ServerGuard::drop` (`rust/src/client_integrations/server.rs:19-23`) never runs and an orphaned `serve` keeps port 19409.
- Later clients either reuse the orphan (`maybe_start_server`, server.rs:31-33) or hit AddrInUse (e.g. `gemini-report.sh:28`). With no capture, `fake-gh.sh:26-29` fails with "The merged context is missing a nonempty server capture".
- The earlier failure in run 45490077033 had a different cause: the 12-minute step timeout at `release-staged.yml:260` expired.

**Classification.** A real regression (stale assertion) that cascades into a false positive on the other clients. The PR result is a false negative because the job is main-only.

**Fix.**
```diff
--- a/experiments/issue_1609_client_reports/stream-report.mjs
+++ b/experiments/issue_1609_client_reports/stream-report.mjs
-  assert.equal(event.permissionMode, 'default');
+  assert.equal(event.permissionMode, 'auto');
```
Add a port cleanup between clients in `verify-release.sh` (around line 115): `fuser -k 19409/tcp 19410/tcp 2>/dev/null || true`. The durable fix is to forward SIGTERM/SIGINT in `formal-ai with` so that `ServerGuard` drops. Better still, derive the expected mode from the seed rather than hard-coding it.

---

## 5. Deploy Pages: POST /pages/deployments returns 404

**Root cause.**
- f8718c44e ("Bind Pages deployment retries to the exact uploaded artifact", PR #1581) replaced `actions/deploy-pages@v5` with the local action `./.formal-ai-pages-protocol/.github/actions/deploy-pages` (`release.yml:996`).
- Its `deploy.mjs:123-134` and `:166-171` send `pages_build_version = sha256(repo, artifact_id, sha)` where the official action sends the commit SHA. The API answers 404. This attribution is inferred: it is the only payload difference.
- Repository Pages settings are fine (`build_type: workflow`).
- `rust/tests/web/pages-deployment.test.mjs` (:24, :79, :315, :322) mocks the API and even asserts a 64-hex version, so it locks in the bug.

**Classification.** Real regression. The PR result is a false negative: the job is main-only and the test is fully mocked.

**Fix.**
```diff
--- a/.github/actions/deploy-pages/deploy.mjs
+++ b/.github/actions/deploy-pages/deploy.mjs
-    pages_build_version: bindingDigest,
+    pages_build_version: sourceSha,
```
Keep the artifact-id binding in the retry logic, not in the API field, and update the test assertions from 64-hex to the 40-hex commit SHA. Fallback: revert to `actions/deploy-pages@v5`, which was last good in run 45275490842.
