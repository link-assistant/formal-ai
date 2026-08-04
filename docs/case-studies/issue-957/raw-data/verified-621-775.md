# Verified requirements R621–R775 (chunk B)

Source: `req-chunk-621-775.ndjson` (175 requirements) verified against repo HEAD
(main, includes PR #926). Companion output: `verified-621-775.ndjson`.

## Verdict counts

| Verdict | Count |
|---|---|
| DELIVERED | 135 |
| PARTIAL | 22 |
| NOT-DELIVERED | 17 |
| OBSOLETE | 0 |
| UNVERIFIABLE-LOCALLY | 1 |
| **Total** | **175** |

`needs_issue=true`: 3 (R643-6, R658-1, R771-2).

## NOT-DELIVERED (and tracking status) — 17

All 17 are tracked by an open issue/PR (no untracked NOT-DELIVERED in this chunk):

| id | tracked_by | requirement (short) |
|---|---|---|
| R643-1..6, R643-8, R643-10, R643-11 (9 items) | #643 | Multi-framework/multi-skin UI polish PR — confirmed still open in raw issue snapshot (`state:"open"`), no MUI/glass code in `src/web/app/theme.js` (Chakra-only) |
| R665-1 | #665 | E46 installable offline PWA |
| R666-1 | #666 | E47 VS Code Marketplace/Open VSX publish |
| R667-1 | #667 | E48/R383 interactive step-debugging view |
| R670-1 | #670 | E51/F5 WebVM/Pyodide browser multi-language experiment |
| R700-1 | #700 | E58 universal measuring-unit support (si-units) |
| R705-1 | #705 | E63 anticipatory dreaming / Markov transition records |
| R720-1 | #720 | 5 real-user failure reports from external user xierongchuan, never triaged |
| R644-1 | #644 | PR #644 "experimental formalization model fallback" — open/undecided, no surviving remote branch |

`needs_issue=true` (no existing tracker covers the specific gap):
- **R643-6** — "Extract Chakra+liquid-glass integration into a separate reusable repo once #643 lands" (a promised follow-up inside PR #643's own thread, not itself covered by #643)
- **R658-1** — Finish E39 (#658): absorb remaining ~27.7k lines of JS worker logic into the Rust→WASM worker (target ≤3,000 UI-glue lines) — verified: `src/web/worker/formal_ai_worker_*.js` files still run 600–1,500+ lines each, summing far past budget
- **R771-2(d)** — Add an automated redaction skill/handler for issue-report publishing (personal/sensitive-data reasoning) — verified: `grep -rl redact src/` returns nothing; only manual "redact it" advice exists in docs

## UNVERIFIABLE-LOCALLY

- **R651-6** — "All created issues must be sub-issues of #651 (merged into a single PR)." Handcheck: query the GitHub sub-issues API for #651 live to confirm the parent/child linkage was actually set (this is GitHub relationship metadata, not something that leaves a trace in the repo tree).

## Focus-point findings

**#702/E60 world models — precise wiring check.**
- `WorldModel::new()` **is** called outside tests: `src/world_model_dialog.rs:306`, reached via `src/solver_dispatch.rs:221 "world_state" => try_world_state(...)` in the production dispatch table, gated by `SolverConfig.world_model_mode` (`solver.rs:207`). R702-1 = DELIVERED, confirmed by direct grep of call sites (20 test call sites plus the one production site).
- **Fabrication check, verified precisely as asked:** `is_placeholder_source()` in `src/fact_checking.rs:588-594` denylists the literal strings `"fabricated"`, `"example.org"`, `"example.com"`, `"example.net"` — this alone would be a weak, gameable fix (just avoid those domain strings). But the deeper structural fix is real: grep across `src/` shows the **only two call sites** that construct a `"source:http"` event (`src/source_fetch.rs:150` inside `SourceCapture::record()`, and `src/probability.rs:442` which calls `source.trace_payload()` on a `SourceCapture`) both require going through the curl-backed `SourceCapture::fetch()` in `source_fetch.rs`. No code path appends `"source:http"` from a free-text/fabricated URL. **Conclusion: the fabrication problem is structurally fixed, not merely domain-renamed** — R702-2 = DELIVERED.
- **proof_engine relative-meta-logic:** `src/proof_engine/mod.rs:130,179` — `deep_relative_meta_logic_step()` only generates **narrated step text** (a human-readable string describing what a relative-meta-logic tactic would do, in 4 languages) and is never a call into an actual `relative_meta_logic` module/crate. This nuance is captured explicitly in R702-2's evidence field as a caveat worth confirming with konard, correctly distinguishing "delivered the blocker-clearing fix" from "proof engine doesn't yet really invoke relative-meta-logic."

**Intent-routing family (#624/#627/#680/#681/#712/#745) vs. #842 ladder.**
- `experiments/issue_840_task_ladder/results.json` confirmed: `"total":24,"passed":24,"failed":0"`, broken out L1(3/3)/L2(6/6)/L3(7/7)/L4(8/8) — the 24/24 claim is real and file-verified, not just asserted.
- `.github/workflows/task-ladder.yml` exists (CI enforcement present).
- Server honoring harness-advertised tool schemas — confirmed: `src/agentic_coding/intent_router.rs` explicitly gates routing on "the CLI actually advertised a matching tool," and `driver.rs:105` records `"tools_advertised": DRIVER_TOOLS` in the action log, i.e. routing decisions are conditioned on the harness's own advertised schema, not assumed.
- Remaining gap: R680-2 (write-effect rungs) is correctly PARTIAL — web_search/web_fetch and write/edit *emit* calls and are lifecycle-tested, but full end-to-end write-EFFECT ladder rungs are still what open epic #916 (E69, #848 ladder) tracks as unfinished.

**Six v0.297.1 closures (#754/#757/#759/#760/#761/#763).** All six verified DELIVERED with concrete artifacts: Cursor MCP wiring + `tests/integration/issue_754_cursor.rs`; `tests/issue_756.rs` shared-memory store (adjacent); `src/client_integrations/completion.rs` + `tests/issue_757_session_files.rs` (file exists — confirmed present at `tests/issue_757_session_files.rs`, consistent with the known macOS `/var` canonicalization flake noted in the task brief; counted as artifact-with-a-bug, not NOT-DELIVERED); agentic-tool-capabilities seed + `tests/integration/issue_758_capability_routing.rs`; t3code matrix leg + `tests/issue_760.rs`; docs pages + `tests/issue_761_docs.rs`; opencode-vscode/opencode-desktop matrix legs + `tests/issue_762.rs`/`tests/issue_763.rs`.

**#771 privacy/redaction.** Split verdict, both halves checked directly:
- R771-1 (extract-plus-source answer quality) = PARTIAL — `tests/unit/issue_771.rs` pins sentence-extraction and per-language coverage; RRF multi-engine fusion is real; but Google-AI-grade answer+source quality is still failing live per open #827/#872/#800/#821.
- R771-2 (redaction/report-confirmation) = PARTIAL, needs_issue=true for the specifically-missing piece — report-format rebuild, gist upload (secret-by-default), and pre-file confirmation are all delivered and pinned; but no dedicated redaction skill/handler exists (`grep -rl redact src/` → empty), only manual "redact it" guidance in docs.

**#716 Docker sandboxing.** R716-1 (route CLI commands to real bash/shell tools) = DELIVERED (`tests/integration/issue_716_agentic_execution.rs`, `issue_749_shell_routing.rs`). R716-2 (one-time temp Docker containers for safety) = PARTIAL, verified precisely: `desktop/lib/tool-router.cjs` comments confirm "shell commands run on the host process **by default**... code-exec/eval-js tools run inside a `konard/box-dind` container... with a graceful fallback when Docker is unavailable," and shell only *opts into* `isolation:"docker"`. No `docker` references found under `src/telegram*.rs` — the Telegram surface is unverified, exactly as flagged. This is the same underlying gap as chunk A's R331-5 (server/Telegram execution sandboxing) — cross-chunk corroboration.

**PR #643 completion-gate/traceability table.** Verified via the raw GitHub snapshot (`raw/issues_prs.ndjson`, `number:643`) that the PR's true state is **`"state":"open","merged":null`** as of the audit snapshot — confirming the gate genuinely never finalized (not merged-then-reverted). `src/web/app/theme.js` in the current tree contains only the Chakra skin bridge (comment: "Issue #550: Chakra UI theme bridge"), no MUI or liquid-glass code, matching the NOT-DELIVERED verdicts for the 9 R643-* acceptance-criteria items. R643-7 (the gate itself holding, i.e. not finalizing without meeting bar) and R643-9 (gate honored but deliverables/table incomplete) are correctly scored DELIVERED/PARTIAL respectively — the discipline of *not* merging a half-done PR is itself evidence the completion gate is doing its job, even while the underlying feature isn't done.

## Surprises

- The example.org/epoch-0 fabrication question (R702-2) turned out to have a more reassuring answer than the "just banned one domain name" framing my task raised as a risk: the actual fix is structural (only two call sites in the whole `src/` tree can construct `source:http`, both gated behind a real curl fetch), with the domain denylist as a secondary/defense-in-depth check, not the whole fix.
- PR #643 is absent from `open-issues.txt`'s curated 61-item list even though the raw GitHub snapshot shows it as genuinely open — the curated list appears to be issues-only (PRs are tracked separately), so anyone relying solely on `open-issues.txt` for gap→tracker mapping would incorrectly conclude #643 has no tracker. Cross-checked against `raw/issues_prs.ndjson` directly to confirm.
- The chunk's working file (`work621/verdicts.py`) was already 100% complete for all 175 IDs with no gaps (unlike chunk A, which was short 6 IDs) — spot-checks across every named focus point held up under direct grep/file verification, so this chunk required confirmation rather than backfill.
