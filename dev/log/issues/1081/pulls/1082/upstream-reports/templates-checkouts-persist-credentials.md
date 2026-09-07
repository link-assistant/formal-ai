# Upstream report 1 - `actions/checkout` keeps `GITHUB_TOKEN` in `.git/config`, and the zizmor gate that would say so floors it out

**Targets:** `link-foundation/js-ai-driven-development-pipeline-template`,
`link-foundation/php-ai-driven-development-pipeline-template`,
`link-foundation/csharp-ai-driven-development-pipeline-template`

**Severity:** every step after the checkout -- including `npm install`,
`composer install`, `dotnet restore` and every third-party action in the job --
runs with a writable repository credential sitting in `.git/config`, in jobs
that never push. The templates already ship the audit that reports this; its
findings are Low confidence and the gate floors confidence at `medium`, so the
gate is green by construction.

---

## Title

`persist-credentials: false` is set on 2 of 27 checkouts (js) / 2 of 11 (php) /
1 of 13 (csharp); zizmor reports the rest but `--min-confidence medium` hides
every one of them

## Evidence

`actions/checkout` writes the job token into `.git/config` as an
`http.extraheader` and leaves it there for the remainder of the job unless
`persist-credentials: false` is passed. That is zizmor's `artipacked` audit.

Measured at the commits in `../references/templates/*.HEAD`, with zizmor
1.29.0, offline, using each template's own `.github/zizmor.yml`:

```console
# what the template's own gate reports
$ zizmor --offline --config .github/zizmor.yml \
    --persona regular --min-confidence medium .github/workflows

# what is actually there
$ zizmor --offline --config .github/zizmor.yml \
    --persona auditor .github/workflows
```

| template | checkouts | `persist-credentials: false` | artipacked (auditor) | artipacked (own gate) | confidences |
|---|---|---|---|---|---|
| rust | 26 | 26 | 2 | 0 | all Low |
| js | 27 | 2 | **25** | **0** | all Low |
| python | 18 | 18 | 1 | 0 | all Low |
| php | 11 | 2 | **9** | **0** | all Low |
| csharp | 13 | 1 | **12** | **0** | all Low |

rust's 2 and python's 1 are not unswept checkouts. They are the release jobs
that push, and each one says so:

```yaml
      # The only checkout in this repository that keeps credentials in
      # .git/config: scripts/version_and_commit.py runs `git push origin main`
      # to publish the version bump, which needs the token wired into the
      # remote. Every other checkout sets persist-credentials: false.
      - uses: actions/checkout@v6
        with:
          persist-credentials: true
          fetch-depth: 0
          token: ${{ secrets.GITHUB_TOKEN }}
```
(`python-ai-driven-development-pipeline-template/.github/workflows/release.yml:633`)

js, php and csharp never set the input at all. The full per-finding list is in
`../analysis/template-artipacked-sweep.md`; the shape is the same everywhere:

```
js   release.yml   test                  L291   uses: actions/checkout@v6
js   release.yml   lint                  L212   uses: actions/checkout@v6
js   security.yml  npm-audit             L85    uses: actions/checkout@v6
php  release.yml   test                  L136   uses: actions/checkout@v4
php  docs.yml      build                 L41    uses: actions/checkout@v4
csharp release.yml build                 L293   uses: actions/checkout@v6
csharp security.yml codeql               L29    uses: actions/checkout@v6
```

## Why the existing gate does not catch it

Each template's zizmor job runs with `min-confidence: medium`:

```yaml
      - uses: zizmorcore/zizmor-action@v0.6.2
        with:
          advanced-security: false
          annotations: true
          config: .github/zizmor.yml
          min-confidence: medium
```
(`js-ai-driven-development-pipeline-template/.github/workflows/workflows.yml:64`)

Every `artipacked` finding on a checkout that simply omits the input is emitted
at **Low** confidence -- zizmor cannot prove from the workflow text that the
credential is unused, so it does not claim to. The floor is above the finding.
This is a false negative in the security gate, not an absence of the defect:
the same measurement with `--persona auditor` and no floor returns 25 / 9 / 12.

`csharp` compounds it: that template has no zizmor job and no
`.github/zizmor.yml` at all (already reported as
`csharp-ai-driven-development-pipeline-template#53`), so its 12 findings have
no pass that could report them even in principle.

## Reproducible example

