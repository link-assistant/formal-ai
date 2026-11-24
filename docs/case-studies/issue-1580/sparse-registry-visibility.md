# Sparse registry visibility without false release conclusions

The API-only waiter in [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
could fail when Cargo's sparse index already exposed the requested version.
The pinned Rust template probes that index before falling back to the JSON API.
Its source was retrieved by the previously recorded Git blob identity, checked
against its original byte count and SHA-256, and read completely.

The waiter now follows that order and retains the local parsed-identity checks.
It parses every nonempty JSON line, validates the crate name and exact version,
and rejects malformed or incomplete bodies. Version-looking text inside another
field cannot authorize visibility. A validated API answer remains a fallback.
Only two unusable observations produce an unknown result; a definitive absence
from either endpoint remains distinct from an observation failure.

Each outer waiter attempt makes at most one request per endpoint, with the
existing bounded connection/read/global HTTP policy. The original attempt count,
sleep options and workflow budgets remain. Other registry callers retain their
existing four-attempt retry policy. Confirmed visibility still requires a
successful Actions-output write; template package identities still skip.

Six new compiled CLI methods cover sparse priority during API failure, short
and mixed-case crate names, fallback, spoofed/incomplete records, unknown versus
absent outcomes and failed output writes. The hashed original API-only source
is retained as the before-control; its native execution remains pending. The existing locked native harness compiles
both controls. One native parser/layout regression covers exact and missing
identities, malformed trailing records, short paths and invalid names.

The source contract fails before the repair and passes afterward; all 67
JavaScript contracts pass. Five real loopback fixture checks pass without a
Rust build. Three Python files parse; eleven affected JavaScript gates and
formatting of both Rust files with edition 2024 pass. The first fixture check
incorrectly expected the normal crate name in its deliberate wrong-identity
response; that failed result is retained beside the corrected expectation.

Reproduce the native regression using:
python3 experiments/issue-1580/release-registry/cli_regressions.py

The command compiles Rust and was not run locally. All six new native CLI
methods and the parser/layout test still need hosted execution. The loopback
fixture checks do not qualify production network behavior or publication
authority. They can be replayed without compilation using:
python3 experiments/issue-1580/verify-sparse-registry-controls.py

The [receipt](../../../experiments/issue-1580/evidence/checks/sparse-registry-visibility-20261123/manifest.json)
retains pinned upstream source, original/current local source, source bindings,
complete local logs and the failed draft. Every archived member was read back
and verified against its byte count and SHA-256. Full template-practice
qualification and all other original acceptance criteria remain unfinished.
The PR stays draft and unmerged; supplied work earns zero authorship credit.
