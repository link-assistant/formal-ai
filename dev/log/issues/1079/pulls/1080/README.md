# Issue #1079 / PR #1080 — CI/CD false positives, false negatives, warnings and errors

Issue: <https://github.com/link-assistant/formal-ai/issues/1079>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1080>

## 1. Scope and collection method

Issue #1079 asks for four things, and the fourth is the one that makes it
different from its predecessor #1076:

* **R1** — find and fix every false positive, false negative, warning and
  error in CI/CD;
* **R2** — compare **all files**, not only the workflows, against **five**
  `link-foundation/*-ai-driven-development-pipeline-template` repositories
  (rust, js, python, php, csharp) and adopt their best practices;
* **R3** — where a defect found here also exists in a template, file it there
  too;
* **R4** — follow
  [`hive-mind/docs/CI-CD-BEST-PRACTICES.md`](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md);
* **R5** — plan and execute all of it in this one pull request.

Everything cited below was downloaded before any code was changed and is
committed next to this document, so a reader can re-derive each claim without
GitHub access.

| Path | Contents |
| --- | --- |
| `runs/run-list-main.json`, `runs/run-3*.json` | Full API metadata for all **ten** workflow runs at `main` head `f971b8205`. Taken from the API rather than from the issue text, so a run the issue did not name could not be missed. |
| `runs/jobs-3*.json` | Per-run job and step records for the same ten runs. |
| `ci-logs/main-head-f971b820/run-*.log` | Complete logs for those runs, with `.stderr` retained even when empty, so a silent download failure is distinguishable from a silent run. |
| `ci-logs/main-head-f971b820/job-101566829972-auto-release.log` | The single job that produced the reported failure, isolated for line-level citation. Line 1638 carries the whole verdict. |
| `ci-logs/main-head-f971b820/job-101567057564-pipeline-status.log` | The aggregate gate that turned it into a red pipeline. |
| `annotations/all-annotations.tsv` | Every annotation GitHub attached to any job of any run at this head: 23 rows. |
| `analysis/cargo-audit-evidence.md` | cargo-audit 0.22.2 run against the committed lockfile, both with and without `--deny warnings`, and again after upgrading the yanked crate. The basis for D2. |
| `analysis/zizmor-auditor-inventory.md` | Every zizmor audit that the configured persona cannot report, with the findings each produces on this tree. The basis for D3. |
| `analysis/zizmor-narrow-pedantic-gate.md` | The live-gate proof for the D3 fix: exit 0 as committed, exit 14 with both protections reverted. |
| `analysis/zizmor-narrow-pedantic-templates.log` | The same measurement run against all five templates. |
| `analysis/zizmor-action-v0.6.2-versions.txt` | `zizmor-action@v0.6.2`'s static version table, with the digest equality that makes `latest` a synonym for `1.29.0`. The basis for D6. |
| `analysis/zizmor-1.29.0-csharp-template.log` | The four templates' zizmor invocation applied to the one template that has no zizmor job. |
| `ci-logs/self-hosting-evidence-101601575998.log` | The complete log of the one job that failed on this branch, kept for line-level citation: line 1570 is the finding D8 is about. |
| `analysis/self-hosting-evidence-deadlock.md` | The ruleset query, the rejected force-push, and the design argument for the retraction trailer. The basis for D8. |
| `analysis/template-diffs/` | Per-file diffs of this repository's workflows against all five templates. |
| `references/CI-CD-BEST-PRACTICES.md` | The Hive Mind guidance as of collection (R4). |
| `references/templates/{rust,js,python,php,csharp}-template/` | Complete immutable copies of all five template trees, with `.git` removed and manifests carrying the `.snapshot` suffix required by issue #1014, so no scanner treats archived evidence as a live project. `*.HEAD` and `*.HEADINFO` record the commit each snapshot is of. |
| `upstream-reports/` | The six reports filed against other repositories, with their reproductions; `upstream-reports/README.md` indexes them against the thirteen issue URLs they were filed as. |

## 2. Reconstructed timeline

