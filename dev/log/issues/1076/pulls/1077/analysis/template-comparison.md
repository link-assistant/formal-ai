# Template comparison — file tree against all four `link-foundation` templates

Issue requirement R2: *"Use all the best practices from CI/CD templates (check
full file tree to compare for all GitHub workflow and CI/CD scripts file)"*, and
R4: *"We should compare all files, so we don't have more CI/CD errors in the
future and reuse all the best practices from these templates."*

This is the comparison. It is deliberately a *file-tree* comparison and not a
spot check: every file under `.github/` or `scripts/` in each template was
matched against this repository, and the outcome of every match is recorded
below — including the matches that produced no change, because "we compared it
and it was fine" is a result and an unexamined file is not.

Inputs, all committed next to this document so the comparison is reproducible:

* `references/templates/{rust,js,python,php}-template/` — complete immutable
  snapshots of all four template trees.
* `template-diffs/file-inventory.txt` — the shared / template-only partition,
  with counts.
* `template-diffs/*.diff` — 30 per-file diffs.
* `template-diffs/diffstat-summary.txt` — every differing file with its size.

---

## 1. Coverage of the comparison

| Template | Template CI files | Shared with this repo | Template-only |
| --- | ---: | ---: | ---: |
| rust | 37 | 31 | 6 |
| js | 50 | 10 | 40 |
| python | 22 | 8 | 14 |
| php | 36 | 4 | 32 |

The js/python/php numbers are dominated by their language toolchains
(`scripts/*.mjs`, `scripts/*.py`, `scripts/src/*.php`) — an npm publish script
has no counterpart in a Rust repository and is not a gap. The comparison that
carries information is against the **rust** template (31 of 37 files shared) and,
across all four, the *language-independent* files: the workflows, the composite
actions, and the shell scripts.

## 2. Language-independent files: the comparison that matters

These exist in more than one template, in more than one language, and therefore
represent a deliberate practice rather than a language accident.

| File | rust | js | python | php | Here | Outcome |
| --- | :-: | :-: | :-: | :-: | :-: | --- |
| `.github/workflows/workflows.yml` | ✓ | ✓ | ✓ | ✓ | **was absent** | **Adopted — D10.** The only file present in *all four* templates and missing here. See §3. |
| `.github/zizmor.yml` | ✓ | ✓ | ✓ | ✓ | **was absent** | **Adopted — D10.** Its companion config. |
| `.github/workflows/links.yml` | ✓ | ✓ | ✓ | ✓ | ✓ | Compared; this repository's is a superset (retry/backoff tuning from issue #1021). No change. |
| `.github/workflows/release.yml` | ✓ | ✓ | ✓ | ✓ | ✓ | 1,671–2,130 differing lines per template — this repository's release pipeline has diverged far past the template. Compared for *practices*, not for text; findings in §4. |
| `.github/workflows/security.yml` | ✓ | ✓ | ✓ | — | ✓ | Compared; equivalent. No change. |
| `scripts/check-pipeline-status.sh` | ✓ | ✓ | ✓ | — | ✓ | Compared; equivalent. No change. |
| `scripts/run-with-budget-warning.sh` | ✓ | ✓ | ✓ | — | ✓ | Compared line by line and **executed**; two upstream defects found. See §5. |
| `scripts/simulate-fresh-merge.sh` | ✓ | ✓ | ✓ | — | ✓ | Compared; equivalent. No change. |
| `.github/actions/setup-buildx-resilient/action.yml` | ✓ | ✓ | — | — | ✓ | Compared; equivalent. No change. |
| `.github/actionlint.yaml` | ✓ | — | — | — | ✓ | Compared; +16 lines here, all additive (extra `self-hosted` labels and config-variable names). No change. |

## 3. The one structural gap: workflow security auditing (D10)

All four templates ship `.github/workflows/workflows.yml`. All four run the same
two tools:

* `docker://rhysd/actionlint:1.7.7` — the *container* form, which matters:
  actionlint delegates `run:` bodies to ShellCheck and **silently skips shell
  checking when ShellCheck is not on PATH**. The container bundles it. This
  repository was invoking the bare binary, so its shell linting was a false
  negative — and the rust template's own comment names the same trap, citing
  "the SC2016 defect in release.yml (issue #141)".
* `zizmorcore/zizmor-action@v0.6.2` with `advanced-security: false`,
  `annotations: true`, `config: .github/zizmor.yml`, `min-confidence: medium`.

This repository ran `actionlint` only, and ran it as a bare binary. It now
matches the templates: a dedicated `workflows.yml`, actionlint via container (at
1.7.12), zizmor with `.github/zizmor.yml`. The suppressions in that config, and
the reasoning for each, are in `best-practices-audit.md`; the ones deliberately
left unsuppressed are in `requirements.md` under "not in scope".

## 4. Container cache scoping — the same defect in three templates

Comparing `cache-to:` across all four release workflows:

| Template | Site | Spec |
| --- | --- | --- |
| rust | `release.yml:275` | `cache-to: type=gha,mode=max` |
| rust | `release.yml:948` | `cache-to: type=gha,mode=max` |
| js | `release.yml:396` | `cache-to: type=gha,mode=max` |
| python | `release.yml:490` | `cache-to: type=gha,mode=max` |
| python | `release.yml:833` | `cache-to: type=gha,mode=max,scope=${{ matrix.platform }}` |
| php | — | no container build |

