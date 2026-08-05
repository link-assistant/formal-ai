The Rust template releases to crates.io and deploys rustdoc to Pages, but a Rust
project that also ships a **desktop application** has no template path at all:
no release-asset build, no checksums, no build provenance, and no `/download`
page. Downstream `link-assistant/formal-ai` built that layer from scratch and
paid for two non-obvious bugs while doing it. Both are worth capturing upstream
so the next project does not repeat them.

This is an **enhancement / optional workflow** request, not a bug report against
the current template.

## Current state (verified 2026-08-05, `main` @ `c867f78`)

```bash
gh api repos/link-foundation/rust-ai-driven-development-pipeline-template/git/trees/HEAD?recursive=1 \
  --jq '.tree[] | select(.path | test("^\\.github/workflows/")) | .path'
# -> .github/workflows/release.yml   (only)

grep -rn 'workflow_run\|head_sha' .github/
# -> no matches
```

No template in the fleet has a desktop-release workflow. The only desktop
packaging anywhere is `js-ai-driven-development-pipeline-template`'s
`example-app.yml` `desktop-package` job (Electron Forge, `[ubuntu, macos,
windows]`), and it uploads to **CI artifacts only** (`upload-artifact@v7`) —
never to a Release — so it does not answer "how does a user download my app".

## What the downstream implementation provides

`link-assistant/formal-ai`'s
[`.github/workflows/desktop-release.yml`](https://github.com/link-assistant/formal-ai/blob/main/.github/workflows/desktop-release.yml):

- a 6-target matrix (`linux-x64`, `linux-arm64`, `macos-x64`, `macos-arm64`,
  `windows-x64`, `windows-arm64`), `fail-fast: false`;
- SLSA build provenance via `actions/attest-build-provenance@v2`
  (verifiable with `gh attestation verify`);
- a consolidated `SHA256SUMS.txt` + `BUILD-PROVENANCE.txt` attached to the
  release;
- a pull-request **dry run** of the whole matrix (everything up to and including
  the smoke tests, with all publishing steps disabled);
- a `/download` page generated from the GitHub Releases API, served from the
  same Pages site as the docs.

## Reproducible bug #1 to avoid — resolving the release from `workflow_run.head_sha`

**Repro.** A `workflow_run`-triggered desktop job that resolves its target tag with

```bash
gh api "repos/$REPO/tags" --jq '.[] | select(.commit.sha=="'"$HEAD_SHA"'")'
```

returns **empty on every run** in a template of this shape. The auto-release
path bumps the version and pushes a *child* commit (`chore: release vX.Y.Z`)
and tags **that** commit; `github.event.workflow_run.head_sha` is the *parent*
CI commit. The tag therefore never sits on the head SHA, the build is skipped
forever, and the download page stays empty — silently, with the pipeline green.
This was `link-assistant/formal-ai`
[#479](https://github.com/link-assistant/formal-ai/issues/479): `/download`
was broken through v0.203.0 with no failing check anywhere.

**Workaround.** Trigger the desktop build manually (`workflow_dispatch` with an
explicit tag) after each release.

**Fix to ship.** Two-tier resolution, as in
[`scripts/desktop-release-resolve.sh`](https://github.com/link-assistant/formal-ai/blob/main/scripts/desktop-release-resolve.sh):
tier 1 (defensive) an exact tag on the head SHA; tier 2 (the normal path) the
**latest published release**, since the auto-release tags a child commit whose
first parent is the CI head SHA. Add an idempotency guard so re-runs do not
re-upload, and unit-test the resolver
([`tests/unit/ci-cd/desktop_release_resolve.rs`](https://github.com/link-assistant/formal-ai/blob/main/tests/unit/ci-cd/desktop_release_resolve.rs)) —
this template already has a `tests/unit/ci-cd/` convention for exactly that.

## Reproducible bug #2 to avoid — electron-builder 26 skips a custom `mac.sign` hook

Only relevant if the optional workflow uses Electron + electron-builder.

**Repro.** On electron-builder **26**, with no Apple certificate present:

```bash
electron-builder --mac --publish never -c.mac.sign=./sign.cjs
```

The hook is **never invoked** and the produced `.app` has no
`Contents/_CodeSignature/CodeResources`. `MacPackager.findSigningIdentity()`
returns `null` for any qualifier other than `"-"` when no certificate exists
(even with `CSC_IDENTITY_AUTO_DISCOVERY=false`), so `isSignAllowed()` skips
signing entirely and the custom hook is bypassed. This is a behavior change from
electron-builder 25, where the hook ran without the flag, and it caused
formal-ai's macOS-only asset failure at v0.205.0 / v0.206.0.

**Workaround / fix.** Always pass `-c.mac.identity=-` on the ad-hoc path:

```bash
electron-builder --mac --publish never \
  -c.mac.notarize=false -c.mac.identity=- -c.mac.sign=./scripts/adhoc-sign-mac.cjs
```

A second, related trap: `builder-util`'s `isPullRequest()` makes
`isSignAllowed()` bail out with *"Current build is a part of pull request, code
signing will be skipped"* whenever `GITHUB_BASE_REF` is set, so the pull-request
dry run silently produces an unsigned bundle. On the ad-hoc path (no secrets,
identity `-`, nothing to leak) set `CSC_FOR_PULL_REQUEST: "true"`.

**Verification to ship with it.** A pre-upload smoke test that fails when a
macOS artifact is missing its signed `CodeResources` envelope — without it both
bugs above are invisible until a user downloads a broken app.

## Suggested scope

An **opt-in** `desktop-release.yml` (gated on a repository variable, like the
existing opt-in Docker Hub publish) plus the resolver script and its unit test.
Templates that do not ship a desktop app pay nothing.

## Provenance

Four-template CI/CD comparison audit run in downstream
`link-assistant/formal-ai` (report:
<https://github.com/link-assistant/formal-ai/blob/main/docs/case-studies/issue-479/template-comparison/REPORT.md>,
upstream recommendation 3; filed under
<https://github.com/link-assistant/formal-ai/issues/894>).
