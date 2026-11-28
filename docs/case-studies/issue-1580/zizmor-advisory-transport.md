# Zizmor advisory transport boundary

Run [44970848479](https://github.com/link-assistant/formal-ai/actions/runs/44970848479)
at `2809ce657a6216599252c929f436687652091302` passed the regular audit, then
failed the pedantic audit before producing a result. Lines 3204–3212 name the
`known-vulnerable-actions` audit and a transport failure fetching the advisory
for `actions/create-github-app-token@v3.4.0`. This was not a vulnerability verdict.

`scripts/run-zizmor.mjs` uses the same zizmor 1.29.0 image digest recorded by
zizmor-action v0.6.4. Both original personas, scope, confidence and severity
filters, GitHub annotations, read-only source mount and online checks remain.
Only the observed advisory transport diagnostic with exit status 1 and no error
annotation permits a complete audit retry: at most three attempts with 2 and 5
second delays. Findings, HTTP errors, invalid configuration, process failures,
image pull failures and exhausted retries remain failures. Every attempt's
stdout and stderr is preserved. The token is inherited through Docker's
environment rather than embedded in its arguments.

Reproduce the finite regression without Docker or network:

```sh
node --test --test-concurrency=1 experiments/issue-1580/zizmor-transport.test.mjs
```

The test replays the original diagnostic, verifies both complete command
boundaries and all fatal controls, and executes the actual CLI against a finite
Docker stand-in. The maintained template-observations suite imports it.

The same log showed an auditor warning about YAML anchors and two warnings
about the shell of dynamically selected macOS runners. Expanding the two CodeQL
aliases exposes both full producer bodies to static audit. An independent parsed
comparison confirmed every Security workflow value remained identical. The
macOS job now explicitly declares its existing Bash shell. These repairs change
audit visibility without removing either producer or verification step.

Local regression success does not establish current-head hosted audit success;
the subsequent complete CI run is still required. Supplied source edits and
replayed upstream code receive no independent Formal AI authorship credit.