**Nine** runs were triggered by one event: `f971b8205` — "Merge pull request
\#1078 from link-assistant/issue-1075-9e9c96124ed3" — pushed to `main` at
2026-09-06T21:34:55Z. A tenth, `Desktop Release`, was triggered 33 minutes
later at the same head.

| Created (UTC) | Run | Workflow | Conclusion |
| --- | --- | --- | --- |
| 21:34:55 | 34061510825 | Stock Rust Install | success |
| 21:34:55 | 34061510830 | Question necessity ratchet | success |
| 21:34:55 | 34061510855 | Task Ladder | success |
| 21:34:55 | 34061510865 | Coverage | success |
| 21:34:55 | 34061510881 | Agentic CLI Matrix | success |
| 21:34:55 | 34061510895 | Write-Effect Ladder | success |
| 21:34:55 | 34061510915 | Security | success |
| 21:34:55 | 34061510964 | Broken Link Checker | success |
| 21:34:56 | 34061511110 | **CI/CD Pipeline** | **failure** |
| 22:08:18 | 34063129734 | Desktop Release | success |

One run is red and it is red for one reason. At 22:07:50Z, `Auto Release`
printed:

```
Self-development release preflight failed: release cycle v0.347.0..HEAD has no
merged Formal AI-authored pull request; a merged pull request counts once it
introduced at least one commit carrying valid session evidence, and every
attributed commit it introduced names that same pull request
```

`Pipeline Status` then failed with `Pipeline failed. Failing jobs:
auto-release`. Nothing else in the ten runs failed, and no step timed out.

The interesting half of the audit is therefore not in the red run. It is in
the nine green ones, and in the two gates that were structurally incapable of
turning red at all.

## 3. The causal chain

The three defects this pull request fixes share one shape, and it is worth
naming before the register lists them separately.

**A gate is a claim about what cannot reach `main`. Two of this repository's
gates made a claim they could not enforce, and the green check was
indistinguishable from the enforced version.**

1. `Security / Rust dependency audit` ran `cargo audit` and reported success.
   It was reporting success *while printing two real findings*, because
   `cargo audit` classifies `unmaintained`, `unsound` and `yanked` as warnings
   and a warning does not move its exit status — the last line of a run with
   findings is `warning: 2 allowed warnings found`, and the status is 0. One
   of the two was `chacha20 0.10.1`, a release its own authors had yanked.
   (D2)
2. `.github/zizmor.yml` has declared `'*': hash-pin` since issue #1076 for
   every namespace this repository has not explicitly chosen to trust. That
   policy configures the `unpinned-uses` audit, which reads *action*
   references only. Container images belong to the separate `unpinned-images`
   audit, which zizmor classifies as Pedantic, and the job runs
   `--persona regular`. So the written policy and the enforced policy were
   different policies, and `docker://rhysd/actionlint:1.7.12` — a mutable
   third-party tag that executes with this repository checked out — sat inside
   the workflow whose whole purpose is auditing the pipeline. (D3)
3. The same zizmor job left `version:` at its default while the comment beside
   it told a maintainer to reproduce with `zizmor==1.30.0`. That is not a
   version drift; the action *cannot* install 1.30.0. It resolves versions
   from a static 37-row table shipped inside itself and `die`s on anything
   absent, and v0.6.2's `latest` row carries the same digest as its `1.29.0`
   row. "Reproduce locally with X" is only true when CI runs X. (D6)

Defects 2 and 3 are the same failure at one remove: the audit that would have
caught the unpinned image was itself misconfigured, and the misconfiguration
was invisible because its own version pin was a comment rather than a setting.

## 4. Defect register

