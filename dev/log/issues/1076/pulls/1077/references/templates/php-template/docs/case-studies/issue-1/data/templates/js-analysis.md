# Analysis: `link-foundation/js-ai-driven-development-pipeline-template`

Reference analysis of the JavaScript/TypeScript AI-driven-development pipeline
template, written to inform an equivalent PHP/Symfony + Composer template. Raw
copies of every file referenced here are saved under
`docs/case-studies/issue-1/data/templates/js/` (relative paths preserved).

The package under test is the **real, published** npm package
`@link-foundation/example-package-name` (current version `0.11.4`). Publishing a
live package means the template's release pipeline is validated end-to-end on
every change, including npm trusted publishing — not mocked.

---

## 1. Repository layout (relevant parts)

| Path                                            | Role                                                              |
| ----------------------------------------------- | ----------------------------------------------------------------- |
| `.github/workflows/release.yml`                 | Main "Checks and release" pipeline (CI + publish). 600 lines.     |
| `.github/workflows/links.yml`                   | Broken-link checker with Web Archive fallback.                    |
| `.github/workflows/example-app.yml`             | Builds/deploys the universal example app + regenerates previews.  |
| `.github/actions/publish-dockerhub/action.yml`  | Composite action: optional Docker Hub publish.                    |
| `.changeset/config.json`, `.changeset/README.md`| Changesets config (versioning mechanism).                         |
| `scripts/*.mjs`, `scripts/*.sh`                  | All release/check logic lives in scripts, invoked by workflows.   |
| `bin/example-package-name.js`                   | Example CLI entrypoint (`add`/`multiply`).                        |
| `package.json`, `deno.json`, `bunfig.toml`      | Multi-runtime manifests (Node, Deno, Bun).                        |
| `eslint.config.js`, `.prettierrc`, `.jscpd.json`| Lint / format / copy-paste-detection config.                      |
| `.secretlintrc.json`, `.lycheeignore`           | Secret scanning, link-check ignore list.                          |
| `.husky/pre-commit`                             | Git pre-commit hook (`npx lint-staged`).                          |
| `docs/BEST-PRACTICES.md`, `docs/CONTRIBUTING.md`| Documentation of conventions.                                     |
| `docs/case-studies/issue-*/`                     | Research notes documenting *why* each decision was made. (exists) |

**Key architectural decision:** workflow YAML stays thin; all logic is in
`scripts/*.mjs` (Node ES modules) and `scripts/*.sh`. The scripts are unit
tested under `tests/` and reused across jobs. This is itself a best practice —
it keeps `release.yml` under the enforced 1500-line limit and makes the logic
testable outside CI.

---

## 2. Release pipeline (versioning, changesets, npm, GitHub release)

### 2.1 Versioning model — Changesets

