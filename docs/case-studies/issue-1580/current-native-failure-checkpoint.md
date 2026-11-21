# Current native failures and gate inputs, 2026-11-21

The published head `8b0d6f59071c7577e8d0563d7220bef4db137001` has 27 terminal
workflows: 24 success, one skip and two failures. All eight native contract
families and the standalone registry-script job pass, qualifying the previous
receipt extraction and package-parser changes on that source. The complete run
logs are retained; every command stream is checked by compressed/raw hashes,
bytes and lines. Custody does not establish literal review of every log.

[Layered CI 44867529175](https://github.com/link-assistant/formal-ai/actions/runs/44867529175)
fails at full-log lines 3772–3793: a TERM-resistant worker is still runnable when
the deadline wrapper returns after sending SIGKILL. Signal delivery is asynchronous.
Both wrappers now wait for bounded process-group termination acknowledgement and
report inspection uncertainty explicitly. A finite delayed-delivery regression
fails on both old wrappers and passes after repair. The deadline, budget and clock
files pass 7, 8 and 2 cases respectively. ShellCheck and shell syntax pass.

[Security 44867525559](https://github.com/link-assistant/formal-ai/actions/runs/44867525559)
fails at full-log line 2801: the optional included-string observer exceeds its
unchanged four-minute deadline. The standard Rust security gate still passes.
The retained evaluator trace has 14,760 events and no completed query; overlapping
path-resolution predicates process tens of millions of rows. Pinned CodeQL source
shows that asking for a CFG node enters a cached stage with global dependencies.
Narrowing output rows alone does not bound that library work. The actual producer
receives four threads, while the observer previously forced two. The candidate
inherits a positive producer setting, caps it at four, retains a two-thread default
outside the action, and rejects invalid settings. Adapter and strict security files
pass 3 and 22 cases. This verifies argument handling; hosted performance is pending.
Query columns, unexpanded/other cases, 4 GiB policy and deadlines remain unchanged.

The native version guard now rejects a failed Git fetch before reading a stale
cached base version. Its actual CLI regression joins the hosted registry harness;
only formatting, Python syntax and JavaScript gates run locally. Pipeline status
also requires exactly one nonempty needs object with completed result records.
Fourteen focused schema cases pass; negative controls retain the old false success.

The pinned C++ gate independently accepts partial output from a failed policy
reader and an empty needs object. Both defects reproduce on the original source;
the disposable candidate passes all 18 controls. The full default-branch policy
file passes 144 cases, and three touched JavaScript gates pass. Current upstream
`45c42aff119b366ba103a9342ec3c33731bb7e2e` contains the identical gate. Reporting or
a local candidate does not establish an upstream repair.

Fresh default-head observations show seven of eight templates have changed since
their original pins; Java is unchanged. Raw heads and comparisons are retained.
The earlier per-file ledgers remain bound to their original trees. Review of the
fresh deltas and complete comparisons remain unfinished. A historical Rust log's
apparently successful Node tail masks an earlier stale pinned-action Python
assertion; the pinned current regression already uses a full-hash pattern.

Failed controller attempts, invalid negative controls, truncated previews and the
complete preview-overflow capture are retained without passing or reading credit.
All supplied patches and conclusions receive zero independent reasoning/authorship
credit. No local native build or CodeQL execution occurs. Complete template and
practice adoption, remaining security semantics and migrations, live publication
and runtime identity, source warnings, and final whole-PR review remain unfinished.
[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) stays draft and
unmerged. Evidence is indexed in
`experiments/issue-1580/evidence/checks/current-native-failure-repairs/index.json`;
the CI receipts are in `experiments/issue-1580/evidence/ci/native-families-8b0d6f59/`.