| ID | Defect | Class | Evidence | Disposition |
| --- | --- | --- | --- | --- |
| D1 | `Auto Release` blocks the release cycle: `v0.347.0..HEAD` contains no merged Formal AI-authored pull request | **true positive** | `ci-logs/.../job-101566829972-auto-release.log:1638` | reported, deliberately not "fixed" — §4.1 |
| D2 | `cargo audit` exits 0 on `unmaintained`, `unsound` and `yanked` findings, so the Rust dependency audit was green while `Cargo.lock` pinned a yanked `chacha20 0.10.1` | false negative | `analysis/cargo-audit-evidence.md` | fixed: `--deny warnings`, plus the two findings answered |
| D3 | The declared `'*': hash-pin` policy had never been applied to a container image; `unpinned-uses` reads actions, images belong to Pedantic `unpinned-images` | false negative | `analysis/zizmor-auditor-inventory.md`, `analysis/zizmor-narrow-pedantic-gate.md` | fixed: digest pin, narrow pedantic pass, one documented exception |
| D4 | Two files are inside the warning band of `scripts/check-file-size.rs`: `src/protocol.rs` at 981/1000 and `src/seed.rs` at 946/1000 | warning | `annotations/all-annotations.tsv` | §4.2 |
| D5 | Two steps report at 70% of their execution budget: the instrumented coverage run (1680s of 2400s) and the specification test shard (980s of 1400s) | warning | `annotations/all-annotations.tsv` | §4.2 |
| D6 | `zizmor-action@v0.6.2` resolves versions from a static table whose `latest` row *is* its `1.29.0` row, while the reproduction comment named 1.30.0 | error (unrunnable instruction) | `analysis/zizmor-action-v0.6.2-versions.txt` | fixed: `version: 1.29.0` on both passes, comment corrected |
| D7 | 46 of 48 `actions/checkout` steps persisted the job token into `.git/config`; all five templates set `persist-credentials: false` | false negative (below every configured confidence floor) | `analysis/artipacked-sweep.md` | fixed: 44 of 48 swept, 4 documented exceptions |
| D8 | `Self-Hosting Evidence Check` errors on a half-trailered commit and tells the author to amend it, in a repository whose ruleset forbids non-fast-forward pushes on `~ALL` branches with no bypass actors | true positive with no available remedy | `analysis/self-hosting-evidence-deadlock.md`, `ci-logs/self-hosting-evidence-101601575998.log:1570` | fixed: `Formal-AI-Retract`, a trailer that can only lower the measured share |
| D9 | `trailer_values` trims each line before matching, so an *indented* `Formal-AI-*: …` line inside a commit message is read as a declared trailer; `git interpret-trailers --parse` ignores it | false positive | `analysis/self-hosting-evidence-deadlock.md` | fixed: an indented line is not a trailer |

### 4.1 Why D1 is reported rather than fixed

`Auto Release` is not wrong. `scripts/self-development-loop.rs` blocks a
release cycle that contains no merged pull request carrying valid Formal AI
session evidence, and the cycle genuinely contains none. Issue #1066
established that such a cycle is reported as blocked *immediately* rather than
deferred, precisely so a stalled self-hosting metric cannot hide behind a green
check — the failure is the design working.

There is a one-line way to turn it green from inside this pull request: add
`Formal-AI-Session`, `Formal-AI-Evidence` and `Formal-AI-Pull-Request` trailers
to these commits. That would be a lie. The metric measures the share of this
repository's own work that its reviewed self-development loop produced, and
this session is an ad-hoc solver run, not that loop. No commit on this branch
carries those trailers, and D1 stays red until work that legitimately carries
them merges.

This is the one defect in the register that is a **true positive**: a red check
reporting a real condition, in the words the condition deserves.

### 4.2 D4 and D5: warnings that are doing their job

Both are annotations rather than failures, and both are correctly calibrated.

`check-file-size.rs` warns for `.rs` at 900 and fails at 1000. `src/protocol.rs`
at 981 and `src/seed.rs` at 946 are inside the band the warning exists to
describe, and the band exists so a split is a planned change rather than an
emergency at 1001 lines. Splitting either file is a source-structure change
with no CI/CD content, and doing it inside a pull request about the pipeline
would put an unreviewable diff next to the gates it is meant to be judged on.
Recorded here, not changed here.

The two 70%-of-budget warnings are the mechanism issue #977 and issue #1017
built on purpose: a step that reaches 70% of its declared budget says so while
it still has 30% left, so drift is visible a release before it is fatal. The
coverage run at 1680/2400s and the specification shard at 980/1400s are the
warning firing as designed, on a run that then finished successfully. Raising
either budget would remove the signal without changing the runtime.

