## Problem

In `docs.yml`, the `Configure GitHub Pages` and `Upload GitHub Pages artifact` steps of the **build** job, and the whole `deploy` job, run only on a push to `main`:

https://github.com/link-foundation/php-ai-driven-development-pipeline-template/blob/3a034b27a56067971ca74dd4c74c7707056b215d/.github/workflows/docs.yml#L62-L77

When Settings → Pages → Source is not "GitHub Actions", pull requests pass and every push to `main` fails. Because `configure-pages` runs inside `build`, the docs **build** job is reported as failed too, not just the deploy. A pull request that broke the docs build would therefore look the same as a missing Pages setting.

## Reproduction

The last four `docs.yml` runs on `main` all failed:
- [34994725744](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/actions/runs/34994725744) (2026-09-15)
- [37743150960](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/actions/runs/37743150960) (2026-10-08)
- [41539790049](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/actions/runs/41539790049) (2026-11-02)
- [43531031679](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/actions/runs/43531031679) (2026-11-13)

The latest one failed in `Build docs` → `Configure GitHub Pages` with `Get Pages site failed`, and `Pipeline Status` reported `Failing jobs: build`.

Steps:
1. Create a repository from the template.
2. Leave Pages unconfigured.
3. Merge a pull request that touches the docs.

## Workaround

Set Settings → Pages → Source = "GitHub Actions".

## Suggested fix

1. Gate the deploy on `vars.DEPLOY_GITHUB_PAGES == 'true'` and print a `::notice::` otherwise, as the python, cpp and csharp templates do. Alternatively, probe the Pages source on pull requests as the js template's `scripts/check-pages.sh` does.
2. Move `configure-pages` and `upload-pages-artifact` out of `build` into `deploy`. A docs build failure then stays distinguishable from a Pages setting failure.
