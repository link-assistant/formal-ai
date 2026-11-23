# Cargo publication credentials and native admission

The publishing helper in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
forwarded its selected token to Cargo with two process arguments. It also warned
that credentials were absent before Cargo could try a configured credential
provider. An empty command-line token suppressed the environment fallback.

Selected nonempty tokens now reach Cargo through CARGO_REGISTRY_TOKEN. The
existing command-line option and native/legacy environment precedence remain.
An empty override falls back; no explicit token leaves Cargo's provider handling
available and prints a neutral message. A rejected credential still returns
failure with publish_result=auth_failed.

The five new process regressions run the compiled helper against a recording
Cargo command. Synthetic fixture tokens cover precedence, empty overrides,
argument privacy, premature warnings and authentication failure. A hashed copy
of the original helper proves the former token argument and premature warning.
The fixture is compiled inside the existing locked release-registry harness.
No package or lockfile dependency is added. The native admission classifier now
covers that entire harness directory, including existing modules and fixtures
which previously could change without selecting native CI.

Run the small JavaScript contract with:
node --test --test-concurrency=1 experiments/issue-1580/native-regression-groups.test.mjs

Its new case fails against the original helper and passes after repair; all
66 cases pass. Two Python files parse, and both recording-Cargo stub controls
run locally. Eleven affected JavaScript gates, whitespace checks and rustfmt
with edition 2024 pass. The first formatting attempt used the wrong default
edition; its failure remains retained separately from the corrected check.
The earlier admission-contract failure is also retained alongside its repair.

The native regression command is:
python3 experiments/issue-1580/release-registry/cli_regressions.py

That command compiles the harness and belongs in the existing hosted native
job. It was not run locally. The five new native process methods, actual Cargo
credential-provider behavior and live publication authority remain unverified.
The recording stub does not implement a real credential provider.

The [receipt](../../../experiments/issue-1580/evidence/checks/publish-credential-repair-20261123/manifest.json)
retains original/current source, failed drafts, complete local logs and source
bindings. Every archive member was reopened and checked against its byte length
and SHA-256 hash. Hosted qualification and the broader original acceptance
criteria remain unfinished; the PR stays draft and unmerged. Supplied work earns
zero independent authorship credit.