### 4.3 D7: the finding no configured gate could report

zizmor's `artipacked` audit reported 46 findings on this tree, every one of
them at **Low** confidence. The default pass floors confidence at `medium` and
the narrow pedantic pass at `high`, so neither gate was ever going to report a
single one of them — this is the same shape as D3, one floor lower. An
invariant that no gate enforces is a comment, which is why the fix ships with
a test rather than only with a configuration change.

`actions/checkout` writes the job's token into `.git/config` as an
`http.extraheader` and leaves it there for the rest of the job. Every later
step, and every tool any of them shells out to, can read it; anything that
archives the workspace carries it out of the run.

42 of the 46 now set `persist-credentials: false`, bringing the repository to
44 of 48. Four do not, because a later step in the same job pushes with exactly
the credential the input removes:

* `external-benchmarks.yml` — `git push origin "HEAD:${GITHUB_REF_NAME}"`
  records the scheduled upstream results. The job is already narrowed to
  `github.event_name != 'pull_request'`.
* `release.yml`, `auto-release` and `manual-release` — both run
  `scripts/version-and-commit.rs`, which ends in `git push` and
  `git push --tags`. `scripts/git-config.rs` sets the committer identity and no
  credential helper, so the persisted header is the only auth those pushes
  have. Gated on `push` to `refs/heads/main` and on `workflow_dispatch`
  respectively.
* `release.yml`, `changelog-pr` — `peter-evans/create-pull-request` commits the
  generated changelog fragment and pushes a branch. The job only runs on
  `workflow_dispatch`.

Each carries the reason in a comment above the step, and
`issue_1079::every_checkout_drops_its_credential_unless_it_pushes` requires
that comment, caps the exceptions at four, and requires the other 44 to stay
swept. Measured before and after with `--persona auditor --min-confidence low`:
46 findings → 4, the four being exactly those sites.

The first pass of this sweep got the release pair wrong in the expensive
direction — it removed a credential two jobs push with — and nothing in the
pipeline would have caught it, because both jobs run only on `main` after a
merge. The correction is not a note in this file but a second test,
`issue_1079::every_job_that_pushes_still_has_a_credential_to_push_with`, which
reads each job body for a remote git write and fails if such a job drops its
credential. `analysis/artipacked-sweep.md` records what the first scan missed
and why.

Two things were verified rather than assumed. First, that `git fetch origin`
still succeeds without the credential: `pin-base-commit.sh` and
`simulate-fresh-merge.sh` fetch the base branch on eight of the swept jobs, and
the repository is public, so an anonymous `git fetch --depth=1 origin main`
from a credential-free clone resolves the same `f971b8205`. Second, the
complete list of remote git writes, re-derived over every file under `.github/`
and `scripts/` rather than over the shell scripts the first scan happened to
read: four call sites, in the four jobs that keep their credential. No
composite action runs git at all.

The cost is 44 lines in `release.yml`, which holds 18 of the 48 checkouts, and
that moves it from 1521 to 1565 lines — past the 1500-line warning band issue
#812 set and into a `check-file-size` warning annotation, because the gate
warns on a file that is both over the band and growing. That is the gate
working, and the growth is paid for deliberately rather than absorbed: the
band in `issue_999` and `issue_1012` moves to 1565 with the reason written
beside it, in the same form as the two moves before it. Three of the four
exceptions live in this file, and their comments are most of the difference
between 1553 — the sweep without them — and 1565. `persist-credentials`
is an input to the action that *performs* the checkout, and a local composite
action cannot wrap it, because a local composite action does not exist until
the checkout has run. The alternative was to leave the release path — the one
place credentials matter most, and the only place either legitimate exception
lives — as the single part of the repository the sweep did not reach.

### 4.4 D8: a correct finding with no available remedy

`Self-Hosting Evidence Check` failed on this very pull request, at `ef3bb4aff`
(`ci-logs/self-hosting-evidence-101601575998.log:1570`), because commit
`bd511432a` records `Formal-AI-Evidence` without `Formal-AI-Session`. The
finding is correct and the commit is mine.