```console
$ git clone https://github.com/link-foundation/js-ai-driven-development-pipeline-template
$ cd js-ai-driven-development-pipeline-template
$ pipx run zizmor==1.29.0 --offline --config .github/zizmor.yml \
    --persona regular --min-confidence medium --format json .github/workflows \
  | jq '[.[] | select(.ident == "artipacked")] | length'
0

$ pipx run zizmor==1.29.0 --offline --config .github/zizmor.yml \
    --persona auditor --format json .github/workflows \
  | jq '[.[] | select(.ident == "artipacked")] | length'
25

$ pipx run zizmor==1.29.0 --offline --config .github/zizmor.yml \
    --persona auditor --format json .github/workflows \
  | jq -r '[.[] | select(.ident == "artipacked")
            | .determinations.confidence] | unique | @csv'
"Low"
```

The last command is the whole report in one line: the findings exist, and they
are all below the floor the gate sets.

## Workaround

For a repository generated from one of these templates, add the input to every
checkout that does not push, and pin the exceptions explicitly:

```yaml
      - uses: actions/checkout@v6
        with:
          persist-credentials: false
```

Until that is done, lower the gate's floor in `.github/workflows/workflows.yml`
to see the findings:

```yaml
        with:
          min-confidence: low
```

That reports them, but it also unfloors every other Low-confidence audit, which
is why it is a workaround and not the fix.

## Suggested fix in code

Three parts, in this order.

**1. Sweep the checkouts that do not push.** The jobs that must keep the
credential are the ones that write to the remote, and they are enumerable:

| template | jobs that push | mechanism | jobs to sweep |
|---|---|---|---|
| js | `release`, `instant-release` | `scripts/version-and-commit.mjs` -> `scripts/push-main-with-rebase-retry.mjs` (`release.yml:515`, `:606`) | 22 of 25 |
| js | `changeset-pr` | `peter-evans/create-pull-request` (`release.yml:828`) | |
| php | `auto-release`, `manual-release` | `scripts/version-and-commit.php` -> `VersionReleaser` -> `Git::push` (`release.yml:263`, `:319`) | 7 of 9 |
| csharp | `release`, `instant-release` | `scripts/version-and-commit.mjs:423` `git push` / `:424` `git push --tags` (`release.yml:418`, `:614`) | 9 of 12 |
| csharp | `changeset-pr` | `peter-evans/create-pull-request` (`release.yml:776`) | |

Everything else -- `lint`, `test`, `build`, `detect-changes`, `docker-*`,
`codeql`, `dependency-review`, `npm-audit`, `link-checker`, `docs`,
`pipeline-status` -- fetches and never writes, so it can drop the credential.
Public-repository fetches still work with no credential at all, which is worth
verifying rather than assuming:

```console
$ git init -q && git remote add origin https://github.com/<owner>/<repo>.git
$ GIT_TERMINAL_PROMPT=0 GIT_ASKPASS=/bin/true git fetch --depth=1 origin main
From https://github.com/<owner>/<repo>
 * branch            main       -> FETCH_HEAD
```

Do this direction carefully: sweeping a job that *does* push removes the only
credential its release has, and the failure surfaces only on `main`, after the
package is already published. Downstream this was gotten wrong once and is
written up in
[`analysis/artipacked-sweep.md`](https://github.com/link-assistant/formal-ai/blob/main/dev/log/issues/1079/pulls/1080/analysis/artipacked-sweep.md#the-two-the-first-pass-of-this-sweep-broke).

**2. Make each exception carry its reason.** The python template's comment
above is the model: name the push the credential exists for.

**3. Make the invariant a test, not a habit.** A swept tree regresses the first
time someone adds a job. Two assertions over the workflow text are enough, and
they are the reason the sweep stays swept:

* every checkout sets `persist-credentials: false` unless its job writes to the
  remote (`git push`, the version-and-commit script, `create-pull-request`),
  and the exception count is pinned to an exact number;
* no job that writes to the remote sets `persist-credentials: false`.

Downstream these are
`tests/unit/issue_1079.rs::every_checkout_drops_its_credential_unless_it_pushes`
and `::every_job_that_pushes_still_has_a_credential_to_push_with`. In the js and
csharp templates the natural home is `tests/ci-timeouts.test.js`'s neighbours;
in php, `tests/` beside the other workflow assertions.

Raising `min-confidence` to `low` is deliberately *not* the recommendation on
its own: it reports 25 findings and an unmeasured number of unrelated
Low-confidence ones in the same run, which is how a gate gets an allowlist
instead of a fix.
