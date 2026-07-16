## Reproduction

The latest `Release` run audited for formal-ai issue 717 emits an unchanged-baseline warning on every run:

- run: https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/actions/runs/29435394576
- `scripts/version-and-commit.rs has 913 lines`, even when that file is not changed
- `.github/workflows/release.yml` also uses `codecov/codecov-action@v5`; GitHub now forces Node 20 actions onto Node 24 and emits a deprecation warning

## Expected

Keep the hard file-size check repository-wide, but emit warning-band annotations only for files changed by the PR (and retain a full baseline report as a non-annotation artifact/summary if desired). Upgrade Codecov to the current `codecov/codecov-action@v7`.

## Suggested test/fix

Add a fixture with one unchanged and one changed 901-line file. Both remain subject to the 1000-line hard limit, while only the changed file should create a warning annotation. Add a workflow policy assertion for `codecov/codecov-action@v7`.

Found while comparing all four pipeline templates for https://github.com/link-assistant/formal-ai/issues/717.