The remedy the check implies — amend the commit — does not exist here.
Ruleset 21300712 applies `deletion` and `non_fast_forward` to `~ALL` branches
with an empty `bypass_actors`, and a `filter-branch` rewrite that passed the
metric locally was rejected on push with `GH013 … Cannot force-push to this
branch`. A gate keyed to commit-message content, in a repository where commit
messages are immutable once pushed, is a gate whose only escape is to abandon
the branch. That is a CI/CD defect in its own right — the same deadlock shape
that `a_malformed_historical_evidence_record_cannot_deadlock_a_release`
already removed from the release path after #796/#810/#812; it survived on the
pull-request path only because nobody had mis-trailered a commit there yet.

The fix is the `Formal-AI-Retract` trailer, and its whole design is the
argument that it is safe to add to a gate whose job is to keep a number
honest: it can only move a commit *out* of the numerator, so no retraction can
raise the measured share. Full transcript, including the ruleset query and the
rejected push, in
[`analysis/self-hosting-evidence-deadlock.md`](analysis/self-hosting-evidence-deadlock.md).

### 4.5 What was checked and found clean

Recorded because "found nothing" is a result, and an audit that only lists hits
cannot be distinguished from an audit that stopped early.

* All ten runs at `f971b8205`, every job, every step record — one failure, no
  timeouts, no cancellations.
* All 23 annotations. Nineteen are `notice` (sccache hit rates 47%–100%,
  closure-driven cache-bucket exemptions, a Codecov-not-configured note); the
  four non-notices are D4 ×2, D5 ×2, and the three `failure` rows are D1.
* `moby/buildkit:buildx-stable-1` in
  `.github/actions/setup-buildx-resilient/action.yml`. It is a *third* mutable
  image reference, and neither zizmor's `unpinned-images` nor this
  repository's own sweep reads it, because it is the `default:` of an action
  input rather than an image key. It stays mutable deliberately: pinning a
  digest to something named `buildx-stable-1` defeats the name, and it is a
  first-party image from the distribution this pipeline already trusts to
  build with.
* The rust template's own `Cargo.lock` at `4d444d97`, audited both ways with
  cargo-audit 0.22.2: 45 crates, no findings either way. D2's *mechanism* is
  present upstream; the instance is not, which is why the report there is
  framed as latent.

## 5. Requirement-by-requirement: root cause and plan

### R1 — every false positive, false negative, warning and error

