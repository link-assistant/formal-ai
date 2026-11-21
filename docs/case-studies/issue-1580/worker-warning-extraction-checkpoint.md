# Worker source warning extraction

The unchanged size gate warned on the 1460-line meaning-lexicon twins and
1417-line code-task twins. The new focused regressions fail on those original
sources while their behavior controls pass. Extracting the existing notation
and refactoring sections into named modules removes four warnings. The retained
meaning lexicon is 1157 lines and notation is 302; code tasks is 1083 and
refactoring is 333. Each extraction has a verified exact byte inverse against
published source `84e1ad85`, including the removed blank separator. No behavior
or existing handler was removed, and the combined worker ceilings decrease by
one line per pair. The source limits and warning thresholds remain unchanged.

Canonical generation updates both module inventories and TypeScript twins.
The maintained status renderer updates its inventory from 177 to 179 worker
budgets. The initial full-gate pass rejected the stale status projection; that
failure remains retained separately from the final successful run.

All 45 focused tests and all 130 maintained observation-entry tests pass,
including both module loading orders, notation quoting and shared variable
bindings, refactoring success and rejection traces, and refusal controls.
The deliberate loop input is refused before execution. All 731 TypeScript
files match canonical generation, and the existing bounded TypeScript runtime
comparison passes. It does not establish complete native or product WASM
parity. No local Rust build was performed.

The complete 141-entry JavaScript gate run has 88 passes and 53 explicit skips,
with zero failures. The actual size gate reports 87 warnings and no violations.
The four removed warnings do not resolve those other warnings or the full
issue. Full gate streams, before-fix failures, helpers, source identities and
compressed archive hashes are retained in
`experiments/issue-1580/evidence/checks/worker-warning-extraction/`.

The published Workflows run
[44887270329](https://github.com/link-assistant/formal-ai/actions/runs/44887270329)
retains the expected official-action failure annotation from the deliberately
injected canonical BuildKit pull outage; mirror recovery succeeds. Its other
annotation is a macOS runner-capacity notice. Earlier artifact-lookup failure
annotations did not recur in this capture. These observations are preserved
without suppressing genuine setup failures or claiming all annotations fixed.

The implementation and commands were supplied through the actual JavaScript
Formal AI driver and receive zero independent reasoning or authorship credit.
Full template/practice, source-warning, original budget/live qualification,
security child semantics and complete pull-request acceptance remain open.

The post-archive naming check rejected four abbreviated evidence filenames.
Their full-word renames preserve the original source paths, bytes and hashes;
the failed 94-versus-90 observation remains retained. The ratchet is unchanged.
