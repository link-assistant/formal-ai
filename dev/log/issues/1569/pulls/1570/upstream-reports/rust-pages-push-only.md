## Problem

`deploy-docs` in `release.yml` runs only on a push to `main` (or an `instant` dispatch). Nothing on a pull request checks that the repository can deploy to GitHub Pages:

https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/blob/309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb/.github/workflows/release.yml#L1467-L1480

When Settings → Pages → Source is not "GitHub Actions" (the state of every repository freshly created from this template), every pull request is green and every push to `main` fails in `actions/configure-pages` with `Get Pages site failed ... Not Found`. `Pipeline Status` fails next. The pull request was a false negative: the defect was present before the merge, and nothing reported it.

The comment above the job already describes the first-deploy failure. However, it is only visible to someone who reads the workflow, and it appears only after the merge.

## Reproduction

Observed on this repository's own `main`: run [43566214814](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/actions/runs/43566214814) (2026-11-14).
- `Deploy Rust Documentation` → step `Configure GitHub Pages` failed.
- `Pipeline Status` → `Check aggregate pipeline status` failed.

Steps:
1. Create a repository from the template.
2. Leave Pages unconfigured.
3. Merge any pull request.
4. Watch the `main` run.

## Workaround

Set Settings → Pages → Source = "GitHub Actions" and rerun the failed jobs.

## Suggested fix

Either of these, both of which already exist in sibling templates:

1. **Opt-in gate.** Run the deploy only when `vars.DEPLOY_GITHUB_PAGES == 'true'` and print a `::notice::` otherwise, as the python, cpp and csharp templates do (csharp issue link-foundation/csharp-ai-driven-development-pipeline-template#33).
2. **Pull-request probe.** Probe the Pages source on pull requests in `warn` mode and on `main` in `require` mode, as the js template's `scripts/check-pages.sh` does (`example-app.yml` lines 96-101). The author then sees the missing setting before merging.