| Defect | Root cause | Fix | Regression test |
| --- | --- | --- | --- |
| D2 | `cargo audit`'s exit status is a function of *vulnerabilities* only. `unmaintained`/`unsound`/`yanked` are a separate class it calls warnings, and the flag that promotes them, `--deny warnings`, is off by default. A yanked crate additionally carries no advisory ID, so `[advisories] ignore` cannot silence it even deliberately — the only lever is `--no-yanked`, which removes the check. | `scripts/check-rust-dependencies.sh` runs `cargo audit --file Cargo.lock --deny warnings`. The two findings it then reports are answered rather than suppressed: `chacha20` upgraded past the yank; `fxhash` ignored under a new proof form. | `issue_1079::the_rust_dependency_audit_treats_a_warning_as_a_finding` |
| D2, the `fxhash` half | `fxhash 0.2.1` is unmaintained with `patched = []`, and it is genuinely compiled in through `web-capture → scraper 0.21 → selectors 0.26`. Nothing in this repository can remove it; the fix is a requirement bump in another project's manifest. | A second proof form. `unreachable` fails when the crate *enters* the graph; the new `blocked-upstream` fails when it *leaves* — which is exactly when the upstream fix lands and the entry should go. It must name a filed report URL, checked by regex. Filed as web-capture#155 with a build proving `scraper 0.25` needs no source changes. | `issue_1079::every_ignored_advisory_carries_exactly_one_proof` |
| D3 | zizmor's persona filter is applied before severity, and `unpinned-uses` (Regular, configured by `policies:`) and `unpinned-images` (Pedantic) are different audits over different reference kinds. Writing `'*': hash-pin` therefore said nothing at all about images. | Digest-pin both actionlint references; add a second zizmor pass at `--persona pedantic --min-severity high --min-confidence high`. There is no narrower expression available: 1.29 and 1.30 both reject `rules.<audit>.persona` ("unknown field `persona`, expected one of `disable`, `ignore`, `config`, `remap`"), and `remap` rewrites severity, which is not what the persona filter reads. The narrow pass reports 0 of the 164 pedantic findings on the clean tree and 2 with the protections reverted. | `issue_1079::every_container_image_is_digest_pinned_or_explicitly_excepted`, `..::a_pedantic_pass_enforces_the_hash_pin_policy_on_images`, `..::the_actionlint_image_is_pinned_once_and_used_everywhere` |
| D6 | `zizmor-action` does not resolve versions from PyPI. It ships `support/versions`, a static table of 37 rows, and `die`s on a version absent from it. Leaving `version:` unset selects the `latest` row, which in v0.6.2 is byte-identical to the `1.29.0` row. So the default does not float — it freezes, one minor release behind. | `version: 1.29.0` on both passes, and the reproduction comment corrected to match. The next bump is now a visible line in the diff rather than a side effect of bumping the action. | `issue_1079::every_zizmor_pass_pins_the_version_its_comment_documents` |
| D7 | `artipacked` is a Low-confidence audit, and both gates floor confidence above it, so a practice all five templates follow could go unadopted at 46 of 48 sites with every check green. | `persist-credentials: false` at the 44 checkouts whose job never pushes; the four that do push keep it and say why in a comment above the step. The sweep is checked in both directions, because removing a credential a release job pushes with fails on `main`, in a workflow no pull request runs. | `issue_1079::every_checkout_drops_its_credential_unless_it_pushes`, `..::every_job_that_pushes_still_has_a_credential_to_push_with` |
| D8 | The pull-request evidence gate is strict on the premise that a commit in review can still be amended. Ruleset 21300712 applies `deletion` and `non_fast_forward` to `~ALL` branches with an empty `bypass_actors`, so no commit message in this repository can ever be rewritten. A correct finding therefore had no remedy — the same deadlock shape #796/#810/#812 removed from the release path, surviving on the pull-request path only because nobody had mis-trailered a commit there yet. | `Formal-AI-Retract: <full sha>`: a later commit in the same range withdraws an earlier claim. It can only move a commit *out* of the numerator, the target must be a full 40-character sha inside the measured range, and it cannot name itself — so it cannot raise the measured share. Applied on both walks, because applying it on one would let a commit leave the metric and still count toward the release floor. | `self_hosting_metric::a_retraction_unblocks_a_branch_whose_history_cannot_be_rewritten`, `..::a_retraction_can_only_lower_the_measured_share`, `..::a_retraction_also_withdraws_the_commit_from_the_release_floor`, `..::a_retraction_must_name_a_full_sha_inside_the_measured_range` |
| D9 | `trailer_values` deliberately scans the whole commit body rather than using git's `%(trailers)` placeholder (issue #796, where a blank line hid a trailer), and it `trim()`s each line before matching the key. Indentation is the one part of git's rule that mattered: a message that *documents* a trailer in an indented example thereby declares one. Reproduced immediately — the commit introducing `Formal-AI-Retract` showed the format in its own message and the gate answered `must name a full 40-character sha, found <full 40-character sha>`. | Skip lines beginning with a space or tab, matching `git interpret-trailers --parse`, which returns nothing for an indented line. The blank-line tolerance #796 needs is untouched. | `self_hosting_metric::an_indented_example_of_a_trailer_is_not_a_trailer` |
| D1 | Not a pipeline defect. §4.1. | — | existing `self-development-loop` tests |
| D4, D5 | Warnings behaving as designed. §4.2. | — | existing size and budget gates |

### R2 — compare all files against five templates

Root cause of what the comparison found: this repository adopted the templates'
*workflows* in #1076 and did not re-derive the comparison after the csharp
template was added to the set. The comparison in `upstream-reports/README.md`
runs both ways and is summarised there; the short version is that the two
defects fixed here reproduce in every template that has the relevant job, and
that three template-side gaps exist which this repository does not share
because it never adopted the gap.

