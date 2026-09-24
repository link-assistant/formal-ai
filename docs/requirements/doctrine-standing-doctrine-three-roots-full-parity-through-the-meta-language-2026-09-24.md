## Standing Doctrine: Three Roots, Full Parity, Through The Meta Language (2026-09-24)

Stated by the project owner as a standing architectural requirement (PR #1139,
issue #1138; quoted in
[the 2026-09-24 architect note](../architect-notes/2026-09-24-three-roots-full-parity-via-the-meta-language.md)).
It supersedes the boundary rule of the 2026-08-04 doctrine above (R536):
JavaScript and TypeScript stop being interfacing-only glue and become full
implementation roots. What remains in force from R536 is its motivation — one
logic, no divergence between surfaces — now guaranteed by translation through
the meta language rather than by shrinking JavaScript out, and the reuse of the
WASM engine by surfaces that want it. The stated reason is iteration speed:
JavaScript executes faster than Rust compiles, so a js-first cycle with
mechanical translation outward shortens the loop.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R992 | Rust, JavaScript and TypeScript each fully implement the client, the backend server, and all other logic. No root is a browser-only or glue-only surface, and none is a rote copy of another: the three stay equivalent through the meta language. | Direction stated 2026-09-24; honest current state: `rust/` is the complete implementation; `js/` carries the web app plus 35 generated mirror modules (30,179 lines as of 2026-09-18) under the shrink ratchet; `ts/` is a README stub. The measured migration is [plan 16](../case-studies/issue-1138/plans/16-js-ts-rust-cycle.md) (L1 delivered; L2-L5 pending). |
| R993 | Translation between any two of the four roots — rust, js, ts, meta — is available as a library function and as a script (a CLI subcommand), through the meta language as the single pivot: translate(X → Y) is extract(X → meta) then render(meta → Y), and the dispatcher holds no per-pair code. A leg that is not materialized names the plan-16 leaf that owes it rather than returning a silent no-op. | Delivered 2026-09-24 (PR #1139): `rust/src/meta_translate.rs` (library entry `formal_ai::meta_translate::translate`, `SourceRoot`, `directions()`) and `formal-ai translate --from --to [--input PATH] [--list]`. Live leg: rust → meta renders the self-AST census document. All other legs report their owing plan-16 leaf. Covered by `rust/tests/unit/issue_1138_translation_tool.rs`. |
| R994 | The development cycle may run js-first: author and test in JavaScript, translate js → ts, and translate onward into Rust through the pivot, with parity pinned by round-trip verification and path-filtered CI that gates js before ts before rust. | Documented in [CONTRIBUTING.md](../../CONTRIBUTING.md) Development Workflow (2026-09-24). The path-filtered CI leg is plan 16 L4 and the CST-equal round trip is L5; both pending, so today the cycle is stated direction plus the R993 dispatcher, not yet a closed loop. |
| R995 | The 2026-08-04 interfacing-only-JavaScript boundary (R536) is superseded as stated above; the worker shrink ratchet `scripts/check-worker-line-budget.rs` stays in force as downward pressure on the generated mirror until the parity migration replaces those modules, after which its 3,000-line end-state dissolves rather than being met. | Doctrine row. R536's text above is unchanged as history; the ratchet and its `data/meta/worker-line-budget/` ceilings are unchanged today and remain a required gate. |
| R996 | The parity state is reported honestly wherever it is claimed: rust complete, js partial (web app plus generated mirror under ratchet), ts a stub, until each leg lands; no document may claim full three-root parity ahead of the measured migration. | This row and R992's status carry the current numbers; plan 16 records per-leaf progress. `docs/source-roots.md` states the per-root role, updated 2026-09-24. |