The template uses [Changesets](https://github.com/changesets/changesets)
(`@changesets/cli`) as the single source of truth for version bumps.
`.changeset/config.json`:

```json
{
  "changelog": "@changesets/cli/changelog",
  "commit": false,
  "fixed": [], "linked": [],
  "access": "public",
  "baseBranch": "main",
  "updateInternalDependencies": "patch",
  "ignore": []
}
```

Mechanics:

- A contributor runs `npm run changeset` (or `bun run changeset`), selects
  patch/minor/major, and writes a one-line summary. This produces a randomly
  named markdown file in `.changeset/` committed with the PR.
- **No one ever edits `version` in `package.json` by hand.** A dedicated CI job
  (`version-check`, runs `scripts/check-version.mjs`) *blocks* PRs that change
  the version field manually — versions are only ever changed by CI.
- Each PR adds **exactly one** changeset (validated by
  `scripts/validate-changeset.mjs` in the `changeset-check` job). Because each PR
  touches a uniquely named file, parallel PRs never produce merge conflicts on
  the version — the central pain point of "everyone bumps package.json".
- Changeset is **not required** for docs-only PRs (markdown / `docs/` changes)
  or PRs from automated `changeset-release/*` branches.

### 2.2 What triggers a release

Releases run only in the `release` job of `release.yml`, gated by:

```yaml
if: !cancelled()
    && github.ref == 'refs/heads/main'
    && github.event_name == 'push'
    && needs.lint.result == 'success'
    && needs.test.result == 'success'
```

So: **a push to `main` (i.e. a merged PR), after lint + the full test matrix
pass.** The release job then:

1. `scripts/setup-npm.mjs` — upgrades npm to `>= 11.5.1` (needed for OIDC
   trusted publishing; Node 20 ships npm 10).
2. `scripts/check-changesets.mjs` — outputs `has_changesets` + `changeset_count`.
3. `scripts/check-release-needed.mjs` — self-healing check: detects whether the
   current `package.json` version is already published on npm. If a previous run
   bumped the version but failed to publish, this lets a re-run finish the
   publish without re-bumping (`should_release` + `skip_bump`).
4. If `changeset_count > 1`: `scripts/merge-changesets.mjs` combines them — the
   **highest bump type wins** (major > minor > patch) and all descriptions are
   preserved chronologically.
5. If changesets exist: `scripts/version-and-commit.mjs --mode changeset` runs
   `changeset version` (bumps `package.json`, updates `CHANGELOG.md`) and
   commits the result directly back to `main`.
6. `scripts/publish-to-npm.mjs --should-pull` — runs `changeset publish` via npm
   OIDC trusted publishing. Multi-layer failure detection scans output for
   `npm error code E*`, `404/401/403`, "failed to publish" etc., and *verifies*
   with `npm view <pkg>@<version> version` before declaring success. Idempotent
   on re-runs.
7. `scripts/create-github-release.mjs` — creates the GitHub Release for the new
   version tag (uses `GH_TOKEN`).
8. `scripts/format-github-release.mjs` — rewrites the release notes into the
   project's preferred format (links commits, PRs, etc.).

The job exposes outputs `published` and `published_version`, consumed
downstream by the optional Docker publish job.

### 2.3 npm trusted publishing (OIDC)

- No npm token is stored. Publishing uses **npm OIDC trusted publishing**, which
  is why the `release`/`instant-release` jobs declare `permissions: id-token:
  write` (plus `contents: write`, `pull-requests: write`).
- **Critical constraint documented in the workflow header:** npm only allows
  *one* workflow file to be registered as a trusted publisher. That is why
  *every* publishing path (automatic release, manual instant release, manual
  changeset PR) is consolidated into `release.yml` rather than split across
  files. A PHP equivalent publishing to Packagist would not need this, but the
  pattern of "one workflow owns publishing" is worth keeping.

### 2.4 Manual release modes (`workflow_dispatch`)

Two manual modes via the workflow's `workflow_dispatch` inputs
(`release_mode` = `instant` | `changeset-pr`, plus `bump_type` and an optional
`description`):

- **`instant-release` job** — `version-and-commit.mjs --mode instant
  --bump-type <patch|minor|major>` then publishes immediately. Bypasses the
  changeset accumulation; useful for hotfixes.
- **`changeset-pr` job** — generates a changeset file
  (`create-manual-changeset.mjs`), Prettier-formats it, and opens a PR via
  `peter-evans/create-pull-request@v8` for human review. Merging that PR then
  flows through the normal automatic release.

---

## 3. Workflow-by-workflow detail

### 3.1 `release.yml` — "Checks and release"

**Triggers:** `push` to `main`; `pull_request` (opened/synchronize/reopened);
`workflow_dispatch` (with the inputs above).

**Concurrency:**
```yaml
group: ${{ github.workflow }}-${{ github.ref }}
cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
```
PR branches cancel stale runs (save CI minutes); `main` runs are **never**
cancelled by a newer push, so an in-flight publish/release is not interrupted.

**Jobs (fast-fail ordering — fast checks gate the slow test matrix):**

| Job                      | Trigger condition                                  | Timeout | Purpose |
| ------------------------ | -------------------------------------------------- | ------- | ------- |
| `detect-changes`         | non-dispatch                                       | 5 min   | `scripts/detect-code-changes.mjs` sets outputs (`mjs-changed`, `js-changed`, `package-changed`, `docs-changed`, `workflow-changed`, `any-code-changed`) used to skip irrelevant jobs. |
| `test-compilation`       | push, or mjs/js changed                            | 5 min   | `node --check` syntax of all `.mjs` (`scripts/check-mjs-syntax.sh`). ~7s. |
| `check-file-line-limits` | push, or code/docs/workflow changed                | 5 min   | Enforces **1500-line limit** on `.js/.mjs/.cjs`, `.md`, and `release.yml` (`scripts/check-file-line-limits.sh`). |
| `version-check`          | PR only                                            | 5 min   | Blocks manual `package.json` version edits (`scripts/check-version.mjs`). |
| `changeset-check`        | PR with `any-code-changed`                         | 10 min  | Validates exactly ONE new changeset added by this PR (`scripts/validate-changeset.mjs`). Skips `changeset-release/*` branches. |
| `lint`                   | push, or code/package/docs/workflow changed        | 10 min  | ESLint, Prettier `--check`, jscpd duplication, secretlint. Runs independently of changeset-check. |
| `test`                   | after fast checks succeed/skip                     | 10 min  | **Matrix: {node, bun, deno} × {ubuntu, macos, windows} = 9 combos**, `fail-fast: false`. |
| `validate-docs`          | push, or docs changed                              | 5 min   | Confirms required docs exist (README, CHANGELOG, CONTRIBUTING, BEST-PRACTICES). |
| `release`                | push to main, lint+test success                    | 30 min  | Versioning + npm publish + GitHub release (see §2.2). `id-token: write`. |
| `instant-release`        | dispatch, `release_mode == instant`                | 30 min  | Manual immediate publish. `id-token: write`. |
| `changeset-pr`           | dispatch, `release_mode == changeset-pr`           | 10 min  | Opens a changeset PR for review. |
| `docker-publish`         | after (instant-)release published                  | 30 min  | Optional Docker Hub publish (see §4). `contents: read`. |

**Fresh-merge simulation:** on PRs, `lint`, `check-file-line-limits`, and `test`
run `scripts/simulate-fresh-merge.sh` first — it fetches the base branch and
merges it into the PR branch so checks run against the *actual* post-merge state,
not a stale GitHub merge preview (documented in case-study issue-23). Requires
`fetch-depth: 0`.

**Cancellation correctness:** downstream jobs use `!cancelled() && ...success`
rather than `always()`. `always()` evaluates true even on cancellation, leaking
jobs through; `!cancelled()` propagates cancellation properly (case-study
issue-25 / hive-mind #1278). Also `needs.X.result == 'skipped'` is explicitly
accepted so docs-only PRs (which skip changeset-check) still run tests.

**Runtimes/actions used:** `actions/checkout@v6`, `actions/setup-node@v6`
(Node `24.x`), `oven-sh/setup-bun@v2`, `denoland/setup-deno@v2` (`v2.x`),
`peter-evans/create-pull-request@v8`. Test commands:
`node --test --test-timeout=30000 tests/*.test.js`, `bun test --timeout 30000`,
`deno test --allow-read`.

### 3.2 `links.yml` — Broken Link Checker

**Triggers:** `push`/`pull_request` filtered to `**.md`, `**.html`, and the
workflow file itself; plus `workflow_dispatch`. (Note: README claims weekly
scheduled runs + auto issue creation, but the committed workflow has **no
`schedule` trigger and no issue-creation step** — that is aspirational/docs
drift in this snapshot.)

**Single job `link-checker`** (`timeout 10 min`, `permissions: contents: read`):

1. `lycheeverse/lychee-action@v2` scans `./**/*.md` and `./**/*.html` with
   `--cache --max-cache-age 1d --max-retries 3 --timeout 30`, excluding
   `docs/case-studies` (external references that legitimately don't resolve).
   `fail: false` so the workflow can do a fallback check; writes `lychee/out.md`
   and a job summary.
2. If lychee found broken links, `scripts/check-web-archive.mjs` looks each one
   up in the **Wayback Machine** and emits `::notice::` annotations suggesting
   archive.org replacements.
3. Final step fails the job only if broken links exist **and** no Web Archive
   fallback is available, printing actionable remediation guidance.

`.lycheeignore` holds regex patterns for known false positives: localhost,
`example.com`, `npmjs.com` (403s bots), `medium.com` (403), `web.archive.org`
(502).

### 3.3 `example-app.yml` — Example app build / Pages deploy / preview regen

**Triggers:** `push` to `main` and `pull_request`, path-filtered to
`examples/universal-app/**`, `src/**`, `package*.json`,
`scripts/update-preview-images.mjs`, and the workflow file; plus
`workflow_dispatch`. Top-level `permissions: contents: read, pages: write,
id-token: write`. Same concurrency rule as release.yml.

Jobs:

- **`web-build`** (10 min): Node 24, npm cache keyed on the example app's
  lockfile. Builds the Vite app (`npm run example:web:build`) with
  `GITHUB_PAGES` + `VITE_REPOSITORY_URL` env. Uploads `dist` as both a normal
  artifact and (on main push) a Pages artifact. Configures Pages only on main.
- **`pages-deploy`** (main push only): `actions/deploy-pages@v5` into the
  `github-pages` environment. Requires a **one-time manual** repo setting
  (Settings → Pages → Source = GitHub Actions) — can't be set from a workflow.
- **`desktop-package`** (matrix ubuntu/macos/windows, 20 min): packages the
  Electron app. **Pinned to Node `20.x`** because Electron Forge packaging exits
  early under Node 24 in CI — a documented runtime-compat workaround. Includes a
  30×1s retry loop polling for `out/` output before failing.
- **`android-build`** / **`ios-build`**: gated behind `workflow_dispatch` AND
  repo variables `EXAMPLE_APP_ENABLE_ANDROID_BUILD` / `..._IOS_BUILD == 'true'`.
  Capacitor + Java/Android SDK (Android) or macOS runner (iOS).
- **`preview-regen`** (main push or dispatch, 20 min): runs in the official
  `mcr.microsoft.com/playwright:v1.59.1-noble` container (browser preinstalled,
  avoids download stalls). Uses `browser-commander` + Playwright via
  `scripts/update-preview-images.mjs` to regenerate
  `docs/screenshots/example-app/*.png`. If screenshots drift, it commits them
  back to `main` as `github-actions[bot]` with a `[skip ci]` message to avoid an
  infinite loop. Uploads failure artifacts on error. `permissions: contents:
  write`, checks out `ref: main` with the GITHUB_TOKEN.

---

## 4. `publish-dockerhub` composite action

`.github/actions/publish-dockerhub/action.yml` — a composite action (not a job)
invoked by the `docker-publish` job in `release.yml`.

**Inputs:** `context` (default `.`), `file` (default `./Dockerfile`), `image`
(required, `namespace/image`), `username` (required), `token` (required),
`version` (required, the published npm version).

**Steps:** `docker/setup-buildx-action@v4` → `docker/login-action@v4` →
`docker/metadata-action@v6` (tags `latest` and `<version>`, label
`org.opencontainers.image.version`) → `docker/build-push-action@v7`
(`push: true`, passes `NPM_PACKAGE_VERSION=<version>` as a build-arg so the
Dockerfile can install the just-published npm package).

**Enablement / gating in `release.yml`'s `docker-publish` job:**

- Entirely optional and **off by default**. Enabled only when repo
  **variable** `DOCKERHUB_IMAGE` is set (plus `DOCKERHUB_USERNAME` var and
  `DOCKERHUB_TOKEN` secret; optional `DOCKER_CONTEXT` / `DOCKERFILE` vars).
- Runs only after `release` or `instant-release` succeeded with
  `published == 'true'`.
- `scripts/check-docker-publish.mjs` decides `enabled` and resolves
  context/dockerfile/image.
- `scripts/wait-for-npm.mjs` **blocks until the exact published version is
  visible on the npm registry** before building, so a Docker image that installs
  the npm package can't be built against a not-yet-propagated version.

---

## 5. Linting, formatting, pre-commit, duplication, secrets, links

### ESLint (`eslint.config.js`, flat config, ESLint 9)
- Base: `@eslint/js` recommended + `eslint-config-prettier` +
  `eslint-plugin-prettier` (`prettier/prettier: error`).
- Targets `**/*.{js,mjs,cjs}`; globals for Node + `Bun`/`Deno` + browser globals
  scoped to the example app.
- Quality rules: `no-unused-vars: error` (**strict — no `_`-prefix exceptions**),
  `eqeqeq`, `curly`, `no-var`, `prefer-const`, `prefer-arrow-callback`,
  `no-duplicate-imports`, `prefer-template`, `object-shorthand`,
  `no-async-promise-executor`, `require-await` (warn; off in tests).
- **Maintainability thresholds:** `complexity` ≤ 15 (warn), `max-depth` ≤ 5,
  `max-lines-per-function` ≤ 150, `max-params` ≤ 6, `max-statements` ≤ 60, and
  **`max-lines: ['error', 1500]`** — the headline file-size rule.
- Ignores `node_modules`, build/dist/out, `*.min.js`, and
  `docs/case-studies/*/data/**` (verbatim external data).

### Prettier (`.prettierrc`)
`semi: true`, `singleQuote: true`, `trailingComma: es5`, `printWidth: 80`,
`tabWidth: 2`, `arrowParens: always`, `endOfLine: lf`. `.prettierignore` exists.
CI runs `prettier --check .` (`npm run format:check`).

### jscpd — copy/paste detection (`.jscpd.json`)
`threshold: 0` (**zero tolerance — any duplication fails**), `minTokens: 30`,
`minLines: 5`, `skipComments: true`. Ignores node_modules, build/dist, min.js,
`.changeset`, case-study data, lockfiles. Console + HTML reporters. Run via
`npm run check:duplication` in the `lint` job.

### secretlint (`.secretlintrc.json`)
Single rule preset `@secretlint/secretlint-rule-preset-recommend`. CI runs
`npx --yes -p secretlint -p @secretlint/...-recommend secretlint "**/*"` in the
`lint` job to catch committed credentials before review.

### Pre-commit (Husky + lint-staged)
- `.husky/pre-commit` → `npx lint-staged`. Husky installed via `prepare`:
  `husky || true`.
- `lint-staged` config in `package.json`:
  - `*.{js,mjs,cjs}` → `eslint --fix --max-warnings 0 --no-warn-ignored`,
    `prettier --write`, `prettier --check`.
  - `*.md` → `prettier --write`, `prettier --check`.

### lychee link checking
See §3.2. Config inline in `links.yml`; ignore patterns in `.lycheeignore`.

### `package.json` scripts of note
`test` (node test, 30s/test cap), `lint`/`lint:fix`, `format`/`format:check`,
`check:duplication`, aggregate `check`, the `example:*` app scripts, and the
changeset scripts (`changeset`, `changeset:version`, `changeset:publish`,
`changeset:status --since=origin/main`). License: **Unlicense (public domain)**.
`engines.node >= 20`. `publishConfig.access: public`.

---

## 6. `docs/BEST-PRACTICES.md` summary

Frames CI/CD as the feedback loop that *forces* AI solvers to iterate until all
gates pass. Enumerates 14 best practices (adapted from the `hive-mind` project):

1. **File size limit 1500 lines** (ESLint `max-lines` + CI) — fits AI context
   windows, forces modularity.
2. **Automated formatting** — ESLint + Prettier + Husky.
3. **Static analysis** — strict unused-vars, async/await rules.
4. **Comprehensive testing** — cross-runtime (Node/Bun/Deno) × cross-OS, via
   `test-anywhere`.
5. **Changeset versioning** — no version merge conflicts, automated
   bumps/changelogs, highest-bump-wins on merge.
6. **Pre-commit hooks** — local gates before CI.
7. **Release automation** — no manual versioning, OIDC trusted publishing,
   validated-only releases, dual (auto + manual) triggers, optional Docker.
8. **Pipeline features** — concurrency control (main never cancelled, PRs
   cancel stale) + fresh-merge simulation.
9. **Fast-fail job ordering** — fast checks (~7-30s) gate the slow 9-cell test
   matrix so a syntax error fails in seconds.
10. **File line limits in CI** — separate shell check covering JS + Markdown +
    `release.yml` (broader than ESLint's lint scope).
11. **Secrets detection** — secretlint in the lint job.
12. **Documentation validation** — required-files + size check, only on docs
    changes.
13. **Reasonable timeouts on every job/test** — explicit `timeout-minutes`
    ~5-10× typical runtime (table: 5 min fast checks, 10 min lint/test/links,
    30 min release jobs), plus per-test 30s budgets where the runner supports it
    (Deno relies on the 10-min job cap).
14. **Proper cancellation propagation** — `!cancelled()` not `always()`.

Closes with a defense-in-depth diagram (developer machine → CI fast checks →
slow checks → docs validation → optional Docker → release) and references to the
hive-mind CI/CD best-practices doc and the `docs/case-studies/` analyses.

---

## 7. The "AI-driven development" conventions (overall philosophy)

- **CI as the contract for AI agents.** Every quality rule is machine-enforced
  in CI so an autonomous solver gets an objective pass/fail signal and iterates
  until green. Nothing relies on a human remembering a convention.
- **Small, context-window-sized files.** The hard 1500-line limit (enforced
  three ways: ESLint, a CI shell check covering more file types, and within the
  workflow file itself) so an AI can load a whole file at once.
- **Thin YAML, logic in tested scripts.** Workflow YAML orchestrates; the real
  logic is in `scripts/*.mjs` with unit tests in `tests/`, so behavior is
  verifiable and the workflow stays under the line limit.
- **Conflict-free parallel changes.** Changesets give every PR an independent
  versioning file; PR validation only inspects the changeset *that PR added*, so
  many AI-generated PRs can merge in any order without versioning conflicts.
- **Self-healing releases.** `check-release-needed` + idempotent publish verify
  npm state and finish an interrupted release on re-run, instead of failing or
  double-publishing — important when runs are flaky/cancelled.
- **Fast feedback first.** Fast-fail ordering + per-job timeouts minimize the
  cycle time of the AI's edit→CI→fix loop.
- **Reproducibility / determinism.** Multi-runtime + multi-OS matrix, pinned
  action versions, fresh-merge simulation, and `[skip ci]` bot commits for
  generated artifacts (preview images) to avoid loops.
- **Decisions are documented as case studies.** `docs/case-studies/issue-*/`
  (not read in detail here) record the concrete upstream issue/PR that motivated
  each pipeline choice (e.g. issue-23 fresh-merge, issue-25 concurrency &
  cancellation, issue-36 self-healing publish), so future agents understand
  *why* a rule exists.

---

## 8. Notes for the PHP/Symfony + Composer port

- **Versioning:** Composer/Packagist releases are git-tag driven, so the
  Changesets mechanism maps to either a Changesets-style accumulator or
  conventional-commit/semantic-release tooling that produces a tag + changelog.
  Keep the "no manual version edits; CI owns versioning; highest-bump-wins;
  one-changeset-per-PR" rules.
- **Publishing:** Packagist auto-updates from a tag/webhook, so there's no direct
  analog to npm OIDC trusted publishing — the "create GitHub Release from tag"
  + "one workflow owns release" patterns still apply. Docker publishing maps
  directly.
- **Quality gates → PHP toolchain:** ESLint→PHPStan/Psalm + PHP_CodeSniffer;
  Prettier→PHP-CS-Fixer/`php-cs-fixer`; jscpd→`phpcpd`/`jscpd` (language-agnostic);
  secretlint and lychee are language-agnostic and port as-is; Husky/lint-staged
  → `captainhook`/`grumphp` or a plain git hook running the formatters.
- **Testing matrix:** replace {Node,Bun,Deno}×OS with {PHP versions}×OS (and/or
  Symfony LTS vs latest) under PHPUnit.
- **Keep:** the 1500-line limit + CI line-limit check, fast-fail ordering,
  explicit per-job timeouts, `!cancelled()` conditions, concurrency policy
  (main never cancelled), fresh-merge simulation, thin-YAML/tested-scripts,
  required-docs validation, and the case-study documentation discipline.