The one practice a template had that this repository lacked:
`persist-credentials: false` on `actions/checkout`, missing at 46 of this
repository's 48 checkouts. That is D7, and it is swept here — see §4.3.

### R3 — file upstream

Thirteen issues, indexed with their URLs and the report body each was filed
from, in [`upstream-reports/README.md`](upstream-reports/README.md). Each
carries a reproduction that runs at the snapshotted template commit, a
workaround, and the code-level fix.

### R4 — hive-mind CI/CD best practices

`references/CI-CD-BEST-PRACTICES.md` is the copy this work was checked against.
Principle 14 is the one that governs the zizmor configuration, and this pull
request tightened rather than relaxed it: the default-persona pass still sets a
**confidence** floor and no **severity** floor, because severity filtering hides
real findings while confidence filtering drops noisy ones. The second pass sets
both floors, and that is the opposite trade — it *adds* a class of finding the
default persona cannot report at all, then narrows to it. The distinction is
pinned in
`issue_1076::workflows_are_audited_for_security_not_only_syntax`, which now
scopes its "no `min-severity`" assertion to the default-persona pass rather
than to the file.

### R5 — one pull request

This one. Every commit on `issue-1079-92eb3dd8a0fa` is atomic and carries its
own reproduction.

## 6. Existing components and libraries

Checked before writing anything, because the best fix for a pipeline defect is
usually a tool that already exists.

| Need | Existing thing | Verdict |
| --- | --- | --- |
| Fail a Rust build on unmaintained/yanked crates | `cargo-audit`'s own `--deny warnings` | **Adopted.** The capability was already installed and switched off. |
| Ignore an advisory without ignoring it forever | `cargo-deny`'s `[advisories] ignore` with `reason` and an expiry | Rejected. `cargo-deny` would be a second dependency-policy tool beside `cargo-audit`, and its expiry is a date — a date does not know whether the upstream fix landed. The two proof forms here are re-derived from `cargo tree` on every run, so they expire on the *fact* rather than on the calendar. |
| Enforce digest pinning on images | zizmor `unpinned-images`; `ratchet`; Dependabot's Docker ecosystem | zizmor **adopted** (it was already present and unreachable). `ratchet` pins and unpins references but has no policy engine and no PR annotations. Dependabot updates a pinned digest but never objects to an unpinned one, so it complements the gate rather than replacing it. |
| Per-rule persona in zizmor | `rules.<audit>.remap` | **Tested and rejected on evidence.** `remap` accepts only `severity`; persona is applied first, so a finding remapped to `high` is still suppressed. `persona:` under `rules.<audit>` is rejected outright by both 1.29.0 and 1.30.0. The two-pass arrangement is the workaround, and the transcripts of both rejections are in `upstream-reports/templates-unpinned-actionlint-image.md`. |
| Lint `run:` blocks | `actionlint` + ShellCheck | Already adopted in #1076, via the Docker image because the bare binary skips every `run:` check and exits 0 when ShellCheck is absent. This repository's ShellCheck canary fixture has no counterpart in any of the five templates, and is recommended to them. |
| Audit PHP dependencies (the php template's gap) | `composer audit --locked --abandoned=fail` | Recommended upstream. Worth recording that Composer answered the same design question as `cargo audit` the *opposite* way: `--abandoned` has defaulted to `fail` since Composer 2.7, having defaulted to `report` in 2.6 when the option was introduced. |

## 7. Verbose output

`FORMAL_AI_CI_VERBOSE` already exists as this repository's opt-in CI diagnostic
switch (issue #1076), and remains default-off; nothing in this pull request
sets it to `true`, and
`issue_1076::runner_telemetry_exists_and_defaults_to_off` enforces that.

No new switch was needed here, because none of D1–D6 was undiagnosable. Each
root cause was reached by direct measurement rather than by inference, and each
measurement is committed in `analysis/` so the next reader re-runs it instead
of trusting this document. The one place where more output *was* the fix is
`actionlint -verbose`, added in #1076 and kept: it costs one line per workflow
and carries the fact the exit status does not — how many files were actually
read. A silent 0 and a 0 after "Found 0 errors in 19 files" are not the same
result.
