## Reproduction

The C# template still uses `actions/setup-dotnet@v4` in `release.yml` and `docs.yml`, and `codecov/codecov-action@v4` in `release.yml`. GitHub's runner now warns when actions target Node 20 and forces them onto Node 24.

Current releases are `actions/setup-dotnet@v5` and `codecov/codecov-action@v7`.

## Suggested fix

Upgrade every setup-dotnet occurrence to v5 and Codecov to v7. Update `scripts/release-workflow-policy.test.mjs`, which currently requires Codecov v4, to reject the old majors as regression coverage.

Found while comparing all four pipeline templates for https://github.com/link-assistant/formal-ai/issues/717.
