The JavaScript template ships `.github/workflows/links.yml` (lychee + a Wayback
Machine fallback via `scripts/check-web-archive.mjs`), so a rotted URL in any
`*.md` / `*.html` file fails a pull request there. This template has **no link
validation of any kind**, so documentation links — including the README badges
and the links a downstream project copies into its own docs — rot undetected.

## Reproduction

Against `main` (verified 2026-08-05):

```bash
gh api repos/link-foundation/rust-ai-driven-development-pipeline-template/git/trees/HEAD?recursive=1 \
  --jq '.tree[] | select(.path | test("^\\.github/workflows/")) | .path'
# -> .github/workflows/release.yml   (only)

gh api repos/link-foundation/js-ai-driven-development-pipeline-template/git/trees/HEAD?recursive=1 \
  --jq '.tree[] | select(.path == ".github/workflows/links.yml") | .path'
# -> .github/workflows/links.yml
```

Concretely: add a link to a URL that returns 404 (for example
`[dead](https://example.com/definitely-not-a-page-404)`) to `README.md` and open
a pull request. In the JS template the `Check Links` job fails; here every check
passes and the dead link is merged.

## Workaround

Run lychee manually before touching docs:

```bash
docker run --rm -v "$PWD":/input lycheeverse/lychee \
  --no-progress --max-retries 3 --timeout 30 \
  --exclude-path docs/case-studies './**/*.md' './**/*.html'
```

That is a per-contributor manual step nobody remembers, which is exactly what
the JS template automated.

## Suggested fix

Port `js-ai-driven-development-pipeline-template`'s
[`.github/workflows/links.yml`](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/blob/main/.github/workflows/links.yml)
verbatim, together with `scripts/check-web-archive.mjs`, adapting three things:

1. **Helper language.** The JS version calls `node scripts/check-web-archive.mjs`.
   Port it to `scripts/check-web-archive.rs` so it runs under `rust-script`, matching the rest of `scripts/*.rs`; alternatively keep the `.mjs` helper, since the runner already has Node.
2. **Exclusions.** Keep `--exclude-path docs/case-studies` (those documents cite
   files and issues that exist only in other repositories). Drop the
   `examples/universal-app/index.html` exclusion — it is JS-template specific.
3. **Everything else transfers unchanged**: the `**.md` / `**.html` path
   filters, `permissions: contents: read`, the per-job `concurrency` group with
   `cancel-in-progress: true`, `timeout-minutes: 10`, `fail: false` plus the
   Wayback fallback gate, and the actionable error block that tells the
   contributor how to fix each broken link.

Two follow-up fixes the JS template already made and worth porting at the same
time: the missing `concurrency` block (js#73) and excluding source HTML whose
root-relative asset URLs are only valid when served by a bundler (js#95).

## Cross-template status

Verified on 2026-08-05:

| Template | HEAD checked | `.github/workflows/links.yml` |
|---|---|---|
| `js-ai-driven-development-pipeline-template` | `7b70923` | **present** (101 lines) |
| `rust-ai-driven-development-pipeline-template` | `c867f78` | absent |
| `python-ai-driven-development-pipeline-template` | `98d6dca` | absent |
| `csharp-ai-driven-development-pipeline-template` | `6806bd9` | absent |

## Provenance

Found by a four-template CI/CD comparison audit run in downstream
`link-assistant/formal-ai` (report:
<https://github.com/link-assistant/formal-ai/blob/main/docs/case-studies/issue-479/template-comparison/REPORT.md>,
finding 5 / upstream recommendation 2; filed under
<https://github.com/link-assistant/formal-ai/issues/894>). The downstream repo
has the same gap and is fixing it locally as well.
