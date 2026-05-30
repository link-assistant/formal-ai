# Issue #362 Coding-Modification Benchmark

Issue #362 asks for a bulk multilingual multi-turn coding-modification
benchmark with a ratchet and download-on-test integration for external edit
datasets. The suite added for this issue lives in
`data/benchmarks/coding-modification-suite.lino` and is exercised by
`issue_362_*` tests in `tests/unit/specification/coding_modification_benchmarks.rs`.

## Local Ratchet

The deterministic part of the benchmark keeps four multi-turn conversations:

- English self-authored list-files program modification.
- Russian issue #349 dialog: initial Rust list-files draft, path-argument edit,
  then the previously failing reverse-sort follow-up.
- Hindi self-authored equivalent.
- Chinese self-authored equivalent.

Every case follows `initial_draft|edit|edit` and asserts that the final answer
routes to `write_program`, keeps the Rust path-argument code, sorts file names
in reverse order, and records `program_parameter:task
list_files_arg_reverse_sort` in Links Notation. The manifest records
`minimum_pass_count "4"`, so ordinary CI fails if any current case regresses.

## External Sources

Full datasets are not committed. The ignored network test downloads these
parquet payloads into `target/formal-ai-benchmarks` only when
`FORMAL_AI_BULK_BENCHMARK=1` is set.

| Source | Use | License | Download |
| --- | --- | --- | --- |
| CanItEdit | instructed code-editing source | MIT | `nuprl/CanItEdit` test parquet |
| HumanEvalFix from HumanEvalPack | program-repair source | MIT | `bigcode/humanevalpack` Python test parquet |
| EDIT-Bench | real-world instructed code-editing source | Apache-2.0 | `copilot-arena/editbench` test parquet |

Source revisions, LFS hashes, license URLs, cache paths, and minimum byte floors
are recorded in the manifest and summarized in `data/benchmarks/LICENSES.md`.

## Audit Notes

The issue requested that the suite account for "Edit, But Verify" before using
external sources as a ratchet. That paper reports that CanItEdit and EDIT-Bench
are useful but narrow proxies: they are heavily Python-centered, underrepresent
several real-world edit categories, and parts of EDIT-Bench have thin tests or
benchmark-artifact failures. The implementation therefore does not ratchet on a
raw external dataset pass rate yet. External datasets are provenance-checked and
download-validated, while the CI ratchet is kept on deterministic formal-ai
multi-turn behavior.

References:

- Issue #362: <https://github.com/link-assistant/formal-ai/issues/362>
- Issue #349 regression dialog: <https://github.com/link-assistant/formal-ai/issues/349>
- CanItEdit: <https://github.com/nuprl/CanItEdit>
- HumanEvalPack: <https://huggingface.co/datasets/bigcode/humanevalpack>
- EDIT-Bench: <https://github.com/waynchi/editbench>
- Edit, But Verify: <https://arxiv.org/abs/2604.05100>
