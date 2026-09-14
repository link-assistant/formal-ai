# Analysis: `link-foundation/csharp-ai-driven-development-pipeline-template`

A detailed reverse-engineering of the C# AI-driven-development pipeline template,
written to inform building an equivalent **PHP / Symfony + Composer** template.

Default branch: `main`. Package under release: `MyPackage` (NuGet id `MyPackage`,
current version `0.3.2`). Target framework: .NET 8.0.

Raw copies of every workflow and script analyzed here live next to this file
under `csharp/` (preserving relative paths).

---

## 0. Mental model / TL;DR

This template applies a **JavaScript-Changesets-style release flow to a C#/NuGet
package**, but implements all the release glue as small, unit-tested **Node ESM
(`.mjs`) scripts run with Bun**, not C# code. The `.cs` source is just a trivial
demo library (`Calculator`, `PackageInfo`). The real product of the template is
the **pipeline**: change detection, changeset validation, multi-OS testing,
strict linting, NuGet publish with self-healing, and DocFX → GitHub Pages docs.

Key cross-language design pillars (these are what to port to PHP/Symfony):

1. **Changeset-driven versioning** — each PR adds exactly one `.changeset/*.md`
   file declaring `major|minor|patch` + a description; merging to `main`
   collects/merges them, bumps the version, updates the changelog, tags, and
   publishes.
2. **Self-healing releases** — the registry (NuGet) + GitHub Releases API are the
   source of truth, *not* git tags. A failed publish is automatically resumed on
   the next push to `main` without a new changeset.
3. **Wait for the registry to index** before cutting the GitHub release, so the
   version named in the release notes is actually installable.
4. **Logic lives in tested helper scripts**, invoked from thin workflow steps.
   The workflow YAML mostly wires `if:` conditions and step outputs together.
5. **Strict quality gates**: warnings-as-errors build, formatter verification,
   a max-file-size check, cross-OS test matrix, coverage upload.
6. **Docs auto-publish** to GitHub Pages on every push to `main` (not gated on a
   release tag), so a fresh repo serves docs immediately.

---

## 1. Repository layout

```
.changeset/            config.json + README.md + per-PR *.md changesets
.github/workflows/     docs.yml, release.yml
docs/                  DocFX conceptual content (index.md, toc.yml, roadmap/)
docfx.json             DocFX project config
examples/              BasicUsage console app (demo only)
scripts/               14 .mjs files: 8 logic scripts + 6 *.test.mjs suites
src/MyPackage/         Calculator.cs, PackageInfo.cs, MyPackage.csproj
tests/MyPackage.Tests/ xUnit tests + csproj
Directory.Build.props  shared MSBuild props (analyzers, warnings-as-errors)
.editorconfig          227 lines of C# style + naming rules
.pre-commit-config.yaml
MyPackage.sln
CHANGELOG.md  CONTRIBUTING.md  README.md  LICENSE (Unlicense)
```

