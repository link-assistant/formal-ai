## Reproduction

At commit `c3a2eb2eaaa5741c9ece6903c7675626d03e7ea3`, the template runs
`actions/dependency-review-action@v5`, but no `pip-audit`/OSV check over the
current Python dependency set. Dependency review only evaluates dependency
changes in pull requests, so a new advisory against an unchanged requirement
can leave every regular workflow green.

For a resolved requirements file, the missing check is:

```bash
python -m pip install pip-audit
pip-audit -r requirements.txt
```

For `pyproject.toml`, first resolve/export the environment using the template's
chosen package manager and audit that complete result.

## Workaround

Run `pip-audit` locally and on a schedule against the resolved application
dependencies.

## Suggested code fix

Pin `pip-audit`, cover each supported dependency declaration/lock, run the gate
on pull requests and pushes plus a schedule, and add a workflow test that fails
if a declared dependency surface is not mapped to an audit command.

