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
| `artipacked` | 46 | 2 |
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

## The two that remain

```
--> .github/workflows/external-benchmarks.yml:87:7
--> .github/workflows/release.yml:879:9
```

Both are jobs whose later steps push with exactly the credential the input
removes:

* `external-benchmarks.yml:170` runs `git push origin "HEAD:${GITHUB_REF_NAME}"`
  to record the scheduled upstream results. The job is gated on
  `github.event_name != 'pull_request'`.
* `release.yml:879` is `peter-evans/create-pull-request@v8`, which commits the
  generated changelog fragment and pushes a branch. The job is gated on
  `github.event_name == 'workflow_dispatch'`.

Each carries the reason in a comment directly above the step.
`issue_1079::every_checkout_drops_its_credential_unless_it_pushes` requires
that comment, caps the exceptions at two, and requires the other 44 to stay
swept -- so a third exception cannot be added silently, which is the failure
mode a hand-written allowlist has.

## What was verified before the sweep, rather than assumed

**Which jobs actually need the credential.** The scan was per job, over the
workflows *and* over every script and composite action they call, for
`git push`, `git fetch`, `git pull`, `git clone`, `git ls-remote` and
`git remote set-url`. Two files outside the workflows perform remote git
operations at all -- `scripts/pin-base-commit.sh` and
`scripts/simulate-fresh-merge.sh` -- and both only fetch.

**That a credential-free fetch still works.** Those two scripts run on eight of
the swept jobs. The repository is public, so the fetch succeeds anonymously:

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

`release.yml` holds 18 of the 48 checkouts, and the sweep costs it 32 lines --
1521 to 1553, past the 1500-line warning band issue #812 set. The input has
nowhere cheaper to live: it is an input to the action that *performs* the
checkout, and a local composite action cannot wrap it, because a local
composite action does not exist until the checkout has run. The band moves to
1553 in `issue_999` and `issue_1012` with the reason written beside it, in the
same form as the two moves before it.