The template is explicitly modeled on the sibling JS / Python / Rust templates
under `link-foundation` (the README "Acknowledgments" and many code comments
reference `js-ai-driven-development-pipeline-template`). Many design notes cite
specific issues (#9, #11, #13, #15) that motivated each piece of hardening — a
useful provenance trail.

---

## 2. The release pipeline

### 2.1 Versioning model

- **Source of version truth in-repo:** the `<Version>` element in
  `src/MyPackage/MyPackage.csproj` (currently `0.3.2`). Scripts parse/rewrite it
  with regex (`<Version>(\d+)\.(\d+)\.(\d+)</Version>`).
- **Semantic versioning**, bumped from changeset `major|minor|patch`:
  - major → `M+1.0.0`
  - minor → `M.m+1.0`
  - patch → `M.m.p+1`
- **Git tag scheme is dual:**
  - `version-and-commit.mjs` creates an annotated git tag `v<version>` (e.g.
    `v0.3.3`) and pushes it (used as an in-repo "already released" guard).
  - The **GitHub Release** uses a *language-prefixed* tag `csharp_v<version>`
    (e.g. `csharp_v0.3.3`), created by `create-github-release.mjs`. The prefix
    lets a multi-language monorepo/family namespace releases per language.

### 2.2 The `.changeset` mechanism

`.changeset/config.json` (mirrors `@changesets/cli` schema but only a subset is
honored, because versioning is done by the custom scripts, not the changesets
CLI):

```json
{
  "$schema": "https://unpkg.com/@changesets/config@3.1.1/schema.json",
  "changelog": false,
  "commit": false,
  "fixed": [],
  "linked": [],
  "access": "public",
  "baseBranch": "main",
  "updateInternalDependencies": "patch",
  "ignore": []
}
```

A changeset file is markdown with YAML-ish frontmatter:

```markdown
---
'MyPackage': patch
---

Description of the change.
```

Rules enforced (by `validate-changeset.mjs` in CI):
- Exactly **one** changeset added per PR (more → fail; zero → fail).
- Must declare a valid bump type for the `'MyPackage'` package name.
- Must have a non-empty description after the closing `---`.
- Exemptions: automated release branches (`changeset-release/*`,
  `changeset-manual-release-*`) skip the check; docs-only PRs skip it because
  the change detector marks them as non-code (see §4.1).

`README.md` files and `config.json` are never treated as changesets. The package
name `MyPackage` is **hardcoded** in three scripts (`version-and-commit.mjs`,
`merge-changesets.mjs`, `validate-changeset.mjs`) and must be edited when
adopting the template — a notable papercut to fix in the PHP port (derive it
from `composer.json`'s `name` instead).

### 2.3 What triggers a release

`release.yml` (workflow name "CI/CD Pipeline") triggers on:
- `push` to `main` → automatic changeset release (`release` job).
- `workflow_dispatch` with `release_mode=instant` → `instant-release` job.
- `workflow_dispatch` with `release_mode=changeset-pr` → `changeset-pr` job
  (opens a PR containing a changeset for review rather than releasing directly).
- `pull_request` (opened/synchronize/reopened) → validation only, no release.

The `release` job runs only when: `github.ref == refs/heads/main` AND
`event == push` AND `lint`, `test`, `build` all succeeded.

### 2.4 Release job step sequence (automatic, changeset mode)

1. **Check for changesets** — counts `.changeset/*.md` minus `README.md`; sets
   `has_changesets`, `changeset_count`.
2. **Check if release is needed** (`check-release-needed.mjs`) — the self-healing
   gate (see §3.3). Outputs `should_release`, `skip_bump`, `current_version`,
   `nuget_published`, `github_release_exists`.
3. **Merge changesets** (`merge-changesets.mjs`) — only if `changeset_count > 1`.
4. **Version and commit** (`version-and-commit.mjs --mode changeset`) — bumps
   csproj, updates `CHANGELOG.md`, deletes changeset files, commits, tags
   `v<version>`, pushes commit + tag. Outputs `version_committed`,
   `new_version`, `already_released`.
5. **Resolve release version** — chooses `version.new_version` if present, else
   `check_release.current_version` (self-healing path where no bump happened).
6. **Build release package** — `dotnet restore && build -c Release && pack`.
7. **Resolve NuGet package id** — via `dotnet msbuild -getProperty:PackageId`,
   falling back to `AssemblyName`, then literal `MyPackage`.
8. **Validate NuGet API key** — warns + soft-skips if `NUGET_API_KEY` unset
   (so the rest of the release still runs); otherwise prints key length.
9. **Publish to NuGet** — `dotnet nuget push ./artifacts/*.nupkg --api-key ...
   --source https://api.nuget.org/v3/index.json --skip-duplicate`. Sets
   `published=true|false`.
10. **Wait for NuGet indexing** (`wait-for-nuget.mjs`) — only when published;
    polls the flat-container nuspec endpoint (see §3.4).
11. **Create GitHub Release** (`create-github-release.mjs`) — tag prefix
    `csharp_v`, language `C#`, attaches `./artifacts/*.nupkg`, appends a NuGet
    badge, body sourced from the matching `CHANGELOG.md` section.

The gating `if:` on steps 5–11 is the compound condition:
`version_committed == true OR already_released == true OR
(check_release.should_release == true AND check_release.skip_bump == true)` —
i.e. either a fresh bump happened, or the version was already tagged, or the
self-healing path wants to resume.

The **`instant-release`** job is nearly identical but calls
`version-and-commit.mjs --mode instant --bump-type <x> --description <d>` and has
no changeset/merge/`check-release-needed` steps (no self-heal).

### 2.5 `changeset-pr` job

For `workflow_dispatch` with `release_mode=changeset-pr`: writes
`.changeset/manual-release-<epoch>.md` with the chosen bump type/description and
opens a PR (`peter-evans/create-pull-request@v8`, branch
`changeset-manual-release-<run_id>`). Merging that PR triggers the normal
automatic release path.

---

## 3. Scripts in `scripts/` (all Node ESM `.mjs`, run with Bun)

All scripts write GitHub step outputs via `appendFileSync(process.env.GITHUB_OUTPUT, ...)`,
echo to stdout for logs, and use a shared "is this invoked directly vs imported"
guard so the pure functions can be unit-tested without running `main()`. Tests
use `bun:test` (Bun's built-in Jest-like runner) and a local `node:http` mock
server for any network-touching logic — no real HTTP, no real registry.

| Script | Unit tested? | Purpose |
|---|---|---|
| `detect-code-changes.mjs` | no | classify changed files → workflow outputs |
| `validate-changeset.mjs` | no | enforce one valid changeset per PR |
| `merge-changesets.mjs` | no | combine N changesets → 1 |
| `version-and-commit.mjs` | **yes** (`version-and-commit.test.mjs`) | bump, changelog, commit, tag, push |
| `bump-version.mjs` | no | standalone version bump utility (dev/manual) |
| `check-release-needed.mjs` | **yes** (`check-release-needed.test.mjs`) | self-healing release gate |
| `wait-for-nuget.mjs` | **yes** (`wait-for-nuget.test.mjs`) | poll NuGet until version is indexed |
| `create-github-release.mjs` | **yes** (`create-github-release.test.mjs`) | create GH release + upload assets |
| `check-file-size.mjs` | no | fail if any `.cs` file > 1000 lines |
| — | `release-workflow-policy.test.mjs` | asserts invariants on `release.yml` itself |

### 3.1 `detect-code-changes.mjs`

- **Inputs (env):** `GITHUB_EVENT_NAME`, `GITHUB_BASE_SHA`, `GITHUB_HEAD_SHA`.
- **Logic:** computes the changed file list. For PRs: `git diff --name-only
  baseSha headSha` (fetching the base commit if missing). For push: `HEAD^..HEAD`,
  falling back to `git ls-tree` on the first commit.
- **Outputs:** booleans `cs-changed`, `csproj-changed`, `sln-changed`,
  `props-changed`, `mjs-changed`, `docs-changed`, `workflow-changed`, and
  `any-code-changed`.
- **Key rule:** `any-code-changed` *excludes* markdown files anywhere and the
  `.changeset/`, `docs/`, `experiments/`, `examples/` folders. This is what lets
  docs-only PRs skip the changeset requirement. The "code" file pattern is
  `.(cs|csproj|sln|props|mjs|json|yml|yaml)$` or `.github/workflows/`.
- **PHP port:** swap extensions to `.php`, `composer.json`, `composer.lock`,
  `phpunit.xml`, `*.yaml`, etc.

### 3.2 `validate-changeset.mjs`

- **Inputs (env):** `GITHUB_BASE_SHA`/`GITHUB_HEAD_SHA` (preferred),
  `GITHUB_BASE_REF` (fallback), else "all changesets in dir" fallback for local.
- **Logic:** uses `git diff --name-status` to find files **added** (`A` status)
  under `.changeset/*.md` (excluding `README.md`). Requires exactly one; then
  validates it declares `'MyPackage': major|minor|patch` and has a non-empty
  description. Emits `::error::` annotations and exits non-zero on failure.
- **Note:** the validator regex requires *quoted* package names
  (`'MyPackage'`/`"MyPackage"`), while the version/merge scripts also accept
  unquoted. The README/CONTRIBUTING examples all use quotes.

### 3.3 `check-release-needed.mjs` (self-healing gate) — exported & tested

- **Why it exists (issue #11):** a previous release pushed a version commit + git
  tag, but `dotnet nuget push` returned HTTP 403, so the package never appeared
  publicly. Re-runs then short-circuited on the existing tag and never retried.
  **Therefore this script ignores git tags entirely** and checks the real
  registries.
- **Inputs:** CLI `--csproj --repository --tag-prefix --package-id`; env
  `HAS_CHANGESETS`, `GITHUB_REPOSITORY`, `GH_TOKEN`/`GITHUB_TOKEN`, plus test
  overrides `NUGET_INDEX_URL`, `GITHUB_API_URL`.
- **Probes:**
  - NuGet flat-container index: `GET {base}/{id-lower}/index.json`. Returns the
    version array; `null` if 404 (package id unregistered).
  - GitHub release-by-tag: `GET /repos/{repo}/releases/tags/{csharp_v<ver>}`
    → exists if 200, missing if 404.
- **Pure `decide()` function (the testable core):**
  - `hasChangesets` → normal release path (`shouldRelease`, no `skipBump`).
  - else if current version not on NuGet → `shouldRelease + skipBump`
    (self-heal; resume publish without a new changeset).
  - else if on NuGet but no GitHub release → `shouldRelease + skipBump`
    (self-heal release creation only).
  - else → `shouldRelease=false` (no-op).
- **Outputs:** `should_release`, `skip_bump`, `current_version`,
  `nuget_published`, `github_release_exists`, `reason`; plus a
  `GITHUB_STEP_SUMMARY` markdown block.
- **PHP port:** the registry is **Packagist**. There is no equivalent of
  "publish a tarball then poll an index" — Packagist *pulls* from a VCS tag via
  webhook/cron. So the self-heal here maps to: did the git tag get pushed, did
  Packagist pick up the version (`https://repo.packagist.org/p2/<vendor>/<name>.json`),
  and does the GitHub release exist. The "wait for registry indexing" step
  becomes "wait for Packagist to reflect the new tag" (or trigger its update API).

### 3.4 `wait-for-nuget.mjs` — exported & tested

- **Why (issue #13):** the old inline retry (0/5/10/20/30/60s, ~125s total) gave
  up well inside NuGet's documented up-to-15-minute indexing window, so GitHub
  releases were cut before the package was installable.
- **Inputs:** `--package-id`, `--release-version` (alias `--version`);
  `--max-attempts` (default **8**), `--sleep-seconds` (default **120**),
  `--flat-container-url`; env overrides (`PACKAGE_ID`, `RELEASE_VERSION`,
  `NUGET_WAIT_MAX_ATTEMPTS`, `NUGET_WAIT_SLEEP_SECONDS`,
  `NUGET_FLAT_CONTAINER_URL`/`NUGET_INDEX_URL`).
- **Logic:** polls `HEAD {base}/{id-lower}/{ver-lower}/{id-lower}.nuspec`; treats
  HTTP 200 as available, 404/network errors as keep-waiting. Total budget
  8 × 120s ≈ 16 min. Output `nuget_available=true|false`; exit 1 if never
  available.
- **Tested behaviors:** success on the last attempt, exhaustion failure,
  immediate success (no sleeps), transient network-error tolerance, URL casing,
  arg/env parsing, and an end-to-end pass against a local mock that returns
  404,404,200.

### 3.5 `create-github-release.mjs` (`#!/usr/bin/env bun`) — exported & tested

- **Inputs:** `--release-version`/`--version`, `--repository`, `--tag-prefix`
  (default `csharp_v`), `--language` (default `C#`), `--package-id`,
  `--assets-glob` (e.g. `./artifacts/*.nupkg`); env equivalents.
- **Logic / notable helpers (all exported for tests):**
  - `normalizeReleaseVersion` — strips `v`/language prefixes to bare semver
    (handles `csharp_v1.2.3`, `csharp-v1.2.3`, `v1.2.3`, pre-release/build
    metadata).
  - `buildReleaseTag` → `csharp_v<semver>`; `buildReleaseTitle` → `[C#] <semver>`.
  - `extractReleaseNotes` — regex-extracts the matching `## [<version>]` section
    from `CHANGELOG.md`; falls back to `Release <semver>`.
  - `appendNuGetBadgeIfMissing` — appends a shields.io NuGet badge unless one is
    already present.
  - `limitReleaseNotesBytes` — truncates the body to **120 000 UTF-8 bytes**
    (under GitHub's ~125 000 limit, issue from CHANGELOG 0.3.2) without splitting
    multibyte chars, appending a link to the full tagged `CHANGELOG.md`.
  - `findPackageId` — walks `*.csproj` (depth ≤ 4, skipping bin/obj/.git/node_modules)
    to discover `PackageId`/`AssemblyName`.
  - `createRelease` — `gh api repos/<repo>/releases -X POST --input -` with the
    JSON payload; treats `already_exists` as success and reconciles assets.
  - `resolveReleaseAssets` + `uploadReleaseAssets` — simple `*`-glob resolution
    and `gh release upload <tag> ... --clobber --repo <repo>`.
- **PHP port:** identical shape; just change tag prefix to e.g. `php_v`,
  language to `PHP`, the badge to a Packagist badge
  (`https://img.shields.io/packagist/v/<vendor>/<name>.svg`), and skip the
  `.nupkg` asset upload (Composer packages are not built artifacts).

### 3.6 `version-and-commit.mjs` — partially tested

- **Modes:** `--mode changeset` (derive bump from changesets) or `--mode instant
  --bump-type <x> [--description <d>]`.
- **Logic:** reads/bumps `<Version>`; in changeset mode parses each changeset for
  bump type (highest of patch<minor<major wins) and collects descriptions;
  updates `CHANGELOG.md` (inserts a `## [<ver>] - <date>` block before the first
  existing version entry, or creates the file); deletes changeset files; configures
  the `github-actions[bot]` git identity; commits `chore: release v<ver>`; creates
  annotated tag `v<ver>`; `git push` + `git push --tags`.
- **Guard:** `checkTagExists()` queries the exact `refs/tags/v<ver>` ref. If the
  tag already exists → output `already_released=true` and exit without
  re-committing (idempotent re-runs).
- **Outputs:** `version_committed`, `new_version`, `already_released`.
- **Tested (issue #9 regression):** starting `2.3.0` + minor changeset must
  produce `2.4.0` and must *not* falsely report `already_released`; the inverse
  (tag exists → `already_released=true`) is also tested. Tests spin up a real
  temp git repo with a bare remote so `git push` works offline.
- **Bug history captured in CHANGELOG:** the `exec()` helper was fixed to
  propagate failures even in silent mode so a missing tag isn't mistaken for an
  existing one.

### 3.7 `bump-version.mjs`

Standalone `--bump-type <x> [--dry-run]` utility that only rewrites the csproj
`<Version>`. Not wired into the workflow (the release path uses
`version-and-commit.mjs`); it's a convenience/dev tool also listed in the README
scripts table.

### 3.8 `merge-changesets.mjs` (`#!/usr/bin/env bun`)

Combines all `.changeset/*.md` into one when ≥2 exist: picks the highest bump
type, concatenates descriptions in mtime order, writes
`merged-<adjective>-<noun>.md` (changesets-style random name), deletes the
originals. No-op for 0/1 changesets.

### 3.9 `check-file-size.mjs`

Recursively scans for `.cs` files (excluding `bin`/`obj`/`.git`/`node_modules`/
`artifacts`) and fails (exit 1) if any exceeds **1000 lines** — a "keep files
small for AI/humans" guard. Run in the `lint` job and as a documented manual
check. CONTRIBUTING also states "keep files under 1000 lines".

### 3.10 `release-workflow-policy.test.mjs` (meta-test on the YAML)

Parses `release.yml` and asserts pipeline invariants:
- Concurrency does **not** cancel in-progress `main` runs
  (`cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`, never
  `cancel-in-progress: true`).
- Every job has an explicit `timeout-minutes`, matching an expected map
  (detect-changes 5, changeset-check 10, lint 20, test 30, build 20, release 30,
  instant-release 30, changeset-pr 10) — and the set of jobs equals exactly that
  list.
- Pinned action major versions are current: `actions/checkout@v6`,
  `actions/upload-artifact@v7`, `peter-evans/create-pull-request@v8` (and *not*
  the older v4/v4/v7). This is a clever way to lint the workflow itself in CI.

---

## 4. Workflows

### 4.1 `release.yml` — "CI/CD Pipeline"

- **Triggers:** `push: [main]`, `pull_request: [opened, synchronize, reopened]`,
  `workflow_dispatch` (inputs: `release_mode` = instant|changeset-pr,
  `bump_type` = patch|minor|major, `description`).
- **Concurrency:** group `${{ github.workflow }}-${{ github.ref }}`,
  `cancel-in-progress` only when **not** on `main` (protect in-flight releases).
- **Env:** disables dotnet first-run/telemetry/logo.
- **Jobs:**
  | Job | Runs when | Key steps |
  |---|---|---|
  | `detect-changes` | not `workflow_dispatch` | Bun + `detect-code-changes.mjs` → file-type outputs |
  | `changeset-check` | PR **and** `any-code-changed` | skip for release branches, else `validate-changeset.mjs` |
  | `lint` | push/dispatch or any relevant file changed | `dotnet restore`; `dotnet format --verify-no-changes`; `dotnet build /warnaserror`; `bun test scripts/*.test.mjs`; `check-file-size.mjs` |
  | `test` | push/dispatch or changeset-check success/skipped | matrix ubuntu/macos/windows; `dotnet test --collect:"XPlat Code Coverage"`; Codecov upload (ubuntu only) |
  | `build` | lint+test success | `dotnet pack`; upload `*.nupkg` artifact |
  | `release` | push to main + lint/test/build success | full release sequence (§2.4); `permissions: contents:write, packages:write` |
  | `instant-release` | dispatch instant + lint/test/build success | manual immediate release; same perms |
  | `changeset-pr` | dispatch changeset-pr | open changeset PR; `permissions: contents:write, pull-requests:write` |
- **Notable patterns:**
  - `lint` deliberately does **not** depend on `changeset-check` (referenced PR
    #18 in JS template): a fast check should always run.
  - `test` uses `if: always() && (... || changeset-check skipped/success)` so
    docs-only PRs (where `changeset-check` is skipped) still test.
  - `fail-fast: false` on the OS matrix.
  - Tooling: `oven-sh/setup-bun@v2` for the scripts, `actions/setup-dotnet@v4`
    (`8.0.x`) for the build.

### 4.2 `docs.yml` — DocFX → GitHub Pages

- **Triggers:** `push: [main]` and `pull_request: [main]` filtered to
  `docs/**`, `src/**`, `docfx.json`, `.github/workflows/docs.yml`; plus
  `workflow_dispatch`.
- **Permissions:** `contents: read`, `pages: write`, `id-token: write`.
- **Concurrency:** same group pattern; cancel only off-`main`.
- **Jobs:**
  - `build`: setup-dotnet → `dotnet tool update -g docfx` → `dotnet restore` →
    `docfx docfx.json -o _site` → debug tree dump → on push-to-main/dispatch:
    `actions/configure-pages@v6` + `actions/upload-pages-artifact@v5`.
  - `deploy`: gated on push-to-main or dispatch; `environment: github-pages`;
    `actions/deploy-pages@v5`.
- **Design note (issue #15):** deploy is gated on `push`/`dispatch`, **never** on
  `release: published`. Gating on releases would block the first deploy until a
  tag is cut, leaving `<org>.github.io/<repo>/` returning 404. One-time manual
  setup: Settings → Pages → Source = GitHub Actions.

---

## 5. Linting / formatting / pre-commit / editorconfig

### 5.1 Build-enforced quality (`Directory.Build.props`)

Applied to all projects: `net8.0`, `ImplicitUsings`, `Nullable` enable,
`LangVersion latest`, **`TreatWarningsAsErrors=true`**, `WarningLevel 9999`,
`EnforceCodeStyleInBuild=true`, `EnableNETAnalyzers=true`,
`AnalysisLevel=latest-all`. So formatting/style violations *fail the build*.

### 5.2 `.editorconfig` (227 lines)

`root = true`; global 4-space/LF/UTF-8/trim-trailing/final-newline. 2-space for
JSON/YAML/XML/csproj/props and for `*.{js,mjs,ts}`. Markdown exempts
trailing-whitespace trimming. Extensive C# rules: file-scoped namespaces
(warning), expression-bodied members, `var` preferences, pattern-matching
preferences, `using` outside namespace, accessibility modifiers required, plus a
full naming-rules block (interfaces `I`-prefixed, types/members PascalCase,
private fields `_camelCase`, constants PascalCase, locals/params camelCase) —
all at `:warning`, which under warnings-as-errors means CI-blocking.
`IDE0058` is suppressed.

### 5.3 `.pre-commit-config.yaml`

- `pre-commit/pre-commit-hooks@v5.0.0`: trailing-whitespace, end-of-file-fixer,
  check-yaml, check-added-large-files, check-merge-conflict, check-json,
  check-xml.
- Local hooks on `types: [c#]`, `pass_filenames: false`: `dotnet format
  --verify-no-changes`, `dotnet build /warnaserror`, `dotnet test`.

### 5.4 CI lint job

`dotnet format --verify-no-changes --verbosity diagnostic` + warnings-as-errors
build + `bun test scripts/*.test.mjs` + file-size check. So the **script unit
tests run in the lint job**, separate from the C# matrix `test` job.

---

## 6. Docs publishing (DocFX)

- `docfx.json`: **metadata** stage extracts API docs from
  `src/MyPackage/**/*.csproj` into `docs/api` (no private members,
  `allowCompilationErrors:false`, flattened namespaces, same-page members);
  **build** stage takes `docs/**/*.{md,yml}` + `docs/images/**` → `_site` with
  `default` + `modern` templates, search enabled, app name/title `MyPackage`.
- API docs derive from C# XML doc comments (csproj sets
  `GenerateDocumentationFile=true`, suppresses CS1591). CONTRIBUTING mandates
  `///` XML docs on all public APIs.
- `docs/`: `index.md` (landing), `toc.yml` (Home/API/Roadmap),
  `roadmap/preview-regeneration.md` (a deferred-work placeholder documenting a
  Playwright/browser-commander preview-regeneration pattern not yet portable
  here because the template has no rendered web surface — only a console
  `examples/BasicUsage`).
- **PHP port:** DocFX is C#-specific. Equivalent options: phpDocumentor or Doctum
  for API docs, or a generic static-site (MkDocs/Docusaurus). The *pattern* to
  keep: build on every push to `main`, deploy via `actions/deploy-pages`, never
  gate on a release.

---

## 7. Test structure

- **C# tests** (`tests/MyPackage.Tests`): xUnit 2.9.2, Microsoft.NET.Test.Sdk
  17.12.0, coverlet.collector 6.0.2. `IsPackable=false`, `IsTestProject=true`,
  warnings-as-errors with a `NoWarn` list waiving test-idiomatic analyzer rules
  (CA1707 underscores, CA1034 nested types, CA1052, CA1515, CA1849, CA2007).
  - Organized with **nested classes per method-under-test** (`AddTests`,
    `MultiplyTests`, `DelayAsyncTests`), `[Fact]` + `[Theory]/[InlineData]`,
    Arrange/Act/Assert. CONTRIBUTING documents this convention.
  - `PackageInfoTests` asserts the version string is non-empty and semver-shaped.
  - Run cross-OS with coverage; Codecov upload on ubuntu (`fail_ci_if_error:false`).
- **Script tests** (`scripts/*.test.mjs`): `bun:test`. Pattern: export pure
  functions + a direct-invocation guard; test pure logic directly and network/
  registry logic against an in-process `node:http` mock (overridable base URLs).
  `version-and-commit.test.mjs` builds a real temp git repo + bare remote to
  exercise commit/tag/push offline. `release-workflow-policy.test.mjs` asserts
  invariants on the YAML itself.

---

## 8. The "AI-driven development" conventions

The template encodes guardrails that make a codebase safe for autonomous/AI
contributors and reproducible CI:

1. **One changeset per PR, validated in CI** — every change self-documents its
   semver impact and changelog entry; releases need no human version bookkeeping.
2. **Small files** — hard 1000-line cap on `.cs` (enforced), "keep functions
   focused" in CONTRIBUTING; keeps units within model context windows.
3. **Strict, build-failing quality** — warnings-as-errors + analyzers +
   formatter verification + editorconfig at warning severity, so an AI can't
   merge subtly-wrong style/quality.
4. **Self-healing, idempotent releases** — registry/release APIs are the source
   of truth (not tags); failed publishes auto-resume; `--skip-duplicate`,
   `already_exists` reconciliation, and tag-existence guards make re-runs safe.
5. **Logic in tested helper scripts, thin YAML** — every non-trivial release
   decision (`decide()`, version math, release-notes building, NuGet polling) is
   a pure, unit-tested function; even the workflow YAML is asserted by a test.
6. **Cross-platform reproducibility** — multi-OS matrix, pinned action major
   versions, explicit job timeouts, telemetry disabled.
7. **Docs always live** — API docs auto-publish from XML comments on every
   `main` push; XML docs required on public APIs.
8. **Provenance in comments** — fixes reference the exact issue numbers (#9, #11,
   #13, #15) and sibling-template PRs that motivated them, so an AI editor
   understands *why* a guard exists before removing it.
9. **Multiple release ergonomics** — automatic (changeset), manual instant, and
   manual changeset-PR modes via `workflow_dispatch`.
10. **Explicit "adopt this template" checklist** in README/CONTRIBUTING (rename
    package, update csproj metadata, fix hardcoded `MyPackage` names).

---

## 9. Porting notes for PHP / Symfony + Composer

| C# template element | PHP/Symfony equivalent |
|---|---|
| `<Version>` in `.csproj` | git tag is canonical for Composer; optionally a `version` in `composer.json` (usually omitted — Packagist derives from tags) |
| NuGet publish (`dotnet nuget push`) | push a git tag; Packagist auto-imports (webhook or `/api/update-package`) |
| `check-release-needed.mjs` NuGet probe | Packagist probe `https://repo.packagist.org/p2/<vendor>/<name>.json` for the version |
| `wait-for-nuget.mjs` | poll Packagist p2 metadata until the new version appears (or trigger + verify) |
| `dotnet format` / analyzers / editorconfig | PHP-CS-Fixer or PHP_CodeSniffer + PHPStan/Psalm; `phpstan.neon`, `.php-cs-fixer.php` |
| warnings-as-errors build | `phpstan analyse --level max` + `php-cs-fixer fix --dry-run` failing CI |
| xUnit + coverlet | PHPUnit + Xdebug/PCOV coverage; Codecov upload unchanged |
| DocFX → Pages | phpDocumentor/Doctum (or MkDocs) → `actions/deploy-pages`, same non-release gating |
| `setup-dotnet` + `setup-bun` | `shivammathur/setup-php` (+ keep Bun/Node for the same `.mjs` scripts, or rewrite scripts in PHP) |
| `.nupkg` artifact upload to GH release | skip (Composer has no build artifact); keep changelog-sourced notes + Packagist badge |
| max 1000-line `.cs` check | same script targeting `.php` |
| hardcoded `MyPackage` in 3 scripts | derive package name from `composer.json` `name` |
| Tag scheme `csharp_v<ver>` / title `[C#]` | `php_v<ver>` / `[PHP]` |

The release-orchestration `.mjs` scripts are essentially language-agnostic
(Bun-run Node) and could be reused almost verbatim in the PHP template, with the
NuGet-specific probe/publish swapped for Packagist and the csproj parsing swapped
for `composer.json`/git-tag handling.
