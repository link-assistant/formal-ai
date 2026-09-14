# Defect D7 -- the `artipacked` sweep

`actions/checkout` writes the job's token into `.git/config` as an
`http.extraheader` and leaves it there for the rest of the job. zizmor's
`artipacked` audit reports that, and it reported it 46 times on this tree.

Every one of the 46 was **Low** confidence. The repository's default zizmor
pass floors confidence at `medium` and its narrow pedantic pass floors it at
`high`, so no configured gate could report any of them. All five
`link-foundation/*-ai-driven-development-pipeline-template` repositories set
`persist-credentials: false`; this one did so at 2 of its 48 checkouts.

## Measurement

Command (auditor persona, no floors, so nothing is hidden by configuration):

```
zizmor --config .github/zizmor.yml --persona auditor --format json \
  .github/workflows .github/actions
```

| audit | before | after |
| --- | --- | --- |
| `artipacked` | 46 | 4 |
| `template-injection` | 41 | 41 |
| `superfluous-actions` | 32 | 32 |
| `secrets-outside-env` | 22 | 22 |
| `undocumented-permissions` | 12 | 12 |
| `github-env` | 3 | 3 |
| `cache-poisoning` | 3 | 3 |
| `anonymous-definition` | 2 | 2 |
| `concurrency-limits` | 1 | 1 |
| **total** | **162** | **118** |

Only `artipacked` moves. The sweep touches one input on one action and changes
no other audit's result, which is the point of measuring the whole set rather
than the one number that was expected to change.

## The four that remain

```
--> .github/workflows/external-benchmarks.yml:96
--> .github/workflows/release.yml:491
--> .github/workflows/release.yml:711
--> .github/workflows/release.yml:889
```

Each is a job whose later steps push with exactly the credential the input
removes:

* `external-benchmarks.yml:178` runs `git push origin "HEAD:${GITHUB_REF_NAME}"`
  to record the scheduled upstream results. The job is gated on
  `github.event_name != 'pull_request'`.
* `release.yml`'s `auto-release` and `manual-release` jobs both run
  `scripts/version-and-commit.rs`, which ends in `git push` (with three retry
  attempts) and `git push --tags`. `auto-release` is gated on
  `github.event_name == 'push' && github.ref == 'refs/heads/main'`;
  `manual-release` on `workflow_dispatch`.
* `release.yml:892` is `peter-evans/create-pull-request@v8`, which commits the
  generated changelog fragment and pushes a branch. The job is gated on
  `github.event_name == 'workflow_dispatch'`.

Each carries the reason in a comment directly above the step, and
`issue_1079::every_checkout_drops_its_credential_unless_it_pushes` requires
that comment, caps the exceptions at four, and requires the other 44 to stay
swept -- so a fifth exception cannot be added silently, which is the failure
mode a hand-written allowlist has.

## The two the first pass of this sweep broke

Worth recording, because it is the more expensive direction to get wrong and
because the first scan below was written as if it had been exhaustive when it
was not.

The scan searched the workflows and the shell scripts they call, found remote
git operations only in `scripts/pin-base-commit.sh` and
`scripts/simulate-fresh-merge.sh`, and concluded that only two jobs push. It
missed `scripts/version-and-commit.rs`, because it is a `rust-script` file
invoked as `rust-script scripts/version-and-commit.rs --bump-type ...` and the
scan had looked at `.sh` files. `auto-release` and `manual-release` therefore
received `persist-credentials: false`, which would have removed the only
credential their release push has -- `scripts/git-config.rs` sets `user.name`
and `user.email` and no credential helper, and nothing else in either job
supplies one.

Nothing would have caught this before it shipped. Both jobs run only on
`main`, after a merge; neither is exercised by any pull request; and the
failure would have surfaced as a release that builds, publishes to crates.io
and GHCR, and then fails at `git push` with the version bump uncommitted.

Both are reverted, each with a comment naming the push it keeps its credential
for. The lesson is in the test, not in this file:
`issue_1079::every_job_that_pushes_still_has_a_credential_to_push_with` reads
each job body, asks whether anything in it writes to the remote over git --
`git push`, `scripts/version-and-commit.rs`, `peter-evans/create-pull-request`
-- and fails if such a job sets `persist-credentials: false`. It asserts the
count is exactly four, so a new pushing job cannot be added without deciding
which side of the invariant it is on.

## What was verified, rather than assumed

**Which jobs actually need the credential.** Re-done over the whole repository
rather than the shell scripts alone: every `.github/**` file, every file under
`scripts/`, and every composite action under `.github/actions/`, for `git push`
in any form and for the actions that push on a job's behalf. The complete list
of remote git writes is four call sites -- `external-benchmarks.yml:178`,
`scripts/version-and-commit.rs:764` and `:797`, and
`peter-evans/create-pull-request@v8` at `release.yml:892`. (`scripts/version-and-commit-tests.rs`
pushes too, but into a temporary local repository it creates itself, and
`scripts/detect-code-changes.rs` matches only because `"push"` is an event
name.) No composite action runs git at all.

**That a credential-free fetch still works.** `scripts/pin-base-commit.sh` and
`scripts/simulate-fresh-merge.sh` fetch the base branch and run on eight of the
swept jobs. The repository is public, so the fetch succeeds anonymously:

```
$ git init -q && git remote add origin https://github.com/link-assistant/formal-ai.git
$ GIT_TERMINAL_PROMPT=0 GIT_ASKPASS=/bin/true \
    git fetch --depth=1 origin main
From https://github.com/link-assistant/formal-ai
 * branch            main       -> FETCH_HEAD
$ git rev-parse FETCH_HEAD
f971b82052e523f6478f6bf1815c89a326153a23
```

Same head the credentialed checkout resolves, from a clone that has no
credential to offer.

**That the workflows still parse and both gates still pass.** actionlint
(`rhysd/actionlint@sha256:b1934ee5...`, `-no-color -verbose`) reports
`Found 0 errors in 19 files`; the default pass and the narrow pedantic pass
both exit 0.

## Cost

`release.yml` holds 18 of the 48 checkouts, and the sweep costs it 44 lines --
1521 to 1565, past the 1500-line warning band issue #812 set. The input has
nowhere cheaper to live: it is an input to the action that *performs* the
checkout, and a local composite action cannot wrap it, because a local
composite action does not exist until the checkout has run. Three of the four
exceptions are in this file too, and their comments are most of the difference
between 1553 (the sweep without them) and 1565. The band moves to 1565 in
`issue_999` and `issue_1012` with the reason written beside it, in the same
form as the two moves before it.
