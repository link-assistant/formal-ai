# Issue #1484: bind the selected Formal AI and Hive Mind sources to the runtime that serves the draft

Issue: <https://github.com/link-assistant/formal-ai/issues/1484>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1494>

Hive Mind implementation: <https://github.com/link-assistant/hive-mind/issues/3909>,
pull request <https://github.com/link-assistant/hive-mind/pull/3912>

## 1. Collected data

| Path | Content |
| --- | --- |
| `raw-data/issue-1484.json` | The issue as of 2026-11-15, including its comments |
| `raw-data/hive-mind-pr-3912.json` | State of the Hive Mind pull request when this change was made |

## 2. Timeline and root cause

- **2026-10-30.** The Hive Mind draft [run 41081977243](https://github.com/link-assistant/hive-mind/actions/runs/41081977243/job/124153176671) reported wrapper and serving backend 0.355.15. Formal AI v0.355.23 had been published eleven hours earlier.
- **Root cause.** At Hive Mind `279ac056`, three things combined:
  - the `Dockerfile` baked `FORMAL_AI_VERSION=0.355.15`;
  - `formal-ai-draft.mjs` mounted a different binary only when one was explicitly supplied;
  - `formal-ai-draft-image.lib.mjs` fell back from the exact image to `latest`.

  The serving version was printed, but nothing tied it to a selected source. A same-version binary built from another commit, or a stale image, could serve the task and still be counted as a current-source draft.
- **Formal AI side, before this change.** `hive-mind-draft.yml` already built the exact `main` commit and mounted it through `FORMAL_AI_DRAFT_BINARY`. However:
  - it kept no single attempt identity across the `build` and `draft` jobs;
  - it did not check that the downloaded binary was the one it built before running the attempt;
  - it never told Hive Mind which source it had selected (`FORMAL_AI_DRAFT_SOURCE_SHA` was unset);
  - it never compared Hive Mind's serving evidence with its own selection.

  A missing artifact also failed the job with no typed outcome.
- **Since 2026-11-02 (hive-mind PR #3912, open).** Hive Mind defines `hive-mind/formal-ai-draft-attempt-identity/v1` and writes three files into `FORMAL_AI_DRAFT_LOG_DIR`:
  - `attempt-identity.json`;
  - `attempt-record.json`: the binary digest, the image pinned by registry digest, and the serving process' `/proc/<pid>/exe` digest;
  - `attempt-outcome.json`, for `deferred-infrastructure` and `identity-mismatch`.

  Hive's hosted run 41547700367 deferred because image 2.39.11 predates that source. This is the contract the Formal AI side now binds to.

## 3. The Formal AI contract (this pull request)

`scripts/lib/hive-draft-attempt-identity.mjs` defines the contract, `scripts/hive-draft-attempt.mjs` runs it, and `.github/workflows/hive-mind-draft.yml` wires it into the workflow.

1. **Admission, `build` job (`--select`).**
   - Admission happens once, after the exact binary is built.
   - The identity records:
     - the Formal AI source SHA, version, binary sha256 and size;
     - the Hive Mind source SHA, version and the exact `konard/hive-mind:<version>` image;
     - the issue, the run, and the selection time.
   - The id is the sha256 of the canonical selection, truncated to 32 hex characters. The time and `run_attempt` are excluded, so a retry keeps the id and a new source gives a new one.
   - Admission is rejected with `identity-mismatch` when:
     - either checkout drifted from the resolved SHA;
     - the image is `latest` or not tagged with the Hive Mind version;
     - the version is not semver.
   - A missing binary digest is `deferred-infrastructure`.
2. **Custody, `draft` job before any task effect (`--verify-custody`).**
   - Checks the identity from the job outputs against:
     - the downloaded binary's sha256 and `--version`;
     - the Hive Mind checkout;
     - the issue.
   - A missing artifact (the download step is `continue-on-error`) or a missing identity is `deferred-infrastructure`. Any other difference is `identity-mismatch`.
   - Version equality alone is not enough: a same-version binary with another digest is rejected.
3. **Handoff.** The attempt step now sets `FORMAL_AI_DRAFT_SOURCE_SHA` (the change Hive Mind's comment on the issue asks for) and `FORMAL_AI_DRAFT_MODE=live-current`. Hive Mind can then bind the caller binary to its source instead of to its digest only.
4. **Serving verification, after the attempt (`--verify-hive`).**
   - Compares Hive Mind's `attempt-identity.json` and `attempt-record.json` with the admitted identity:
     - mode, repository and issue;
     - both source SHAs and the image;
     - the caller binary digest and the mounted binary digest;
     - the serving executable digest and the serving Formal AI version;
     - the Hive Mind version;
     - that the image is bound by a registry digest.
   - Hive Mind's typed outcome is kept as is, never upgraded.
   - When the Hive Mind checkout writes no such files (every release up to 2.40.4), the receipt is `serving-identity-unverified`: the run does not fail, but it earns no current-source credit.
   - A mismatch fails the job and blocks the exact-PR evidence step.
5. **Receipts and dashboards.**
   - Every receipt carries `mode`: `live-current`, `published-release` or `historical-replay`.
   - `currentSourceCredit` is true only for a verified `live-current` receipt whose serving identity was checked.
   - The sealed #1364 ledger rows (`taskMode: historical-1364`) are classified as historical replay, so they can never count as current-source success.
   - The step summary shows which kind of attempt ran, and the failure comment names the typed outcome with zero capability credit.

## 4. Verification

`rust/tests/web/hive-draft-attempt-identity.test.mjs` runs in the fast JavaScript suite. It covers:

- one identity across retries and a new identity for a new source;
- checkout drift, `latest` and inexact images, and missing binaries;
- a stale binary with the same version, a stale Hive Mind checkout, and the wrong issue;
- Hive Mind serving another executable, the image-baked backend, a tag-only image, another Hive Mind version, and a Hive Mind without the contract;
- typed Hive Mind outcomes being preserved;
- historical and release receipts never earning credit;
- the CLI end to end, writing the identity, outcome and receipt files;
- the workflow order: admit, then download, then custody, then the attempt, then serving verification, then the exact-PR step.

## 5. What remains blocked

The issue's ownership section says completion is blocked by link-assistant/hive-mind#3909. On 2026-11-15:

- hive-mind PR #3912 is open and unmerged (head `8d73500f`);
- Hive Mind `main` and the latest release v2.40.4 have no `:latest` fallback, but they do not read `FORMAL_AI_DRAFT_SOURCE_SHA` and do not write the attempt identity or record.

These acceptance items therefore stay open until that pull request is merged, released, and one hosted issue-created attempt runs:

- the hosted end-to-end attempt (advance Formal AI, open an issue, and the next live draft runs the new source with a `verified` receipt);
- the serving-identity half of "both repositories demonstrate the same contract".

Until then, the Formal AI workflow records `serving-identity-unverified` with zero credit and never reports success for the current source.