Four of the five sites omit `scope=`. buildx documents the default as `buildkit`
and warns: *"If you build multiple images, each build will overwrite the cache
of the previous, leaving only the final cache."* Every unscoped build in a
repository therefore shares one cache object and evicts the others.

The python template's line 833 is the same fix, already present one job away
from its own line 490 — which is what makes this a genuine oversight rather than
a deliberate choice, and what makes the fix uncontroversial to propose.

This repository had the defect at **five** sites; all five now carry
`scope=docker-image`, with `tests/unit/ci-cd/issue_1076.rs` failing if an
unscoped exporter reappears. Reported upstream:
`upstream-reports/templates-unscoped-gha-cache.md`.

**`mode=max` is correct and was kept at every site.** An earlier draft of this
work proposed `mode=min` to reduce the 4.91 GB the layer blobs occupy; that is
wrong for a multi-stage build and would have reintroduced issue #977. The
upstream report says so explicitly so nobody acts on the wrong half.

## 5. `run-with-budget-warning.sh` — three implementations, three behaviours

The three implementations were not merely diffed, they were **run**, with an
identical harness: 2-second budget, 3-second grace, and a child that installs a
SIGTERM handler and ignores it. Scripts and outputs in
`experiments/issue-1076/`.

| Implementation | Deadline clock | Grace-loop clock | Escalation target | poll=1 | poll=0.5 |
| --- | --- | --- | --- | --- | --- |
| rust template | `elapsed=$(( elapsed + POLL_SECONDS ))` (L81) — **counts iterations** | `waited=$(( waited + 1 ))` (L89) — correct for integer poll | process group of the command — correct | exit 124, 0 survivors | **exit 0 after the full command**: `$(( elapsed + 0.5 ))` is a bash syntax error and the budget never fires |
| js template | `SECONDS=0` … `[ "${SECONDS}" -ge … ]` (L107/L116) — correct | `waited=$((waited + poll_seconds))` (L93) — same arithmetic defect | `command_pid` is the **wrapper subshell** (L70), which exits before the child, so `command_is_running` (L83) goes false and SIGKILL is never sent | exit 124, **1 survivor** | exit 143, **1 survivor** |
| python template | `started=$SECONDS`; `elapsed=$((SECONDS - started))` (L45/L74) — correct | `grace_deadline=$((SECONDS + grace_seconds))` (L91) — correct | process group — correct | exit 124, 0 survivors | exit 124, 0 survivors |
| **this repository** | `started=$SECONDS`; `elapsed=$((SECONDS - started))` (L44/L95) — correct | `grace_deadline=$((SECONDS + grace_seconds))` (L114) — correct | process group — correct | exit 124, 0 survivors | exit 124, 0 survivors |

Two conclusions, and they point in opposite directions — which is the reason to
compare rather than to copy:

1. **This repository is immune to both defects.** Its script descends from the
   python lineage, which reads `$SECONDS`. Nothing was changed here as a result
   of §5; the value of the comparison was the two upstream reports.
2. **The templates are not.** Both defects were reproduced, minimised, and filed
   with patches: `upstream-reports/rust-template-budget-poll.md` and
   `upstream-reports/js-template-termination-path.md`.

The js escalation defect is the more serious of the two: a test worker that
ignores SIGTERM survives the step, keeps holding the runner, and the wrapper
still reports the documented exit code — a false negative inside the very tool
built to prevent false negatives.

## 6. Template-only files: gap or different mechanism?

Six rust-template files have no same-named counterpart here. A missing *name* is
not a missing *practice*, so each was resolved to what does the job here:

| Template file | Purpose | Here |
| --- | --- | --- |
| `scripts/check-crate-size.rs` | fail if the packaged crate exceeds crates.io limits | `scripts/check-crate-package-size.rs` — same gate, renamed |
| `scripts/smoke-test-published-crate.rs` | install the published crate and run it | `scripts/smoke-test-published-crate.sh` — same gate, shell instead of rust-script |
| `scripts/check-cargo-lock.rs` | *"Binary crates should commit `Cargo.lock` … especially important for workflows that use cache keys based on `hashFiles('**/Cargo.lock')`: when no lockfile is committed, that expression falls back to the same empty hash across runs"* | **Same invariant, enforced as a test instead of a CI script**: `tests/unit/ci-cd/issue_1017.rs::cargo_lock_is_committed_so_cache_keys_stay_meaningful` runs `git ls-files --error-unmatch Cargo.lock` and additionally asserts the `hashFiles` key still exists. Verified: `Cargo.lock` is tracked. No gap. |
| `scripts/release-naming.rs` | release title/tag formatting | inlined in `scripts/create-github-release.rs` |
| `scripts/package-desktop.sh` | desktop bundle packaging | `scripts/desktop-release-resolve.sh` + `desktop-release.yml` |
| `scripts/fixtures/lychee-report.md` | fixture for the template's own tests | not applicable |

No adoption is outstanding from this list.

## 7. What this comparison did *not* find

Recorded because a comparison that only reports hits is not evidence of having
looked:

* **No practice in any template is missing here except D10.** The templates are
  smaller pipelines; this repository already carries budget wrappers, disk
  reclamation, cache consolidation, ratchets and a `ci-cd` test module that none
  of the templates have.
* **The reverse direction is where the findings were.** Three of the four
  defects that came out of this comparison are defects *in the templates*, found
  by holding this repository's implementation against theirs. That is why R3
  produced five filed issues (`upstream-reports/README.md`) and R2 produced one
  adoption.
